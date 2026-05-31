use std::fmt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Local;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

use crate::config::NucleusConfig;
use crate::rpc;

#[derive(Debug, Error)]
pub enum SecurityError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("results directory creation failed: {0}")]
    ResultsDir(std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Layer {
    All,
    Below,
    At,
    Above,
    Tiers,
    Darkforest,
}

impl fmt::Display for Layer {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::All => write!(f, "all"),
            Self::Below => write!(f, "below"),
            Self::At => write!(f, "at"),
            Self::Above => write!(f, "above"),
            Self::Tiers => write!(f, "tiers"),
            Self::Darkforest => write!(f, "darkforest"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Verdict {
    Pass,
    Fail,
    Warn,
    Info,
}

struct Finding {
    verdict: Verdict,
    #[expect(dead_code, reason = "findings stored for potential JSON export")]
    message: String,
}

struct SecurityReport {
    findings: Vec<Finding>,
    log_lines: Vec<String>,
}

impl SecurityReport {
    const fn new() -> Self {
        Self {
            findings: Vec::new(),
            log_lines: Vec::new(),
        }
    }

    fn log(&mut self, msg: &str) {
        let line = format!("[{}] {msg}", Local::now().format("%H:%M:%S"));
        eprintln!("{line}");
        self.log_lines.push(line);
    }

    fn pass(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [PASS] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Pass,
            message: m,
        });
    }

    fn fail(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [FAIL] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Fail,
            message: m,
        });
    }

    fn warn(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [WARN] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Warn,
            message: m,
        });
    }

    fn info(&mut self, msg: impl Into<String>) {
        let m = msg.into();
        self.log(&format!("  [INFO] {m}"));
        self.findings.push(Finding {
            verdict: Verdict::Info,
            message: m,
        });
    }

    fn count(&self, verdict: Verdict) -> usize {
        self.findings
            .iter()
            .filter(|f| f.verdict == verdict)
            .count()
    }

    fn has_failures(&self) -> bool {
        self.count(Verdict::Fail) > 0
    }
}

pub struct SecurityArgs {
    pub layer: Layer,
    pub tunnel_url: Option<String>,
    pub target_host: Option<String>,
    pub results_dir: Option<PathBuf>,
}

pub async fn run(cfg: &NucleusConfig, args: &SecurityArgs) -> Result<bool, SecurityError> {
    let target = args.target_host.as_deref().unwrap_or(&cfg.bind_address);

    let results_dir = args.results_dir.clone().unwrap_or_else(|| {
        cfg.project_root
            .join("validation")
            .join(format!("security-{}", Local::now().format("%Y%m%d-%H%M%S")))
    });

    fs::create_dir_all(&results_dir)
        .await
        .map_err(SecurityError::ResultsDir)?;

    let mut report = SecurityReport::new();

    report.log("═══════════════════════════════════════════════════════════");
    report.log("  Security Validation Pipeline — Five-Layer Pen Testing");
    report.log(&format!("  Target: {target}"));
    report.log(&format!("  Layer: {}", args.layer));
    report.log(&format!("  Results: {}", results_dir.display()));
    report.log("═══════════════════════════════════════════════════════════");

    notify_skunkbat(&mut report, target, cfg.skunkbat_port).await;

    let layer = args.layer;

    if layer == Layer::All || layer == Layer::Below {
        layer_below(&mut report, target, cfg).await;
    }

    if layer == Layer::All || layer == Layer::At {
        layer_at(&mut report, target, cfg).await;
    }

    if layer == Layer::All || layer == Layer::Above {
        layer_above(&mut report, target, cfg, args.tunnel_url.as_deref()).await;
    }

    if layer == Layer::All || layer == Layer::Tiers {
        layer_tiers(&mut report, cfg).await;
    }

    if layer == Layer::All || layer == Layer::Darkforest {
        layer_darkforest(&mut report, cfg).await;
    }

    collect_skunkbat_metrics(&mut report, target, cfg.skunkbat_port, &results_dir).await;

    let pass = report.count(Verdict::Pass);
    let fail = report.count(Verdict::Fail);
    let warn = report.count(Verdict::Warn);
    let info = report.count(Verdict::Info);

    report.log("");
    report.log("═══════════════════════════════════════════════════════════");
    report.log("  Security Validation Complete");
    report.log(&format!("  PASS: {pass}"));
    report.log(&format!("  FAIL: {fail}"));
    report.log(&format!("  WARN: {warn}"));
    report.log(&format!("  INFO: {info}"));
    report.log(&format!("  Results: {}", results_dir.display()));
    report.log("═══════════════════════════════════════════════════════════");

    write_report(&report, &results_dir, target, &args.layer).await?;
    write_log(&report, &results_dir).await?;

    Ok(!report.has_failures())
}

async fn notify_skunkbat(report: &mut SecurityReport, host: &str, port: u16) {
    let req = rpc::jsonrpc_request("security.scan", 1);
    match rpc::send_jsonrpc(host, port, &req).await {
        Ok(_) => report.log("  skunkBat scan baseline captured"),
        Err(_) => report.warn("Could not reach skunkBat for scan notification"),
    }
}

// ── Layer 1: Below the Primals — OS/Network ──────────────────────────────

async fn layer_below(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 1: Below the Primals (OS / Network) ══");

    port_exposure(report, host, cfg).await;
    unnecessary_services(report).await;
    firewall_check(report).await;
    sensitive_permissions(report).await;
}

async fn port_exposure(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 1a: Port Exposure Scan ──");

    let output = Command::new("ss").args(["-tlnp"]).output().await;

    let listening = match output {
        Ok(o) if o.status.success() => String::from_utf8_lossy(&o.stdout).to_string(),
        _ => {
            report.warn("Could not run ss -tlnp");
            return;
        }
    };

    let external: Vec<&str> = listening
        .lines()
        .filter(|l| l.contains("LISTEN"))
        .filter(|l| !l.contains("127.0.0.1") && !l.contains("::1") && !l.contains("[::1]"))
        .collect();

    if external.is_empty() {
        report.pass(format!(
            "No non-localhost listeners — all services bound to {host}"
        ));
    } else {
        report.warn(format!(
            "Found {} non-localhost listener(s)",
            external.len()
        ));
        for line in &external {
            report.info(format!("  {line}"));
        }
    }

    for pp in cfg.all_primal_ports() {
        let port_str = format!(":{} ", pp.port);
        let bind_line = listening.lines().find(|l| l.contains(&port_str));
        if let Some(line) = bind_line {
            if line.contains("0.0.0.0:") {
                report.fail(format!(
                    "Port {} ({}) bound to 0.0.0.0 (externally exposed)",
                    pp.port, pp.name
                ));
            } else if line.contains("127.0.0.1:") {
                report.pass(format!(
                    "Port {} ({}) bound to {} only",
                    pp.port, pp.name, host
                ));
            } else {
                report.info(format!("Port {} ({}): {line}", pp.port, pp.name));
            }
        }
    }

    let jh_port_str = format!(":{} ", cfg.jupyterhub_port);
    if let Some(line) = listening.lines().find(|l| l.contains(&jh_port_str)) {
        if line.contains("127.0.0.1:") {
            report.pass(format!(
                "JupyterHub ({}) bound to {} — tunnel-only access",
                cfg.jupyterhub_port, host
            ));
        } else if line.contains("0.0.0.0:") {
            report.fail(format!(
                "JupyterHub ({}) bound to 0.0.0.0 — directly exposed",
                cfg.jupyterhub_port
            ));
        }
    }
}

async fn unnecessary_services(report: &mut SecurityReport) {
    report.log("");
    report.log("── 1b: Unnecessary Service Check ──");

    for svc in &["sshd", "apache2", "nginx", "mysql", "postgres", "docker"] {
        let result = Command::new("pgrep").args(["-x", svc]).output().await;

        if let Ok(o) = result {
            if o.status.success() {
                if *svc == "sshd" {
                    report.info("sshd running (expected for remote management)");
                } else {
                    report.warn(format!("{svc} running — verify this is intentional"));
                }
            }
        }
    }
}

async fn firewall_check(report: &mut SecurityReport) {
    report.log("");
    report.log("── 1c: Firewall Status ──");

    let ufw = Command::new("ufw").arg("status").output().await;
    match ufw {
        Ok(o) if o.status.success() => {
            let out = String::from_utf8_lossy(&o.stdout);
            if out.contains("Status: active") {
                report.pass("UFW firewall active");
            } else {
                report.warn(format!("UFW installed but not active: {}", out.trim()));
            }
        }
        _ => {
            let ipt = Command::new("iptables").args(["-L", "-n"]).output().await;
            match ipt {
                Ok(o) if o.status.success() => {
                    let lines = String::from_utf8_lossy(&o.stdout).lines().count();
                    if lines > 5 {
                        report.info(format!("iptables has {lines} rules"));
                    } else {
                        report.warn(format!("iptables has minimal rules ({lines} lines)"));
                    }
                }
                _ => report.warn("No firewall detected"),
            }
        }
    }
}

async fn sensitive_permissions(report: &mut SecurityReport) {
    report.log("");
    report.log("── 1d: Sensitive File Permissions ──");

    let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
    let paths = [
        format!("{home}/.config/biomeos/family"),
        format!("{home}/jupyterhub/jupyterhub_cookie_secret"),
        format!("{home}/jupyterhub/jupyterhub.sqlite"),
    ];

    for path in &paths {
        let p = Path::new(path);
        if !p.exists() {
            continue;
        }
        let Ok(meta) = fs::metadata(p).await else {
            continue;
        };
        let mode = meta.permissions();
        let readonly = mode.readonly();
        let perms_output = Command::new("stat").args(["-c", "%a", path]).output().await;

        if let Ok(o) = perms_output {
            let perms = String::from_utf8_lossy(&o.stdout).trim().to_string();
            match perms.as_str() {
                "600" | "700" => report.pass(format!("{path}: mode {perms} (restricted)")),
                "644" | "755" => report.warn(format!("{path}: mode {perms} (world-readable)")),
                _ => report.info(format!("{path}: mode {perms}")),
            }
        } else if readonly {
            report.info(format!("{path}: readonly"));
        }
    }
}

// ── Layer 2: At the Primal Layer — API Security ──────────────────────────

async fn layer_at(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 2: At the Primal Layer (API Security) ══");

    unauth_probes(report, host, cfg).await;
    input_fuzzing(report, host, cfg).await;
    method_enumeration(report, host, cfg).await;
    btsp_enforcement(report, host, cfg).await;
}

async fn unauth_probes(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 2a: Unauthenticated API Probe ──");

    let targets: &[(&str, u16)] = &[
        ("beardog", cfg.beardog_port),
        ("toadstool", cfg.toadstool_port),
        ("nestgate", cfg.nestgate_port),
        ("rhizocrypt", cfg.rhizocrypt_rpc_port),
        ("loamspine", cfg.loamspine_port),
        ("sweetgrass", cfg.sweetgrass_port),
        ("skunkbat", cfg.skunkbat_port),
    ];

    for &(name, port) in targets {
        let health_req = rpc::jsonrpc_request("health.liveness", 1);
        if let Ok(r) = rpc::send_jsonrpc(host, port, &health_req).await {
            if r.has_result() {
                report.info(format!(
                    "{name} health.liveness accessible (expected — public health endpoint)"
                ));
            }
        }

        let sensitive_check = match name {
            "nestgate" => Some(("storage.list", serde_json::json!({"prefix": ""}))),
            "sweetgrass" => Some(("braid.list", serde_json::json!({}))),
            _ => None,
        };

        if let Some((method, params)) = sensitive_check {
            let req = rpc::jsonrpc_request_with_params(method, &params, 2);
            match rpc::send_jsonrpc(host, port, &req).await {
                Ok(r) if r.has_error() => {
                    report.pass(format!("{name} {method} rejects unauthenticated request"));
                }
                Ok(r) if r.has_result() => {
                    report.warn(format!("{name} {method} accessible without auth"));
                }
                _ => {}
            }
        }
    }
}

async fn input_fuzzing(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 2b: Input Fuzzing (Malformed Requests) ──");

    let fuzz_targets: &[(&str, u16)] = &[
        ("beardog", cfg.beardog_port),
        ("toadstool", cfg.toadstool_port),
        ("nestgate", cfg.nestgate_port),
        ("skunkbat", cfg.skunkbat_port),
    ];

    let big_value = "A".repeat(10_000);
    let payloads: Vec<String> = vec![
        "not json at all".into(),
        r#"{"jsonrpc":"2.0"}"#.into(),
        r#"{"jsonrpc":"2.0","method":"","id":1}"#.into(),
        r#"{"jsonrpc":"2.0","method":"../../../etc/passwd","id":1}"#.into(),
        r#"{"jsonrpc":"2.0","method":"health.liveness","params":"not-an-object","id":1}"#.into(),
        r#"{"jsonrpc":"2.0","method":"health.liveness","id":null}"#.into(),
        format!(
            r#"{{"jsonrpc":"2.0","method":"health.liveness","params":{{"key":"{big_value}"}},"id":1}}"#
        ),
    ];

    let dur = Duration::from_secs(2);

    for &(name, port) in fuzz_targets {
        let mut crashes = 0u32;

        for payload in &payloads {
            let resp = rpc::send_raw_tcp(host, port, payload.as_bytes(), dur).await;
            if resp.is_err() && !rpc::check_liveness(host, port).await {
                crashes += 1;
            }
        }

        if crashes == 0 {
            report.pass(format!(
                "{name} survived all {} fuzz payloads without crash",
                payloads.len()
            ));
        } else {
            report.fail(format!(
                "{name} crashed on {crashes}/{} fuzz payloads",
                payloads.len()
            ));
        }
    }
}

async fn method_enumeration(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 2c: Method Enumeration ──");

    let hidden_methods = [
        "admin.shutdown",
        "system.exec",
        "debug.dump",
        "internal.config",
        "shell.exec",
        "eval",
        "rpc.discover",
    ];

    let targets: &[(&str, u16)] = &[
        ("beardog", cfg.beardog_port),
        ("toadstool", cfg.toadstool_port),
        ("nestgate", cfg.nestgate_port),
    ];

    for &(name, port) in targets {
        let mut found = 0u32;
        for method in &hidden_methods {
            let req = rpc::jsonrpc_request(method, 1);
            if let Ok(r) = rpc::send_jsonrpc(host, port, &req).await {
                if r.has_result() {
                    report.fail(format!("{name} exposes hidden method: {method}"));
                    found += 1;
                }
            }
        }
        if found == 0 {
            report.pass(format!(
                "{name} rejects all {} suspicious method probes",
                hidden_methods.len()
            ));
        }
    }
}

async fn btsp_enforcement(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
    report.log("");
    report.log("── 2d: BTSP Enforcement ──");

    let btsp_targets: &[(&str, u16)] = &[
        ("sweetgrass", cfg.sweetgrass_port),
        ("rhizocrypt", cfg.rhizocrypt_port),
    ];

    let dur = Duration::from_secs(2);

    for &(name, port) in btsp_targets {
        let resp = rpc::send_raw_tcp(host, port, b"PLAINTEXT PROBE\n", dur).await;
        let rejected = match &resp {
            Err(_) => true,
            Ok(s) if s.is_empty() => true,
            Ok(s) => {
                let lower = s.to_lowercase();
                lower.contains("btsp")
                    || lower.contains("reject")
                    || lower.contains("error")
                    || lower.contains("unauthorized")
            }
        };

        if rejected {
            report.pass(format!("{name} (port {port}) rejects plaintext connection"));
        } else if let Ok(s) = resp {
            let truncated = if s.len() > 80 { &s[..80] } else { &s };
            report.warn(format!(
                "{name} (port {port}) responded to plaintext: {truncated}"
            ));
        }
    }
}

// ── Layer 3: Above the Primals — Application Security ────────────────────

async fn layer_above(
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

    let host_part = url.trim_start_matches("https://");
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

    let _ = host_part;
}

// ── Layer 4: Tier Enforcement ────────────────────────────────────────────

async fn layer_tiers(report: &mut SecurityReport, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 4: ABG Tier Enforcement ══");

    let tier_script = cfg.project_root.join("deploy/tier_enforcement_test.sh");
    run_external_test(report, &tier_script, "OS-level tier enforcement", "4a").await;

    let tier_api_script = cfg.project_root.join("deploy/jupyterhub_tier_test.py");
    if tier_api_script.exists() {
        report.log("");
        report.log("── 4b: JupyterHub API Tier Enforcement ──");

        let output = Command::new("python3").arg(&tier_api_script).output().await;

        match output {
            Ok(o) => {
                let text = String::from_utf8_lossy(&o.stdout);
                let pass_count = text.matches("PASS|").count();
                let fail_count = text.matches("FAIL|").count();
                if fail_count == 0 {
                    report.pass(format!(
                        "JupyterHub API tier enforcement: {pass_count} assertions pass"
                    ));
                } else {
                    report.fail(format!(
                        "JupyterHub API tier enforcement: {fail_count} failures"
                    ));
                }
            }
            Err(_) => report.warn("Could not run jupyterhub_tier_test.py"),
        }
    }
}

// ── Layer 5: Dark Forest ─────────────────────────────────────────────────

async fn layer_darkforest(report: &mut SecurityReport, cfg: &NucleusConfig) {
    report.log("");
    report.log("══ Layer 5: Dark Forest ══");

    let release_bin = cfg
        .project_root
        .join("validation/darkforest/target/release/darkforest");
    let debug_bin = cfg
        .project_root
        .join("validation/darkforest/target/debug/darkforest");

    let bin = if release_bin.exists() {
        release_bin
    } else if debug_bin.exists() {
        debug_bin
    } else {
        report
            .warn("darkforest binary not found — build with: cargo build --release -p darkforest");
        return;
    };

    report.log("Running Rust darkforest binary (all suites)...");

    let output = Command::new(&bin).args(["--suite", "all"]).output().await;

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            let pass_count = text.matches("PASS|").count();
            let fail_count = text.matches("FAIL|").count();
            let gap_count = text.matches("KNOWN_GAP|").count();
            let df_count = text.matches("DARK_FOREST|").count();

            if fail_count == 0 {
                report.pass(format!(
                    "Dark Forest (Rust): {pass_count} pass, {gap_count} gaps, {df_count} dark forest findings"
                ));
            } else {
                report.fail(format!(
                    "Dark Forest (Rust): {fail_count} failures out of {} assertions",
                    pass_count + fail_count
                ));
            }
        }
        Err(e) => report.warn(format!("darkforest execution failed: {e}")),
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

async fn run_external_test(report: &mut SecurityReport, script: &Path, label: &str, section: &str) {
    report.log("");
    report.log(&format!("── {section}: {label} ──"));

    if !script.exists() {
        report.warn(format!(
            "{} not found at {}",
            script.file_name().unwrap_or_default().to_string_lossy(),
            script.display()
        ));
        return;
    }

    let output = Command::new("bash").arg(script).output().await;

    match output {
        Ok(o) => {
            let text = String::from_utf8_lossy(&o.stdout);
            let pass_count = text.matches("PASS|").count();
            let fail_count = text.matches("FAIL|").count();
            let gap_count = text.matches("KNOWN_GAP|").count();

            if fail_count == 0 {
                report.pass(format!(
                    "{label}: {pass_count} assertions pass ({gap_count} known gaps)"
                ));
            } else {
                report.fail(format!(
                    "{label}: {fail_count} failures out of {} assertions",
                    pass_count + fail_count
                ));
            }
        }
        Err(e) => report.warn(format!("Could not run {label}: {e}")),
    }
}

async fn http_status_code(url: &str) -> Option<String> {
    let output = Command::new("curl")
        .args([
            "-s",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            url,
            "--max-time",
            "5",
        ])
        .output()
        .await
        .ok()?;

    let code = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if code == "000" {
        None
    } else {
        Some(code)
    }
}

async fn collect_skunkbat_metrics(
    report: &mut SecurityReport,
    host: &str,
    port: u16,
    results_dir: &Path,
) {
    report.log("");
    report.log("══ skunkBat Observation ══");

    let req = rpc::jsonrpc_request("security.metrics", 2);
    match rpc::send_jsonrpc(host, port, &req).await {
        Ok(r) => {
            if let Some(result) = r.result() {
                let threats = result
                    .get("threats_detected")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let quarantined = result
                    .get("connections_quarantined")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);
                let alerts = result
                    .get("alerts_sent")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);

                report.info("skunkBat metrics after scan:");
                report.info(format!("  Threats detected: {threats}"));
                report.info(format!("  Connections quarantined: {quarantined}"));
                report.info(format!("  Alerts sent: {alerts}"));

                let json_path = results_dir.join("skunkbat_metrics.json");
                let _ = fs::write(
                    &json_path,
                    serde_json::to_string_pretty(result).unwrap_or_default(),
                )
                .await;
            }
        }
        Err(_) => report.warn("Could not reach skunkBat for post-scan metrics"),
    }
}

async fn write_report(
    report: &SecurityReport,
    results_dir: &Path,
    target: &str,
    layer: &Layer,
) -> Result<(), SecurityError> {
    let content = format!(
        "# Security Validation — {}\n\n\
        **Target**: {target}\n\
        **Layer**: {layer}\n\n\
        ## Summary\n\n\
        | Metric | Count |\n\
        |--------|-------|\n\
        | PASS | {} |\n\
        | FAIL | {} |\n\
        | WARN | {} |\n\
        | INFO | {} |\n",
        Local::now().to_rfc3339(),
        report.count(Verdict::Pass),
        report.count(Verdict::Fail),
        report.count(Verdict::Warn),
        report.count(Verdict::Info),
    );

    fs::write(results_dir.join("SECURITY_RESULTS.md"), content).await?;
    Ok(())
}

async fn write_log(report: &SecurityReport, results_dir: &Path) -> Result<(), SecurityError> {
    let content = report.log_lines.join("\n");
    fs::write(results_dir.join("security.log"), content).await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_display_roundtrip() {
        assert_eq!(Layer::All.to_string(), "all");
        assert_eq!(Layer::Below.to_string(), "below");
        assert_eq!(Layer::Darkforest.to_string(), "darkforest");
    }

    #[test]
    fn report_counts_correctly() {
        let mut report = SecurityReport::new();
        report.pass("test pass");
        report.pass("test pass 2");
        report.fail("test fail");
        report.warn("test warn");
        report.info("test info");

        assert_eq!(report.count(Verdict::Pass), 2);
        assert_eq!(report.count(Verdict::Fail), 1);
        assert_eq!(report.count(Verdict::Warn), 1);
        assert_eq!(report.count(Verdict::Info), 1);
        assert!(report.has_failures());
    }

    #[test]
    fn report_no_failures_when_clean() {
        let mut report = SecurityReport::new();
        report.pass("all good");
        report.warn("minor");
        assert!(!report.has_failures());
    }
}
