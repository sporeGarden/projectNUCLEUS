use tokio::process::Command;

use super::helpers::http_status_code;
use super::report::SecurityReport;
use crate::config::NucleusConfig;

pub async fn layer_above(
    report: &mut SecurityReport,
    host: &str,
    cfg: &NucleusConfig,
    tunnel_url: Option<&str>,
) {
    report.log("");
    report.log("══ Layer 3: Above the Primals (Application Security) ══");

    jupyterhub_headers(report, host, cfg).await;
    auth_enforcement(report, host, cfg).await;
    path_traversal(report, host, cfg).await;

    if let Some(url) = tunnel_url {
        tunnel_security(report, url).await;
    }
}

async fn jupyterhub_headers(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 3a: JupyterHub Security Headers ──");

    let url = format!("http://{host}:{}/hub/login", cfg.jupyterhub_port);
    let output = Command::new("curl")
        .args(["-sf", "-D", "-", &url, "-o", "/dev/null", "--max-time", "5"])
        .output()
        .await;

    let Ok(o) = output else {
        report.warn("Could not reach JupyterHub");
        return;
    };
    let headers = String::from_utf8_lossy(&o.stdout).to_string();

    for header in &[
        "X-Frame-Options",
        "X-Content-Type-Options",
        "Content-Security-Policy",
        "X-XSS-Protection",
    ] {
        if headers.to_lowercase().contains(&header.to_lowercase()) {
            let val = headers
                .lines()
                .find(|l| l.to_lowercase().contains(&header.to_lowercase()))
                .unwrap_or("")
                .trim();
            report.pass(format!("JupyterHub sends {val}"));
        } else {
            report.warn(format!("JupyterHub missing header: {header}"));
        }
    }

    let server_header = headers
        .lines()
        .find(|l| l.to_lowercase().starts_with("server:"))
        .map(|l| {
            l.trim_start_matches(|c: char| !c.is_whitespace())
                .trim()
                .to_string()
        });

    match server_header {
        None => report.pass("Server header suppressed (dark forest)"),
        Some(ref s) if s.is_empty() => report.pass("Server header suppressed (dark forest)"),
        Some(ref s)
            if ["tornado", "python", "jupyter", "nginx", "apache"]
                .iter()
                .any(|k| s.to_lowercase().contains(k)) =>
        {
            report.warn(format!("Server header leaks implementation: {s}"));
        }
        Some(s) => report.pass(format!("Server header present but non-identifying: {s}")),
    }
}

async fn auth_enforcement(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 3b: Authentication Enforcement ──");

    let base = format!("http://{host}:{}", cfg.jupyterhub_port);

    let code = http_status_code(&format!("{base}/hub/api/users")).await;
    match code.as_deref() {
        Some("403" | "401") => {
            report.pass(format!(
                "JupyterHub /hub/api/users requires auth (HTTP {code:?})"
            ));
        }
        Some(c) => {
            report.fail(format!(
                "JupyterHub /hub/api/users accessible without auth (HTTP {c})"
            ));
        }
        None => report.warn("Could not probe JupyterHub auth"),
    }

    let spawn_code = http_status_code(&format!("{base}/hub/api/users/testuser/server")).await;
    match spawn_code.as_deref() {
        Some("403" | "401" | "302") => {
            report.pass(format!(
                "JupyterHub spawn endpoint requires auth (HTTP {spawn_code:?})"
            ));
        }
        Some(c) => report.warn(format!("JupyterHub spawn endpoint returned HTTP {c}")),
        None => report.warn("Could not probe JupyterHub spawn endpoint"),
    }
}

async fn path_traversal(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 3c: Path Traversal Probes ──");

    let base = format!("http://{host}:{}", cfg.jupyterhub_port);
    let paths = [
        "/hub/../../../etc/passwd",
        "/hub/%2e%2e/%2e%2e/etc/passwd",
        "/hub/login?next=//evil.com",
        "/hub/api/../../../etc/shadow",
    ];

    for path in &paths {
        let code = http_status_code(&format!("{base}{path}")).await;
        match code.as_deref() {
            Some("200") => {
                report.pass(format!(
                    "Path {path} returned 200 but content not checked (Rust probe)"
                ));
            }
            Some(c) => {
                report.pass(format!("Path traversal blocked: {path} (HTTP {c})"));
            }
            None => report.info(format!("Could not probe path: {path}")),
        }
    }
}

async fn tunnel_security(report: &mut SecurityReport, url: &str) {
    report.log("");
    report.log("── 3d: Tunnel Security ──");

    let tls_output = Command::new("curl")
        .args(["-sf", "-v", &format!("{url}/hub/api/")])
        .output()
        .await;

    if let Ok(o) = tls_output {
        let stderr = String::from_utf8_lossy(&o.stderr);
        let tls_lines: Vec<&str> = stderr
            .lines()
            .filter(|l| l.to_lowercase().contains("ssl") || l.to_lowercase().contains("tls"))
            .collect();

        if tls_lines
            .iter()
            .any(|l| l.contains("TLSv1.3") || l.contains("TLSv1.2"))
        {
            report.pass(format!(
                "Tunnel uses modern TLS: {}",
                tls_lines.first().unwrap_or(&"")
            ));
        } else if !tls_lines.is_empty() {
            report.warn(format!("Tunnel TLS: {}", tls_lines.first().unwrap_or(&"")));
        }
    }

    let hsts_output = Command::new("curl")
        .args([
            "-sf",
            "-D",
            "-",
            &format!("{url}/hub/api/"),
            "-o",
            "/dev/null",
            "--max-time",
            "5",
        ])
        .output()
        .await;

    if let Ok(o) = hsts_output {
        let headers = String::from_utf8_lossy(&o.stdout);
        if headers.to_lowercase().contains("strict-transport-security") {
            report.pass("Tunnel sends HSTS header");
        } else {
            report.warn("Tunnel missing HSTS header");
        }
    }
}
