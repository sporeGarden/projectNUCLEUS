use std::path::Path;

use serde_json::Value;
use tokio::fs;
use tokio::process::Command;

use super::report::SecurityReport;
use crate::rpc;

pub async fn run_external_test(
    report: &mut SecurityReport,
    script: &Path,
    label: &str,
    section: &str,
) {
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

pub async fn http_status_code(url: &str) -> Option<String> {
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

pub async fn notify_skunkbat(report: &mut SecurityReport, host: &str, port: u16) {
    let req = rpc::jsonrpc_request("security.scan", 1);
    match rpc::send_jsonrpc(host, port, &req).await {
        Ok(_) => report.log("  skunkBat scan baseline captured"),
        Err(_) => report.warn("Could not reach skunkBat for scan notification"),
    }
}

pub async fn collect_skunkbat_metrics(
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
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let quarantined = result
                    .get("connections_quarantined")
                    .and_then(Value::as_u64)
                    .unwrap_or(0);
                let alerts = result
                    .get("alerts_sent")
                    .and_then(Value::as_u64)
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
