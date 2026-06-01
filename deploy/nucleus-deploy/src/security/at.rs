use std::time::Duration;

use super::report::SecurityReport;
use crate::config::NucleusConfig;
use crate::rpc;

pub async fn layer_at(report: &mut SecurityReport, host: &str, cfg: &NucleusConfig) {
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
