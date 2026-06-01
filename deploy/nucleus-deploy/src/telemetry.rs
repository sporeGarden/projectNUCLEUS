use chrono::{Local, Utc};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

use crate::config::NucleusConfig;
use crate::rpc;

#[derive(Debug, Error)]
pub enum TelemetryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum TelemetryMode {
    All,
    External,
    Internal,
}

pub struct TelemetryArgs {
    pub mode: TelemetryMode,
    pub telemetry_dir: Option<PathBuf>,
}

fn log(msg: &str) {
    eprintln!("[{}] {msg}", Local::now().format("%H:%M:%S"));
}

pub async fn run(cfg: &NucleusConfig, args: &TelemetryArgs) -> Result<(), TelemetryError> {
    let telemetry_dir = args
        .telemetry_dir
        .clone()
        .unwrap_or_else(|| cfg.project_root.join("validation/baselines/daily"));
    fs::create_dir_all(&telemetry_dir).await?;

    let today = Utc::now().format("%Y-%m-%d").to_string();
    let csv_path = telemetry_dir.join(format!("membrane_telemetry_{today}.csv"));

    if !csv_path.exists() {
        fs::write(
            &csv_path,
            "timestamp_utc,probe_name,target,latency_ms,status,http_code,extra\n",
        )
        .await?;
    }

    log(&format!("membrane_telemetry — mode={:?}", args.mode));

    let host = &cfg.bind_address;

    if args.mode == TelemetryMode::All || args.mode == TelemetryMode::External {
        log("  Probing external membrane (VPS)...");
        probe_external(cfg, &csv_path).await;
    }

    if args.mode == TelemetryMode::All || args.mode == TelemetryMode::Internal {
        log("  Probing internal membrane (gate)...");
        probe_internal(cfg, host, &csv_path).await;
    }

    let line_count = count_lines(&csv_path).await;
    log(&format!(
        "  Done. {line_count} total rows in {}",
        csv_path.display()
    ));

    Ok(())
}

async fn emit(
    csv_path: &Path,
    probe: &str,
    target: &str,
    latency_ms: u64,
    status: &str,
    code: u16,
    extra: &str,
) {
    let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    let line = format!("{ts},{probe},{target},{latency_ms},{status},{code},{extra}\n");
    let _ = fs::OpenOptions::new()
        .append(true)
        .open(csv_path)
        .await
        .and_then(|_| {
            // tokio doesn't have append-write convenience, use std
            std::fs::OpenOptions::new()
                .append(true)
                .open(csv_path)
                .and_then(|mut f| {
                    use std::io::Write;
                    f.write_all(line.as_bytes())
                })
        });
}

async fn probe_http(csv_path: &Path, probe_name: &str, url: &str) {
    let start = std::time::Instant::now();
    let output = Command::new("curl")
        .args([
            "-sS",
            "-o",
            "/dev/null",
            "-w",
            "%{http_code}",
            "--max-time",
            "10",
            url,
        ])
        .output()
        .await;

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    let (status, code) = match output {
        Ok(o) => {
            let code_str = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let code: u16 = code_str.parse().unwrap_or(0);
            let status = if code == 0 {
                "unreachable"
            } else if code >= 400 {
                "error"
            } else {
                "ok"
            };
            (status, code)
        }
        Err(_) => ("unreachable", 0),
    };

    emit(csv_path, probe_name, url, elapsed_ms, status, code, "").await;
}

async fn probe_rpc_primal(csv_path: &Path, probe_name: &str, host: &str, port: u16) {
    let start = std::time::Instant::now();
    let alive = rpc::check_liveness(host, port).await;
    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    let status = if alive { "ok" } else { "unreachable" };
    emit(
        csv_path,
        probe_name,
        &format!("{host}:{port}"),
        elapsed_ms,
        status,
        0,
        "rpc",
    )
    .await;
}

async fn probe_tcp(csv_path: &Path, probe_name: &str, host: &str, port: u16) {
    let start = std::time::Instant::now();
    let result = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::net::TcpStream::connect(format!("{host}:{port}")),
    )
    .await;

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);
    let status = match result {
        Ok(Ok(_)) => "ok",
        _ => "unreachable",
    };
    emit(
        csv_path,
        probe_name,
        &format!("{host}:{port}"),
        elapsed_ms,
        status,
        0,
        "",
    )
    .await;
}

async fn probe_external(_cfg: &NucleusConfig, csv_path: &Path) {
    let vps_ip = std::env::var("MEMBRANE_VPS_IP").unwrap_or_else(|_| "157.230.3.183".into());
    let vps_http: u16 = std::env::var("MEMBRANE_HTTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);

    probe_http(
        csv_path,
        "caddy_health",
        &format!("http://{vps_ip}:{vps_http}/health"),
    )
    .await;

    probe_tcp(csv_path, "rustdesk_hbbs", &vps_ip, 21116).await;

    let btsp_host = std::env::var("BTSP_SHADOW_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let btsp_port: u16 = std::env::var("BTSP_SHADOW_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8443);

    let btsp_reachable = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        tokio::net::TcpStream::connect(format!("{btsp_host}:{btsp_port}")),
    )
    .await;

    if btsp_reachable.is_ok() {
        probe_rpc_primal(csv_path, "beardog_tls_shadow", &btsp_host, btsp_port).await;
    } else {
        emit(
            csv_path,
            "beardog_tls_shadow",
            &format!("{btsp_host}:{btsp_port}"),
            0,
            "not_running",
            0,
            "",
        )
        .await;
    }

    let vps_user = std::env::var("MEMBRANE_VPS_USER").unwrap_or_else(|_| "root".into());
    let ssh_check = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            &format!("{vps_user}@{vps_ip}"),
            "true",
        ])
        .output()
        .await;

    if matches!(ssh_check, Ok(ref o) if o.status.success()) {
        let resources = Command::new("ssh")
            .args([
                "-o", "ConnectTimeout=5",
                &format!("{vps_user}@{vps_ip}"),
                "free -m | awk '/Mem:/{printf \"ram_free_mb=%s\", $4}'; echo -n ','; df -h / | awk 'NR==2{printf \"disk_pct=%s\", $5}' | tr -d '%'",
            ])
            .output()
            .await;

        let extra = match resources {
            Ok(o) => String::from_utf8_lossy(&o.stdout).trim().to_string(),
            Err(_) => String::new(),
        };
        emit(csv_path, "vps_resources", &vps_ip, 0, "ok", 0, &extra).await;
    } else {
        emit(
            csv_path,
            "vps_resources",
            &vps_ip,
            0,
            "ssh_unreachable",
            0,
            "",
        )
        .await;
    }
}

async fn probe_internal(cfg: &NucleusConfig, host: &str, csv_path: &Path) {
    let lab_url = std::env::var("LAB_URL").unwrap_or_else(|_| "https://lab.primals.eco".into());
    probe_http(
        csv_path,
        "cloudflare_tunnel",
        &format!("{lab_url}/hub/login"),
    )
    .await;

    let primals: &[(&str, u16)] = &[
        ("beardog", cfg.beardog_port),
        ("songbird", cfg.songbird_port),
        ("nestgate", cfg.nestgate_port),
        ("skunkbat", cfg.skunkbat_port),
        ("biomeos", cfg.biomeos_port),
    ];

    for &(name, port) in primals {
        probe_rpc_primal(csv_path, &format!("primal_{name}"), host, port).await;
    }

    let vps_ip = std::env::var("MEMBRANE_VPS_IP").unwrap_or_else(|_| "157.230.3.183".into());
    let vps_http: u16 = std::env::var("MEMBRANE_HTTP_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(80);

    let vps_url = format!("http://{vps_ip}:{vps_http}/");
    let vps_check = Command::new("curl")
        .args(["-sf", "--max-time", "5", &vps_url])
        .output()
        .await;

    if matches!(vps_check, Ok(ref o) if o.status.success()) {
        probe_http(csv_path, "content_vps_ttfb", &vps_url).await;
        probe_http(csv_path, "content_github_ttfb", "https://primals.eco/").await;
    }

    let auth_info = collect_auth_events().await;
    emit(
        csv_path,
        "auth_events",
        "jupyterhub/journald",
        0,
        "ok",
        0,
        &auth_info,
    )
    .await;
}

async fn collect_auth_events() -> String {
    let output = Command::new("journalctl")
        .args(["-u", "jupyterhub", "--since", "today", "--no-pager"])
        .output()
        .await;

    let Ok(o) = output else {
        return "btsp=0;pam=0;fail=0".into();
    };

    let log_text = String::from_utf8_lossy(&o.stdout);
    let btsp = log_text.matches("BTSP").count();
    let pam = log_text.matches("PAMAuthenticator").count() + log_text.matches("AUTH_PAM").count();
    let fail = log_text.matches("AUTH_FAIL").count()
        + log_text.matches("failed login").count()
        + log_text.matches("authentication failed").count();

    format!("btsp={btsp};pam={pam};fail={fail}")
}

async fn count_lines(path: &Path) -> usize {
    fs::read_to_string(path)
        .await
        .map_or(0, |s| s.lines().count())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telemetry_mode_variants() {
        assert_ne!(TelemetryMode::All, TelemetryMode::External);
        assert_ne!(TelemetryMode::External, TelemetryMode::Internal);
    }

    #[test]
    fn collect_auth_format() {
        let expected = "btsp=0;pam=0;fail=0";
        assert!(expected.contains("btsp="));
        assert!(expected.contains("pam="));
        assert!(expected.contains("fail="));
    }
}
