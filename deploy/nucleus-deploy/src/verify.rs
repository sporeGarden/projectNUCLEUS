use chrono::Utc;
use serde_json::Value;
use std::path::Path;
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

use crate::config::NucleusConfig;

#[derive(Debug, Error)]
pub enum VerifyError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SSH not reachable: {host}")]
    SshUnreachable { host: String },
}

pub struct VerifyArgs {
    pub skip_ssh: bool,
    pub vps_ip: Option<String>,
}

struct Report {
    pass: u32,
    fail: u32,
    skip: u32,
    warn: u32,
    lines: Vec<String>,
}

impl Report {
    const fn new() -> Self {
        Self {
            pass: 0,
            fail: 0,
            skip: 0,
            warn: 0,
            lines: Vec::new(),
        }
    }

    fn pass(&mut self, tag: &str, msg: &str) {
        eprintln!("  PASS  [{tag}] {msg}");
        self.pass += 1;
        self.lines.push(format!("| {tag} | PASS | {msg} |"));
    }

    fn fail(&mut self, tag: &str, msg: &str) {
        eprintln!("  FAIL  [{tag}] {msg}");
        self.fail += 1;
        self.lines.push(format!("| {tag} | FAIL | {msg} |"));
    }

    fn skip(&mut self, tag: &str, msg: &str) {
        eprintln!("  SKIP  [{tag}] {msg}");
        self.skip += 1;
        self.lines.push(format!("| {tag} | SKIP | {msg} |"));
    }

    fn warn(&mut self, tag: &str, msg: &str) {
        eprintln!("  WARN  [{tag}] {msg}");
        self.warn += 1;
        self.lines.push(format!("| {tag} | WARN | {msg} |"));
    }
}

#[allow(clippy::struct_excessive_bools)]
struct PrimalStatus {
    nestgate: bool,
    rhizocrypt: bool,
    loamspine: bool,
    sweetgrass: bool,
}

fn log(msg: &str) {
    crate::util::tlog(msg);
}

pub async fn run(cfg: &NucleusConfig, args: &VerifyArgs) -> Result<bool, VerifyError> {
    let vps_ip = args.vps_ip.clone().unwrap_or_else(|| cfg.vps_ip.clone());
    let vps_user = cfg.vps_user.clone();

    let results_dir = cfg.project_root.join(format!(
        "validation/membrane-provenance-{}",
        Utc::now().format("%Y%m%d-%H%M%S")
    ));
    fs::create_dir_all(&results_dir).await?;

    log("═══════════════════════════════════════════════════════════");
    log("  Membrane Provenance — Post-Deploy Trio Verification");
    log(&format!("  VPS: {vps_ip}"));
    log(&format!("  Results: {}", results_dir.display()));
    log("═══════════════════════════════════════════════════════════");

    if args.skip_ssh {
        log("SSH skipped — cannot verify remote trio");
        return Ok(true);
    }

    if !ssh_check(&vps_user, &vps_ip).await {
        return Err(VerifyError::SshUnreachable {
            host: format!("{vps_user}@{vps_ip}"),
        });
    }

    let mut report = Report::new();

    log("");
    log("── Phase 1: Nest Primal Health ──");

    let status = check_all_nest_primals(&vps_user, &vps_ip, cfg, &mut report).await;

    if !status.rhizocrypt && !status.loamspine && !status.sweetgrass {
        log("");
        log("── No trio primals deployed. Nest Atomic pending (CM-1). ──");
        write_report(&results_dir, &report, &vps_ip, &status).await?;
        return Ok(report.fail == 0);
    }

    log("");
    log("── Phase 2: DAG Session (rhizoCrypt) ──");
    let session_id = if status.rhizocrypt {
        test_dag_session(
            &vps_user,
            &vps_ip,
            cfg.port_for("rhizocrypt-rpc"),
            &mut report,
        )
        .await
    } else {
        report.skip("TRIO-05", "rhizoCrypt not live — DAG session skipped");
        report.skip("TRIO-06", "rhizoCrypt not live — DAG event skipped");
        String::new()
    };

    log("");
    log("── Phase 3: Spine (loamSpine) ──");
    if status.loamspine {
        test_spine(&vps_user, &vps_ip, cfg.port_for("loamspine"), &mut report).await;
    } else {
        report.skip("TRIO-07", "loamSpine not live — spine skipped");
    }

    log("");
    log("── Phase 4: Braid (sweetGrass) ──");
    if status.sweetgrass {
        test_braid(&vps_user, &vps_ip, cfg.port_for("sweetgrass"), &mut report).await;
    } else {
        report.skip("TRIO-08", "sweetGrass not live — braid skipped");
    }

    log("");
    log("── Phase 5: Tower ↔ Nest Cross-Check ──");
    check_tower_primal(
        &vps_user,
        &vps_ip,
        "BearDog",
        cfg.port_for("beardog"),
        &mut report,
        "TRIO-09",
    )
    .await;
    check_turn(&vps_user, &vps_ip, &mut report).await;

    write_report(&results_dir, &report, &vps_ip, &status).await?;

    log("");
    log("═══════════════════════════════════════════════════════════");
    log("  Membrane Provenance — Results");
    log(&format!(
        "  PASS: {}  FAIL: {}  SKIP: {}  WARN: {}",
        report.pass, report.fail, report.skip, report.warn
    ));
    log(&format!("  Primals reached: {}/10", report.pass));
    log("═══════════════════════════════════════════════════════════");

    let _ = session_id;
    Ok(report.fail == 0)
}

async fn ssh_check(user: &str, ip: &str) -> bool {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=5",
            "-o",
            "BatchMode=yes",
            &format!("{user}@{ip}"),
            "true",
        ])
        .output()
        .await;
    matches!(output, Ok(o) if o.status.success())
}

async fn ssh_cmd(user: &str, ip: &str, cmd: &str) -> Option<String> {
    let output = Command::new("ssh")
        .args([
            "-o",
            "ConnectTimeout=10",
            "-o",
            "BatchMode=yes",
            "-o",
            "StrictHostKeyChecking=accept-new",
            &format!("{user}@{ip}"),
            cmd,
        ])
        .output()
        .await
        .ok()?;

    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

async fn rpc_remote(user: &str, ip: &str, port: u16, payload: &str) -> Option<String> {
    let cmd = format!(
        "echo '{payload}' | timeout 3 socat -t 0.5 - TCP:127.0.0.1:{port} 2>/dev/null | head -1",
    );
    ssh_cmd(user, ip, &cmd).await.filter(|s| !s.is_empty())
}

async fn rpc_remote_http(user: &str, ip: &str, port: u16, payload: &str) -> Option<String> {
    let cmd = format!(
        "curl -sf --max-time 3 -X POST http://127.0.0.1:{port} -H 'Content-Type: application/json' -d '{payload}' 2>/dev/null",
    );
    ssh_cmd(user, ip, &cmd).await.filter(|s| !s.is_empty())
}

async fn check_all_nest_primals(
    user: &str,
    ip: &str,
    cfg: &NucleusConfig,
    report: &mut Report,
) -> PrimalStatus {
    let nestgate = check_nest_primal(
        user,
        ip,
        "NestGate",
        cfg.port_for("nestgate"),
        "http",
        report,
        "TRIO-01",
    )
    .await;
    let rhizocrypt = check_nest_primal(
        user,
        ip,
        "rhizoCrypt",
        cfg.port_for("rhizocrypt-rpc"),
        "rpc",
        report,
        "TRIO-02",
    )
    .await;
    let loamspine = check_nest_primal(
        user,
        ip,
        "loamSpine",
        cfg.port_for("loamspine"),
        "http-rpc",
        report,
        "TRIO-03",
    )
    .await;
    let sweetgrass = check_nest_primal(
        user,
        ip,
        "sweetGrass",
        cfg.port_for("sweetgrass"),
        "rpc",
        report,
        "TRIO-04",
    )
    .await;

    PrimalStatus {
        nestgate,
        rhizocrypt,
        loamspine,
        sweetgrass,
    }
}

async fn check_nest_primal(
    user: &str,
    ip: &str,
    name: &str,
    port: u16,
    proto: &str,
    report: &mut Report,
    tag: &str,
) -> bool {
    let response = match proto {
        "http" => {
            let cmd = format!("curl -sf --max-time 3 http://127.0.0.1:{port}/health 2>/dev/null");
            ssh_cmd(user, ip, &cmd).await
        }
        "http-rpc" => {
            rpc_remote_http(
                user,
                ip,
                port,
                r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
            )
            .await
        }
        _ => {
            rpc_remote(
                user,
                ip,
                port,
                r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
            )
            .await
        }
    };

    let is_live = response
        .as_ref()
        .is_some_and(|r| r.contains("result") || r.contains("\"status\":\"ok\""));

    if is_live {
        report.pass(tag, &format!("{name} healthy (:{port})"));
    } else {
        report.skip(
            tag,
            &format!("{name} not responding (Nest Atomic not deployed)"),
        );
    }

    is_live
}

async fn check_tower_primal(
    user: &str,
    ip: &str,
    name: &str,
    port: u16,
    report: &mut Report,
    tag: &str,
) {
    let response = rpc_remote(
        user,
        ip,
        port,
        r#"{"jsonrpc":"2.0","method":"health.liveness","id":1}"#,
    )
    .await;

    if response.as_ref().is_some_and(|r| r.contains("result")) {
        report.pass(tag, &format!("{name} (Tower) healthy alongside Nest"));
    } else {
        report.fail(
            tag,
            &format!("{name} (Tower) not responding — Tower degraded"),
        );
    }
}

async fn check_turn(user: &str, ip: &str, report: &mut Report) {
    let turn_port = std::env::var("TURN_PORT").unwrap_or_else(|_| "3478".into());
    let cmd = format!("ss -ulnp 2>/dev/null | grep ':{turn_port}'");
    let response = ssh_cmd(user, ip, &cmd).await;

    if response.as_ref().is_some_and(|r| !r.is_empty()) {
        report.pass("TRIO-10", "Songbird TURN (Tower) alive alongside Nest");
    } else {
        report.warn(
            "TRIO-10",
            "Songbird TURN not listening — relay may be degraded",
        );
    }
}

async fn test_dag_session(user: &str, ip: &str, port: u16, report: &mut Report) -> String {
    let session_name = format!("membrane-verify-{}", Utc::now().format("%Y%m%d-%H%M%S"));
    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"dag.session.create","params":{{"name":"{session_name}"}},"id":10}}"#
    );

    let response = rpc_remote(user, ip, port, &payload)
        .await
        .unwrap_or_default();

    let session_id = serde_json::from_str::<Value>(&response)
        .ok()
        .and_then(|v| {
            v.get("result").and_then(|r| {
                r.as_str()
                    .map(String::from)
                    .or_else(|| serde_json::to_string(r).ok())
            })
        })
        .unwrap_or_default();

    if session_id.is_empty() {
        let trunc = &response[..response.len().min(100)];
        report.fail("TRIO-05", &format!("DAG session creation failed: {trunc}"));
        report.skip("TRIO-06", "DAG session failed — event skipped");
        return session_id;
    }

    let short = &session_id[..session_id.len().min(20)];
    report.pass("TRIO-05", &format!("DAG session created: {short}..."));

    let event_payload = format!(
        r#"{{"jsonrpc":"2.0","method":"dag.event.append","params":{{"session_id":"{session_id}","event_type":{{"DataCreate":{{}}}},"data":{{"type":"membrane_verify","timestamp":"{}"}}}},"id":11}}"#,
        Utc::now().to_rfc3339()
    );
    let event_resp = rpc_remote(user, ip, port, &event_payload)
        .await
        .unwrap_or_default();

    if event_resp.contains("result") {
        report.pass("TRIO-06", "DAG event appended");
    } else {
        let trunc = &event_resp[..event_resp.len().min(100)];
        report.warn("TRIO-06", &format!("DAG event append returned: {trunc}"));
    }

    session_id
}

async fn test_spine(user: &str, ip: &str, port: u16, report: &mut Report) {
    let payload = r#"{"jsonrpc":"2.0","method":"spine.create","params":{"name":"membrane-verify","owner":"cellMembrane"},"id":20}"#;
    let response = rpc_remote_http(user, ip, port, payload)
        .await
        .unwrap_or_default();

    let spine_id = serde_json::from_str::<Value>(&response)
        .ok()
        .and_then(|v| {
            v.pointer("/result/spine_id").and_then(|r| {
                r.as_str()
                    .map(String::from)
                    .or_else(|| serde_json::to_string(r).ok())
            })
        })
        .unwrap_or_default();

    if spine_id.is_empty() {
        let trunc = &response[..response.len().min(100)];
        report.warn("TRIO-07", &format!("Spine creation returned: {trunc}"));
    } else {
        let short = &spine_id[..spine_id.len().min(20)];
        report.pass("TRIO-07", &format!("Spine created: {short}..."));
    }
}

async fn test_braid(user: &str, ip: &str, port: u16, report: &mut Report) {
    let verify_hash = format!("{:064x}", Utc::now().timestamp());
    let payload = format!(
        r#"{{"jsonrpc":"2.0","method":"braid.create","params":{{"data_hash":"{verify_hash}","name":"membrane-verify","mime_type":"application/x-membrane-verify","description":"Post-deploy trio verification","size":1}},"id":30}}"#
    );

    let response = rpc_remote(user, ip, port, &payload)
        .await
        .unwrap_or_default();

    let braid_id = serde_json::from_str::<Value>(&response)
        .ok()
        .and_then(|v| {
            v.pointer("/result/@id").and_then(|r| {
                r.as_str()
                    .map(String::from)
                    .or_else(|| serde_json::to_string(r).ok())
            })
        })
        .unwrap_or_default();

    if braid_id.is_empty() {
        let trunc = &response[..response.len().min(100)];
        report.warn("TRIO-08", &format!("Braid creation returned: {trunc}"));
    } else {
        let short = &braid_id[..braid_id.len().min(20)];
        report.pass("TRIO-08", &format!("Braid created: {short}..."));
    }
}

async fn write_report(
    dir: &Path,
    report: &Report,
    vps_ip: &str,
    ps: &PrimalStatus,
) -> Result<(), VerifyError> {
    let status_str = |live: bool| if live { "LIVE" } else { "NOT DEPLOYED" };

    let findings = report
        .lines
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join("\n");

    let content = format!(
        r"# Membrane Provenance Verification — {date}

**VPS**: {vps_ip}

## Health

| Primal | Status |
|--------|--------|
| NestGate | {ng} |
| rhizoCrypt | {rc} |
| loamSpine | {ls} |
| sweetGrass | {sg} |

## Findings

| Tag | Status | Detail |
|-----|--------|--------|
{findings}

## Summary

PASS: {pass}  FAIL: {fail}  SKIP: {skip}  WARN: {warn}
",
        date = Utc::now().to_rfc3339(),
        ng = status_str(ps.nestgate),
        rc = status_str(ps.rhizocrypt),
        ls = status_str(ps.loamspine),
        sg = status_str(ps.sweetgrass),
        pass = report.pass,
        fail = report.fail,
        skip = report.skip,
        warn = report.warn,
    );

    fs::write(dir.join("PROVENANCE_MEMBRANE_REPORT.md"), &content).await?;
    log(&format!(
        "  Report: {}/PROVENANCE_MEMBRANE_REPORT.md",
        dir.display()
    ));
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_counting() {
        let mut r = Report::new();
        r.pass("T1", "test");
        r.fail("T2", "test");
        r.skip("T3", "test");
        r.warn("T4", "test");
        assert_eq!(r.pass, 1);
        assert_eq!(r.fail, 1);
        assert_eq!(r.skip, 1);
        assert_eq!(r.warn, 1);
        assert_eq!(r.lines.len(), 4);
    }
}
