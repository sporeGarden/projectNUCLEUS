use std::path::Path;

use tokio::fs;
use tokio::process::Command;

use super::report::SecurityReport;
use crate::config::NucleusConfig;

pub async fn layer_below(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
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
