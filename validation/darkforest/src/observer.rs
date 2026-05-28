use crate::check::{Category, CheckBuilder, CheckResult, Severity};
use crate::net::http_get;
use std::fs;
use std::path::{Path, PathBuf};

const SUITE: &str = "observer";
const THEME_MARKER: &str = "jp-layout-color0: #0d1117";
const VOILA_LINK: &str = r#"href="/voila/render/"#;
const TRACEBACK: &str = "Traceback (most recent call last)";
const INPUT_PROMPT_MARKER: &str = r#"class="jp-InputPrompt"#;

fn observer_port() -> u16 {
    std::env::var("VOILA_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8866)
}

fn html_dir() -> PathBuf {
    let gate = std::env::var("GATE_HOME")
        .or_else(|_| std::env::var("HOME"))
        .unwrap_or_else(|_| "/home/nobody".into());
    let abg = std::env::var("ABG_SHARED").unwrap_or_else(|_| format!("{gate}/shared/abg"));
    PathBuf::from(abg).join("public/.pappusCast/html_export")
}

fn collect_html(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                files.extend(collect_html(&p));
            } else if p.extension().is_some_and(|e| e == "html") {
                files.push(p);
            }
        }
    }
    files.sort();
    files
}

/// OBS-01: Verify rendered HTML files exist
fn check_html_exists(dir: &Path, results: &mut Vec<CheckResult>) -> Vec<PathBuf> {
    let cb = CheckBuilder::new("OBS-01", SUITE, Category::Observer, Severity::High)
        .remediation("Run 'pappusCast.py export' to generate static HTML");
    let files = collect_html(dir);
    if files.is_empty() {
        results.push(cb.fail("No HTML files in html_export/", &dir.display().to_string()));
    } else {
        results.push(cb.pass(
            &format!("{} HTML files in html_export/", files.len()),
            &dir.display().to_string(),
        ));
    }
    files
}

/// OBS-02: Dark theme CSS injected into every page
fn check_theme(dir: &Path, files: &[PathBuf], results: &mut Vec<CheckResult>) {
    for file in files {
        if file.file_name().is_some_and(|n| n == "index.html") {
            continue;
        }
        let rel = file.strip_prefix(dir).unwrap_or(file);
        let id = format!("OBS-02:{}", rel.display());
        let cb = CheckBuilder::new(&id, SUITE, Category::Observer, Severity::Medium)
            .remediation("Ensure _inject_nav() applies observer_theme.css");
        match fs::read_to_string(file) {
            Ok(body) if body.contains(THEME_MARKER) => {
                results.push(cb.pass("Dark theme CSS present", &rel.display().to_string()));
            }
            Ok(_) => {
                results.push(cb.fail(
                    "Missing dark theme CSS override",
                    &rel.display().to_string(),
                ));
            }
            Err(e) => {
                results.push(cb.fail("Cannot read file", &e.to_string()));
            }
        }
    }
}

/// OBS-03: Navigation bar present with Home link
fn check_nav(dir: &Path, files: &[PathBuf], results: &mut Vec<CheckResult>) {
    for file in files {
        if file.file_name().is_some_and(|n| n == "index.html") {
            continue;
        }
        let rel = file.strip_prefix(dir).unwrap_or(file);
        let id = format!("OBS-03:{}", rel.display());
        let cb = CheckBuilder::new(&id, SUITE, Category::Observer, Severity::Medium)
            .remediation("Ensure _inject_nav() inserts <nav> element");
        match fs::read_to_string(file) {
            Ok(body) if body.contains("<nav ") && body.contains("index.html") => {
                results.push(cb.pass("Nav bar with Home link present", &rel.display().to_string()));
            }
            Ok(_) => {
                results.push(cb.fail("Missing nav bar or Home link", &rel.display().to_string()));
            }
            Err(e) => {
                results.push(cb.fail("Cannot read file", &e.to_string()));
            }
        }
    }
}

/// OBS-04: No legacy Voila links remain
fn check_no_voila_links(dir: &Path, files: &[PathBuf], results: &mut Vec<CheckResult>) {
    for file in files {
        if file.file_name().is_some_and(|n| n == "index.html") {
            continue;
        }
        let rel = file.strip_prefix(dir).unwrap_or(file);
        let id = format!("OBS-04:{}", rel.display());
        let cb = CheckBuilder::new(&id, SUITE, Category::Observer, Severity::High)
            .remediation("_inject_nav must rewrite /voila/render/ links to .html");
        match fs::read_to_string(file) {
            Ok(body) => {
                let count = body.matches(VOILA_LINK).count();
                if count > 0 {
                    results.push(cb.fail(
                        &format!("{count} broken Voila links remaining"),
                        &rel.display().to_string(),
                    ));
                } else {
                    results.push(cb.pass("No legacy Voila links", &rel.display().to_string()));
                }
            }
            Err(e) => {
                results.push(cb.fail("Cannot read file", &e.to_string()));
            }
        }
    }
}

/// OBS-05: No tracebacks in rendered output
fn check_no_tracebacks(dir: &Path, files: &[PathBuf], results: &mut Vec<CheckResult>) {
    for file in files {
        if file.file_name().is_some_and(|n| n == "index.html") {
            continue;
        }
        let rel = file.strip_prefix(dir).unwrap_or(file);
        let id = format!("OBS-05:{}", rel.display());
        let cb = CheckBuilder::new(&id, SUITE, Category::Observer, Severity::High)
            .remediation("Re-export notebook after fixing errors");
        match fs::read_to_string(file) {
            Ok(body) => {
                let count = body.matches(TRACEBACK).count();
                if count > 0 {
                    results.push(cb.fail(
                        &format!("{count} traceback(s) in rendered output"),
                        &rel.display().to_string(),
                    ));
                } else {
                    results.push(cb.pass("No tracebacks", &rel.display().to_string()));
                }
            }
            Err(e) => {
                results.push(cb.fail("Cannot read file", &e.to_string()));
            }
        }
    }
}

/// OBS-06: Source code stripped (--no-input)
fn check_source_stripped(dir: &Path, files: &[PathBuf], results: &mut Vec<CheckResult>) {
    for file in files {
        if file.file_name().is_some_and(|n| n == "index.html") {
            continue;
        }
        let rel = file.strip_prefix(dir).unwrap_or(file);
        let id = format!("OBS-06:{}", rel.display());
        let cb = CheckBuilder::new(&id, SUITE, Category::InfoLeak, Severity::High)
            .remediation("Export with --no-input flag");
        match fs::read_to_string(file) {
            Ok(body) => {
                let count = body.matches(INPUT_PROMPT_MARKER).count();
                if count > 0 {
                    results.push(cb.fail(
                        &format!("{count} code input cells visible"),
                        &rel.display().to_string(),
                    ));
                } else {
                    results.push(cb.pass("No source code visible", &rel.display().to_string()));
                }
            }
            Err(e) => {
                results.push(cb.fail("Cannot read file", &e.to_string()));
            }
        }
    }
}

/// OBS-07: Root index serves HTTP 200 with expected content
fn check_root_index(host: &str, results: &mut Vec<CheckResult>) {
    let port = observer_port();
    let cb = CheckBuilder::new("OBS-07", SUITE, Category::Observer, Severity::High)
        .remediation("Ensure observer-static.service is running on port 8866");
    match http_get(host, port, "/", "", 5000) {
        Some((200, _, body)) if body.contains("NUCLEUS Observer") => {
            results.push(cb.pass("Root serves index.html with NUCLEUS Observer", "HTTP 200"));
        }
        Some((code, _, body)) => {
            let snippet = &body[..body.len().min(100)];
            results.push(cb.fail(&format!("HTTP {code}, missing expected content"), snippet));
        }
        None => {
            results.push(cb.fail(
                "Cannot connect to static observer",
                &format!("{host}:{port}"),
            ));
        }
    }
}

/// OBS-08: Security response headers
fn check_response_headers(host: &str, results: &mut Vec<CheckResult>) {
    let port = observer_port();
    let expected = [
        ("X-Robots-Tag", "noai"),
        ("X-Content-Type-Options", "nosniff"),
        ("X-Frame-Options", "DENY"),
        ("Referrer-Policy", "no-referrer"),
    ];
    if let Some((_, headers, _)) = http_get(host, port, "/", "", 5000) {
        for (name, value) in &expected {
            let id = format!("OBS-08:{name}");
            let cb = CheckBuilder::new(&id, SUITE, Category::Network, Severity::Medium)
                .remediation("Add header to ObserverHandler.end_headers()");
            let needle = format!("{name}: {value}");
            if headers.contains(&needle) {
                results.push(cb.pass(&format!("{name}: {value}"), &needle));
            } else {
                results.push(cb.fail(
                    &format!("Missing or wrong {name}"),
                    &format!("Expected '{value}' in headers"),
                ));
            }
        }
    } else {
        let cb = CheckBuilder::new("OBS-08", SUITE, Category::Network, Severity::Medium)
            .remediation("Ensure observer-static.service is running");
        results.push(cb.fail(
            "Cannot connect to static observer",
            &format!("{host}:{port}"),
        ));
    }
}

/// OBS-09: Internal directories not served
fn check_dir_blocking(host: &str, results: &mut Vec<CheckResult>) {
    let port = observer_port();
    let blocked = [
        "envs",
        "wheelhouse",
        "templates",
        ".ipynb_checkpoints",
        ".pappusCast",
    ];
    for dir in &blocked {
        let id = format!("OBS-09:{dir}");
        let cb = CheckBuilder::new(&id, SUITE, Category::Isolation, Severity::Medium)
            .remediation("Static server should only serve html_export/");
        let path = format!("/{dir}/");
        match http_get(host, port, &path, "", 3000) {
            Some((code, _, _)) if code == 404 || code == 403 => {
                results.push(cb.pass(&format!("/{dir}/ blocked (HTTP {code})"), ""));
            }
            Some((code, _, _)) => {
                results.push(cb.fail(
                    &format!("/{dir}/ accessible (HTTP {code})"),
                    "Should return 403 or 404",
                ));
            }
            None => {
                results.push(cb.pass(&format!("/{dir}/ not accessible"), "Connection refused"));
            }
        }
    }
}

pub fn run(host: &str, results: &mut Vec<CheckResult>) {
    println!("\n── OBS: Static Observer Surface ──");

    let dir = html_dir();
    println!("  html_dir: {}", dir.display());

    let files = check_html_exists(&dir, results);
    if files.is_empty() {
        return;
    }

    check_theme(&dir, &files, results);
    check_nav(&dir, &files, results);
    check_no_voila_links(&dir, &files, results);
    check_no_tracebacks(&dir, &files, results);
    check_source_stripped(&dir, &files, results);

    println!("\n── OBS: HTTP Behavior ──");
    check_root_index(host, results);
    check_response_headers(host, results);
    check_dir_blocking(host, results);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::check::Status;
    use std::fs;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn temp_html_export() -> (PathBuf, PathBuf) {
        let n = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
        let base = std::env::temp_dir().join(format!("darkforest_observer_test_{n}"));
        let dir = base.join("html_export");
        let _ = fs::remove_dir_all(&base);
        fs::create_dir_all(&dir).expect("create html_export");
        (base, dir)
    }

    fn write_page(dir: &Path, name: &str, body: &str) {
        fs::write(dir.join(name), body).expect("write html");
    }

    fn good_page_body() -> String {
        format!(
            "<html><head>{THEME_MARKER}</head><body><nav ><a href=\"index.html\">Home</a></nav></body></html>"
        )
    }

    fn cleanup(base: &Path) {
        let _ = fs::remove_dir_all(base);
    }

    #[test]
    fn collect_html_finds_nested_files() {
        let (base, dir) = temp_html_export();
        let nested = dir.join("subdir");
        fs::create_dir_all(&nested).expect("mkdir");
        write_page(&dir, "index.html", "<html></html>");
        write_page(&nested, "page.html", "<html></html>");
        fs::write(dir.join("notes.txt"), "skip").expect("write txt");

        let files = collect_html(&dir);
        assert_eq!(files.len(), 2);
        assert!(files.iter().any(|p| p.ends_with("page.html")));

        cleanup(&base);
    }

    #[test]
    fn check_html_exists_fails_on_empty_dir() {
        let (base, dir) = temp_html_export();
        let mut results = Vec::new();
        let files = check_html_exists(&dir, &mut results);
        assert!(files.is_empty());
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn check_html_exists_passes_with_files() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let mut results = Vec::new();
        let files = check_html_exists(&dir, &mut results);
        assert_eq!(files.len(), 1);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_theme_passes_with_marker() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_theme(&dir, &files, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_theme_fails_without_marker() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", "<html><body>plain</body></html>");
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_theme(&dir, &files, &mut results);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn check_theme_skips_index_html() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "index.html", "<html><body>no theme</body></html>");
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_theme(&dir, &files, &mut results);
        assert!(results.is_empty());
        cleanup(&base);
    }

    #[test]
    fn check_nav_passes_with_home_link() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_nav(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_nav_fails_without_nav() {
        let (base, dir) = temp_html_export();
        write_page(
            &dir,
            "page.html",
            &format!("<html><head>{THEME_MARKER}</head><body>no nav</body></html>"),
        );
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_nav(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn check_no_voila_links_passes_when_clean() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_no_voila_links(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_no_voila_links_fails_when_present() {
        let (base, dir) = temp_html_export();
        write_page(
            &dir,
            "page.html",
            &format!("<html>{VOILA_LINK}{THEME_MARKER}</html>"),
        );
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_no_voila_links(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn check_no_tracebacks_passes_when_clean() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_no_tracebacks(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_no_tracebacks_fails_when_present() {
        let (base, dir) = temp_html_export();
        write_page(
            &dir,
            "page.html",
            &format!("<html>{TRACEBACK}{THEME_MARKER}</html>"),
        );
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_no_tracebacks(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn check_source_stripped_passes_without_input_prompt() {
        let (base, dir) = temp_html_export();
        write_page(&dir, "page.html", &good_page_body());
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_source_stripped(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Pass);
        cleanup(&base);
    }

    #[test]
    fn check_source_stripped_fails_with_input_prompt() {
        let (base, dir) = temp_html_export();
        write_page(
            &dir,
            "page.html",
            &format!("<html>{INPUT_PROMPT_MARKER}{THEME_MARKER}</html>"),
        );
        let files = collect_html(&dir);
        let mut results = Vec::new();
        check_source_stripped(&dir, &files, &mut results);
        assert_eq!(results[0].status, Status::Fail);
        cleanup(&base);
    }

    #[test]
    fn html_dir_ends_with_html_export() {
        let dir = html_dir();
        assert!(dir.ends_with("public/.pappusCast/html_export"));
    }

    #[test]
    fn observer_port_defaults_to_8866() {
        let port = observer_port();
        assert!(port == 8866 || std::env::var("VOILA_PORT").is_ok());
    }
}
