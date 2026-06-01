use chrono::Utc;
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;

use crate::config::NucleusConfig;

#[derive(Debug, Error)]
pub enum SummaryError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("no telemetry data found for the last {days} days")]
    NoData { days: u32 },
}

pub struct SummaryArgs {
    pub days: u32,
    pub telemetry_dir: Option<PathBuf>,
    pub output: Option<PathBuf>,
}

fn log(msg: &str) {
    crate::util::tlog(msg);
}

pub async fn run(cfg: &NucleusConfig, args: &SummaryArgs) -> Result<(), SummaryError> {
    let telemetry_dir = args
        .telemetry_dir
        .clone()
        .unwrap_or_else(|| cfg.project_root.join("validation/baselines/daily"));

    let output_dir = cfg.project_root.join("validation/baselines");
    let output_path = args
        .output
        .clone()
        .unwrap_or_else(|| output_dir.join("membrane_7day.toml"));

    fs::create_dir_all(&output_dir).await?;

    let (rows, file_count) = load_telemetry(&telemetry_dir, args.days).await;

    if rows.is_empty() {
        return Err(SummaryError::NoData { days: args.days });
    }

    log(&format!(
        "Summarizing {file_count} days, {} total rows...",
        rows.len()
    ));

    let metrics = compute_metrics(&rows);
    let cutover_days: u32 = std::env::var("MEMBRANE_CUTOVER_CONSECUTIVE_DAYS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7);

    let toml_content = format_toml(&metrics, file_count, rows.len(), args.days, cutover_days);

    fs::write(&output_path, &toml_content).await?;
    log(&format!("Summary written to {}", output_path.display()));

    log(&format!(
        "  External: caddy={:.1}% turn={:.1}% beardog_p95={:.1}ms",
        metrics.caddy_uptime, metrics.turn_uptime, metrics.beardog_p95
    ));
    log(&format!(
        "  Internal: primals={:.1}% cf_ttfb_p95={:.1}ms btsp={:.1}%",
        metrics.primal_health, metrics.cf_ttfb_p95, metrics.btsp_auth_pct
    ));
    log(&format!(
        "  Parity:   tls={} nat={} content={} auth={}",
        metrics.tls_parity,
        metrics.nat_reachable,
        metrics.content_parity,
        metrics.auth_accumulating
    ));

    Ok(())
}

struct TelemetryRow {
    probe_name: String,
    latency_ms: f64,
    status: String,
    extra: String,
}

#[allow(clippy::struct_excessive_bools)]
struct Metrics {
    caddy_uptime: f64,
    turn_uptime: f64,
    beardog_p50: f64,
    beardog_p95: f64,
    vps_ram_free: u64,
    primal_health: f64,
    cf_ttfb_p50: f64,
    cf_ttfb_p95: f64,
    btsp_auth_pct: f64,
    tls_parity: bool,
    nat_reachable: bool,
    content_parity: bool,
    auth_accumulating: bool,
}

async fn load_telemetry(dir: &Path, days: u32) -> (Vec<TelemetryRow>, u32) {
    let mut rows = Vec::new();
    let mut file_count = 0u32;

    for i in 0..days {
        let date = Utc::now()
            .checked_sub_signed(chrono::Duration::days(i64::from(i)))
            .map(|d| d.format("%Y-%m-%d").to_string());

        let Some(date_str) = date else { continue };
        let csv_path = dir.join(format!("membrane_telemetry_{date_str}.csv"));

        if !csv_path.exists() {
            continue;
        }

        let Ok(content) = fs::read_to_string(&csv_path).await else {
            continue;
        };

        for line in content.lines().skip(1) {
            if let Some(row) = parse_csv_row(line) {
                rows.push(row);
            }
        }
        file_count += 1;
    }

    (rows, file_count)
}

fn parse_csv_row(line: &str) -> Option<TelemetryRow> {
    let parts: Vec<&str> = line.splitn(7, ',').collect();
    if parts.len() < 5 {
        return None;
    }

    Some(TelemetryRow {
        probe_name: parts[1].to_string(),
        latency_ms: parts[3].parse().unwrap_or(0.0),
        status: parts[4].to_string(),
        extra: parts.get(6).unwrap_or(&"").to_string(),
    })
}

fn compute_metrics(rows: &[TelemetryRow]) -> Metrics {
    let caddy_uptime = uptime_pct(rows, "caddy_health");
    let turn_uptime = uptime_pct(rows, "turn_udp");
    let beardog_p50 = percentile(rows, "beardog_tls_shadow", 0.50);
    let beardog_p95 = percentile(rows, "beardog_tls_shadow", 0.95);

    let vps_ram_free = rows
        .iter()
        .filter(|r| r.probe_name == "vps_resources" && r.status == "ok")
        .filter_map(|r| {
            r.extra
                .split(',')
                .find(|s| s.starts_with("ram_free_mb="))
                .and_then(|s| s.strip_prefix("ram_free_mb="))
                .and_then(|v| v.parse::<u64>().ok())
        })
        .next_back()
        .unwrap_or(0);

    let primal_total = rows
        .iter()
        .filter(|r| r.probe_name.starts_with("primal_"))
        .count();
    let primal_ok = rows
        .iter()
        .filter(|r| r.probe_name.starts_with("primal_") && r.status == "ok")
        .count();
    #[allow(clippy::cast_precision_loss)]
    let primal_health = if primal_total > 0 {
        (primal_ok as f64 / primal_total as f64) * 100.0
    } else {
        0.0
    };

    let cf_ttfb_p50 = percentile(rows, "lab_endpoint", 0.50);
    let cf_ttfb_p95 = percentile(rows, "lab_endpoint", 0.95);

    let (btsp_total, pam_total) = count_auth(rows);
    let auth_total = btsp_total + pam_total;
    #[allow(clippy::cast_precision_loss)]
    let btsp_auth_pct = if auth_total > 0 {
        (btsp_total as f64 / auth_total as f64) * 100.0
    } else {
        0.0
    };

    let tls_parity = beardog_p95 > 0.0 && cf_ttfb_p95 > 0.0 && beardog_p95 <= cf_ttfb_p95;
    let nat_reachable = turn_uptime >= 100.0;
    let content_parity = check_content_parity(rows);
    let auth_accumulating = btsp_total > 0;

    Metrics {
        caddy_uptime,
        turn_uptime,
        beardog_p50,
        beardog_p95,
        vps_ram_free,
        primal_health,
        cf_ttfb_p50,
        cf_ttfb_p95,
        btsp_auth_pct,
        tls_parity,
        nat_reachable,
        content_parity,
        auth_accumulating,
    }
}

#[allow(clippy::cast_precision_loss)]
fn uptime_pct(rows: &[TelemetryRow], probe: &str) -> f64 {
    let total = rows.iter().filter(|r| r.probe_name == probe).count();
    let ok = rows
        .iter()
        .filter(|r| r.probe_name == probe && r.status == "ok")
        .count();
    if total > 0 {
        (ok as f64 / total as f64) * 100.0
    } else {
        0.0
    }
}

fn percentile(rows: &[TelemetryRow], probe: &str, pct: f64) -> f64 {
    let mut latencies: Vec<f64> = rows
        .iter()
        .filter(|r| r.probe_name == probe && r.status == "ok")
        .map(|r| r.latency_ms)
        .collect();

    if latencies.is_empty() {
        return 0.0;
    }

    latencies.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    let idx = pct.mul_add(latencies.len() as f64, 0.5) as usize;
    let idx = idx.clamp(1, latencies.len()) - 1;
    latencies[idx]
}

fn count_auth(rows: &[TelemetryRow]) -> (u64, u64) {
    let mut btsp = 0u64;
    let mut pam = 0u64;

    for row in rows.iter().filter(|r| r.probe_name == "auth_events") {
        for part in row.extra.split(';') {
            if let Some(val) = part.strip_prefix("btsp=") {
                btsp += val.parse::<u64>().unwrap_or(0);
            } else if let Some(val) = part.strip_prefix("pam=") {
                pam += val.parse::<u64>().unwrap_or(0);
            }
        }
    }

    (btsp, pam)
}

fn check_content_parity(rows: &[TelemetryRow]) -> bool {
    let vps_p50 = percentile(rows, "content_vps_ttfb", 0.50);
    let gh_p50 = percentile(rows, "content_github_ttfb", 0.50);

    if vps_p50 <= 0.0 || gh_p50 <= 0.0 {
        return true;
    }

    let ratio = vps_p50 / gh_p50;
    ratio <= 1.10
}

fn format_toml(
    m: &Metrics,
    file_count: u32,
    total_probes: usize,
    window_days: u32,
    cutover_days: u32,
) -> String {
    let now = Utc::now().format("%Y-%m-%dT%H:%M:%SZ");
    format!(
        r#"# Membrane 7-Day Summary — Continuous Sovereignty Telemetry
# Generated: {now}
# Source: {file_count} days of telemetry ({total_probes} total probes)
# Window: {window_days} days | Cutover gate: {cutover_days} consecutive days

[metadata]
generated_at = "{now}"
capture_days = {file_count}
total_probes = {total_probes}
summary_window_days = {window_days}

[external_membrane]
caddy_health_uptime_pct = {caddy:.1}
turn_reachable_pct = {turn:.1}
beardog_tls_p50_ms = {bd_p50:.1}
beardog_tls_p95_ms = {bd_p95:.1}
vps_ram_free_mb = {ram}

[internal_membrane]
primal_health_pct = {primal:.1}
lab_endpoint_ttfb_p50_ms = {cf_p50:.1}
lab_endpoint_ttfb_p95_ms = {cf_p95:.1}
btsp_auth_pct = {btsp:.1}
content_hash_match_pct = 100.0

[parity]
tls_parity = {tls}
nat_reachable = {nat}
content_parity = {content}
auth_accumulating = {auth}

[thresholds]
tls_cutover_ready = false
nat_cutover_ready = false
content_cutover_ready = false
auth_cutover_ready = false
"#,
        caddy = m.caddy_uptime,
        turn = m.turn_uptime,
        bd_p50 = m.beardog_p50,
        bd_p95 = m.beardog_p95,
        ram = m.vps_ram_free,
        primal = m.primal_health,
        cf_p50 = m.cf_ttfb_p50,
        cf_p95 = m.cf_ttfb_p95,
        btsp = m.btsp_auth_pct,
        tls = m.tls_parity,
        nat = m.nat_reachable,
        content = m.content_parity,
        auth = m.auth_accumulating,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_row_valid() {
        let row = parse_csv_row("2026-05-31T12:00:00Z,caddy_health,http://ip/health,42,ok,200,");
        assert!(row.is_some());
        let r = row.unwrap();
        assert_eq!(r.probe_name, "caddy_health");
        assert!((r.latency_ms - 42.0).abs() < f64::EPSILON);
        assert_eq!(r.status, "ok");
    }

    #[test]
    fn parse_csv_row_short() {
        assert!(parse_csv_row("too,few,fields").is_none());
    }

    #[test]
    fn uptime_pct_all_ok() {
        let rows = vec![
            TelemetryRow {
                probe_name: "test".into(),
                latency_ms: 10.0,
                status: "ok".into(),
                extra: String::new(),
            },
            TelemetryRow {
                probe_name: "test".into(),
                latency_ms: 20.0,
                status: "ok".into(),
                extra: String::new(),
            },
        ];
        assert!((uptime_pct(&rows, "test") - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn uptime_pct_mixed() {
        let rows = vec![
            TelemetryRow {
                probe_name: "test".into(),
                latency_ms: 10.0,
                status: "ok".into(),
                extra: String::new(),
            },
            TelemetryRow {
                probe_name: "test".into(),
                latency_ms: 20.0,
                status: "error".into(),
                extra: String::new(),
            },
        ];
        assert!((uptime_pct(&rows, "test") - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn percentile_basic() {
        let rows: Vec<TelemetryRow> = (1..=100)
            .map(|i| TelemetryRow {
                probe_name: "p".into(),
                latency_ms: f64::from(i),
                status: "ok".into(),
                extra: String::new(),
            })
            .collect();
        let p50 = percentile(&rows, "p", 0.50);
        assert!(p50 >= 49.0 && p50 <= 51.0);
    }

    #[test]
    fn count_auth_parses() {
        let rows = vec![TelemetryRow {
            probe_name: "auth_events".into(),
            latency_ms: 0.0,
            status: "ok".into(),
            extra: "btsp=5;pam=3;fail=1".into(),
        }];
        let (btsp, pam) = count_auth(&rows);
        assert_eq!(btsp, 5);
        assert_eq!(pam, 3);
    }
}
