//! Provenance trio integration for spore workloads
//! (`rhizoCrypt` → `NestGate` → `sweetGrass`).

use chrono::Local;
use serde_json::{json, Value};
use std::path::Path;
use tokio::fs;

use crate::config::NucleusConfig;
use crate::rpc;
use crate::util::tlog as log;

fn extract_hex(v: &Value) -> String {
    crate::util::value_to_hex(v)
}

async fn blake3_hash(path: &Path) -> Option<String> {
    crate::util::blake3_hash(path).await
}

pub(super) async fn capture_provenance(
    cfg: &NucleusConfig,
    workload_name: &str,
    outputs_dir: &Path,
    braids_dir: &Path,
) -> (Option<String>, Option<String>) {
    let host = &cfg.bind_address;

    if !rpc::check_liveness(host, cfg.rhizocrypt_rpc_port).await {
        log("   [SKIP] Provenance — trio primals not live");
        return (None, None);
    }

    log("   Capturing provenance chain...");

    let session_name = format!(
        "spore-{workload_name}-{}",
        Local::now().format("%Y%m%d-%H%M%S")
    );

    let session_req =
        rpc::jsonrpc_request_with_params("dag.session.create", &json!({"name": &session_name}), 1);
    let session_id = match rpc::send_jsonrpc(host, cfg.rhizocrypt_rpc_port, &session_req).await {
        Ok(r) => {
            let id = r
                .result()
                .and_then(|v| {
                    v.as_str()
                        .map(String::from)
                        .or_else(|| serde_json::to_string(v).ok())
                })
                .unwrap_or_else(|| "unknown".into());
            log(&format!("   [OK] DAG session: {id}"));
            id
        }
        Err(e) => {
            log(&format!("   [WARN] DAG session failed: {e}"));
            return (None, None);
        }
    };

    if let Ok(mut entries) = fs::read_dir(outputs_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.is_file() {
                if let Some(hash) = blake3_hash(&path).await {
                    let key = format!(
                        "spore:{workload_name}:{}",
                        path.file_name()
                            .map_or("output", |n| n.to_str().unwrap_or("output"))
                    );
                    let store_req = rpc::jsonrpc_request_with_params(
                        "storage.store",
                        &json!({"key": &key, "value": format!("blake3:{hash}")}),
                        10,
                    );
                    let _ = rpc::send_jsonrpc(host, cfg.nestgate_port, &store_req).await;
                }
            }
        }
    }

    let merkle_req = rpc::jsonrpc_request_with_params(
        "dag.merkle.root",
        &json!({"session_id": &session_id}),
        900,
    );
    let merkle_hex = rpc::send_jsonrpc(host, cfg.rhizocrypt_rpc_port, &merkle_req)
        .await
        .map_or_else(
            |_| "0".repeat(64),
            |r| {
                let v = r.result().cloned().unwrap_or(Value::Null);
                extract_hex(&v)
            },
        );

    let braid_req = rpc::jsonrpc_request_with_params(
        "braid.create",
        &json!({
            "data_hash": &merkle_hex,
            "name": &session_name,
            "mime_type": "application/x-pseudospore-provenance",
            "description": format!("pseudoSpore provenance for {workload_name}"),
            "size": 1,
        }),
        902,
    );

    let braid_id = match rpc::send_jsonrpc(host, cfg.sweetgrass_port, &braid_req).await {
        Ok(r) => {
            let result = r.result().cloned().unwrap_or(Value::Null);
            let id = result
                .get("@id")
                .and_then(Value::as_str)
                .unwrap_or("unknown")
                .to_string();
            log(&format!("   [OK] Braid: {id}"));

            let braid_json = serde_json::to_string_pretty(&result).unwrap_or_default();
            let braid_file = braids_dir.join(format!("{workload_name}_braid.json"));
            let _ = fs::write(&braid_file, &braid_json).await;

            id
        }
        Err(e) => {
            log(&format!("   [WARN] Braid creation failed: {e}"));
            "unknown".into()
        }
    };

    (Some(session_id), Some(braid_id))
}

pub(super) async fn inject_provenance(
    spore_dir: &Path,
    session_id: &str,
    braid_id: Option<&str>,
    spring: &str,
) {
    let ferment_path = spore_dir.join("provenance/ferment_transcript.json");
    if !ferment_path.exists() {
        return;
    }

    let transcript = json!({
        "dataset_id": session_id,
        "spring": spring,
        "braid_id": braid_id.unwrap_or("pending"),
        "dag_session_id": session_id,
        "pipeline": "nucleus-deploy spore",
        "timestamp": Local::now().to_rfc3339(),
        "computation": {
            "executor": "toadStool",
            "provenance": "rhizoCrypt → loamSpine → sweetGrass"
        }
    });

    let content = serde_json::to_string_pretty(&transcript).unwrap_or_default();
    let _ = fs::write(&ferment_path, &content).await;
}
