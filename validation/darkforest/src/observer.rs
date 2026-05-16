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
