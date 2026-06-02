mod manifest;

use chrono::Local;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

use crate::config::NucleusConfig;
use crate::rpc;

#[derive(Debug, Error)]
pub enum ProvenanceError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("health check failed: {count} required primal(s) unreachable")]
    HealthCheckFailed { count: u32 },

    #[error("DAG session creation failed: {0}")]
    DagSessionFailed(String),

    #[error("spine creation failed: {0}")]
    SpineCreationFailed(String),

    #[error("results directory: {0}")]
    ResultsDir(std::io::Error),
}

pub struct ProvenanceArgs {
    pub workloads_dir: Option<PathBuf>,
    pub results_dir: Option<PathBuf>,
}

struct ProvenanceState {
    session_id: String,
    spine_id: String,
    event_idx: u64,
    artifact_hashes: Vec<ArtifactRecord>,
    workload_results: Vec<WorkloadRecord>,
}

struct ArtifactRecord {
    key: String,
    hash: String,
    size: u64,
}

struct WorkloadRecord {
    name: String,
    checks: String,
    duration_ms: u64,
    status: String,
    hash_prefix: String,
}

fn log(msg: &str) {
    crate::util::tlog(msg);
}

pub async fn run(cfg: &NucleusConfig, args: &ProvenanceArgs) -> Result<(), ProvenanceError> {
    let workloads_dir = args
        .workloads_dir
        .clone()
        .unwrap_or_else(|| cfg.project_root.join("workloads/wetspring"));

    let results_dir = args.results_dir.clone().unwrap_or_else(|| {
        cfg.project_root.join("validation").join(format!(
            "provenance-run-{}",
            Local::now().format("%Y%m%d-%H%M%S")
        ))
    });

    fs::create_dir_all(&results_dir)
        .await
        .map_err(ProvenanceError::ResultsDir)?;

    let host = &cfg.bind_address;

    log("═══════════════════════════════════════════════════════════");
    log("  Provenance Pipeline — Nest Atomic Full Rigor");
    log(&format!("  Results: {}", results_dir.display()));
    log("═══════════════════════════════════════════════════════════");

    // Phase 1: Health checks
    phase_health_checks(cfg, host).await?;

    // Phase 2: Create DAG session
    let session_name = format!("abg-pipeline-{}", Local::now().format("%Y%m%d-%H%M%S"));
    let session_id =
        phase_create_dag_session(host, cfg.port_for("rhizocrypt"), &session_name).await?;

    // Phase 3: Create loamSpine spine
    let spine_id = phase_create_spine(host, cfg.port_for("loamspine"), &session_name).await?;

    let mut state = ProvenanceState {
        session_id,
        spine_id,
        event_idx: 0,
        artifact_hashes: Vec::new(),
        workload_results: Vec::new(),
    };

    // Phase 4: Register data artifacts
    phase_register_artifacts(cfg, host, &mut state).await;

    // Phase 5: Execute workloads with provenance wrapping
    phase_execute_workloads(cfg, host, &workloads_dir, &results_dir, &mut state).await;

    // Phase 6: Dehydrate → Merkle root
    let merkle_hex = phase_merkle_root(host, cfg.port_for("rhizocrypt"), &state).await;

    // Phase 7: Commit to loamSpine
    let commit_index =
        phase_loamspine_commit(host, cfg.port_for("loamspine"), &state, &merkle_hex).await;

    // Phase 8: Create sweetGrass braid
    let (braid_id, braid_witness) = phase_sweetgrass_braid(
        host,
        cfg.port_for("sweetgrass"),
        &state,
        &session_name,
        &merkle_hex,
    )
    .await;

    // Phase 9: Write manifest
    phase_write_manifest(
        &results_dir,
        &session_name,
        &state,
        &merkle_hex,
        &commit_index,
        &braid_id,
        &braid_witness,
    )
    .await?;

    log("");
    log("═══════════════════════════════════════════════════════════");
    log("  Provenance Pipeline Complete");
    log(&format!("  Session:     {}", state.session_id));
    log(&format!("  Events:      {}", state.event_idx));
    log(&format!("  Merkle Root: {merkle_hex}"));
    log(&format!("  Spine:       {}", state.spine_id));
    log(&format!("  Braid:       {braid_id}"));
    log(&format!("  Results:     {}", results_dir.display()));
    log("═══════════════════════════════════════════════════════════");

    Ok(())
}

// ── Phase 1: Health Checks ───────────────────────────────────────────────

async fn phase_health_checks(cfg: &NucleusConfig, host: &str) -> Result<(), ProvenanceError> {
    log("");
    log("── Phase 1: Health Checks ──");

    let required: &[(&str, u16)] = &[
        ("BearDog", cfg.port_for("beardog")),
        ("ToadStool", cfg.port_for("toadstool")),
        ("NestGate", cfg.port_for("nestgate")),
        ("rhizoCrypt", cfg.port_for("rhizocrypt")),
        ("loamSpine", cfg.port_for("loamspine")),
        ("sweetGrass", cfg.port_for("sweetgrass")),
    ];

    let optional: &[(&str, u16)] = &[("Songbird", cfg.port_for("songbird"))];

    let mut failures = 0u32;

    for &(name, port) in required {
        if check_primal_health(host, name, port).await {
            log(&format!("  [OK] {name} (TCP {port}) healthy"));
        } else {
            log(&format!("  [FAIL] {name} (TCP {port}) not responding"));
            failures += 1;
        }
    }

    for &(name, port) in optional {
        if !check_primal_health(host, name, port).await {
            log(&format!(
                "  [SKIP] {name} — optional for provenance pipeline"
            ));
        }
    }

    if failures > 0 {
        log(&format!(
            "  {failures} required primal(s) failed health check."
        ));
        return Err(ProvenanceError::HealthCheckFailed { count: failures });
    }

    Ok(())
}

async fn check_primal_health(host: &str, name: &str, port: u16) -> bool {
    if name == "Songbird" {
        let url = format!("http://{host}:{port}/health");
        let output = Command::new("curl")
            .args(["-sf", "--max-time", "3", &url])
            .output()
            .await;
        return matches!(output, Ok(o) if String::from_utf8_lossy(&o.stdout).trim() == "OK");
    }

    let probe_port = if name == "rhizoCrypt" { port + 1 } else { port };

    if name == "loamSpine" {
        let url = format!("http://{host}:{port}");
        let payload = r#"{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}"#;
        let output = Command::new("curl")
            .args([
                "-sf",
                "--max-time",
                "3",
                &url,
                "-X",
                "POST",
                "-H",
                "Content-Type: application/json",
                "-d",
                payload,
            ])
            .output()
            .await;
        return matches!(output, Ok(o) if String::from_utf8_lossy(&o.stdout).contains("\"result\""));
    }

    rpc::check_liveness(host, probe_port).await
}

// ── Phase 2: DAG Session ─────────────────────────────────────────────────

async fn phase_create_dag_session(
    host: &str,
    rhizocrypt_port: u16,
    session_name: &str,
) -> Result<String, ProvenanceError> {
    log("");
    log("── Phase 2: Create DAG Session ──");

    let jsonrpc_port = rhizocrypt_port + 1;
    let req =
        rpc::jsonrpc_request_with_params("dag.session.create", &json!({"name": session_name}), 1);

    let resp = rpc::send_jsonrpc(host, jsonrpc_port, &req)
        .await
        .map_err(|e| ProvenanceError::DagSessionFailed(e.to_string()))?;

    let session_id = resp
        .result()
        .and_then(|r| {
            r.as_str()
                .map(String::from)
                .or_else(|| serde_json::to_string(r).ok())
        })
        .ok_or_else(|| ProvenanceError::DagSessionFailed(resp.raw.clone()))?;

    log(&format!("  [OK] Session: {session_id} ({session_name})"));
    Ok(session_id)
}

// ── Phase 3: LoamSpine Spine ─────────────────────────────────────────────

async fn phase_create_spine(
    host: &str,
    loamspine_port: u16,
    session_name: &str,
) -> Result<String, ProvenanceError> {
    log("");
    log("── Phase 3: Create LoamSpine Spine ──");

    let payload = json!({
        "jsonrpc": "2.0",
        "method": "spine.create",
        "params": {"name": session_name, "owner": "ecoPrimal"},
        "id": 1,
    })
    .to_string();

    let url = format!("http://{host}:{loamspine_port}");
    let output = Command::new("curl")
        .args([
            "-sf",
            &url,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload,
        ])
        .output()
        .await
        .map_err(|e| ProvenanceError::SpineCreationFailed(e.to_string()))?;

    let body = String::from_utf8_lossy(&output.stdout);
    let parsed: Value = serde_json::from_str(body.trim())
        .map_err(|e| ProvenanceError::SpineCreationFailed(e.to_string()))?;

    let spine_id = parsed
        .pointer("/result/spine_id")
        .and_then(|v| {
            v.as_str()
                .map(String::from)
                .or_else(|| serde_json::to_string(v).ok())
        })
        .ok_or_else(|| ProvenanceError::SpineCreationFailed(body.to_string()))?;

    log(&format!("  [OK] Spine: {spine_id}"));
    Ok(spine_id)
}

// ── Phase 4: Register Artifacts ──────────────────────────────────────────

async fn phase_register_artifacts(cfg: &NucleusConfig, host: &str, state: &mut ProvenanceState) {
    log("");
    log("── Phase 4: Register Data Artifacts ──");

    let wetspring_dir = cfg.ecoprimals_root.join("springs/wetSpring");

    let artifacts: &[(&str, &str, &str, &str)] = &[
        (
            "data/paper_proxy/nannochloropsis_16s/SRR7760408/SRR7760408_1.fastq.gz",
            "ncbi:SRR7760408:R1",
            "ncbi_fastq",
            "SRR7760408",
        ),
        (
            "data/paper_proxy/nannochloropsis_16s/SRR7760408/SRR7760408_2.fastq.gz",
            "ncbi:SRR7760408:R2",
            "ncbi_fastq",
            "SRR7760408",
        ),
        (
            "data/paper_proxy/nannochloropsis_pilots/SRR5534045/SRR5534045_1.fastq.gz",
            "ncbi:SRR5534045:R1",
            "ncbi_fastq",
            "SRR5534045",
        ),
        (
            "data/paper_proxy/nannochloropsis_pilots/SRR5534045/SRR5534045_2.fastq.gz",
            "ncbi:SRR5534045:R2",
            "ncbi_fastq",
            "SRR5534045",
        ),
    ];

    for &(path_suffix, key, artifact_type, accession) in artifacts {
        let filepath = wetspring_dir.join(path_suffix);
        register_artifact(host, cfg, state, &filepath, key, artifact_type, accession).await;
    }
}

async fn register_artifact(
    host: &str,
    cfg: &NucleusConfig,
    state: &mut ProvenanceState,
    filepath: &Path,
    artifact_key: &str,
    _artifact_type: &str,
    _accession: &str,
) {
    if !filepath.exists() {
        log(&format!(
            "  [SKIP] {artifact_key} — file not found: {}",
            filepath.display()
        ));
        return;
    }

    let Some(hash) = blake3_hash(filepath).await else {
        log(&format!("  [SKIP] {artifact_key} — b3sum failed"));
        return;
    };

    let Ok(meta) = fs::metadata(filepath).await else {
        log(&format!("  [SKIP] {artifact_key} — stat failed"));
        return;
    };
    let size = meta.len();

    let store_req = rpc::jsonrpc_request_with_params(
        "storage.store",
        &json!({"key": artifact_key, "value": format!("blake3:{hash} size:{size}")}),
        state.event_idx + 100,
    );
    let _ = rpc::send_jsonrpc(host, cfg.port_for("nestgate"), &store_req).await;

    state.event_idx += 1;
    state.artifact_hashes.push(ArtifactRecord {
        key: artifact_key.to_string(),
        hash: hash.clone(),
        size,
    });

    let hash_prefix = &hash[..hash.len().min(16)];
    log(&format!(
        "  [OK] {artifact_key} → blake3:{hash_prefix}… ({size}B)"
    ));
}

async fn blake3_hash(path: &Path) -> Option<String> {
    crate::util::blake3_hash(path).await
}

// ── Phase 5: Execute Workloads ───────────────────────────────────────────

async fn phase_execute_workloads(
    cfg: &NucleusConfig,
    host: &str,
    workloads_dir: &Path,
    results_dir: &Path,
    state: &mut ProvenanceState,
) {
    log("");
    log("── Phase 5: Execute Workloads with Provenance ──");

    let Ok(mut entries) = fs::read_dir(workloads_dir).await else {
        log(&format!(
            "  [WARN] No workload TOMLs found in {}",
            workloads_dir.display()
        ));
        return;
    };

    let mut toml_paths = Vec::new();
    while let Ok(Some(entry)) = entries.next_entry().await {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "toml") {
            toml_paths.push(path);
        }
    }
    toml_paths.sort();

    if toml_paths.is_empty() {
        log(&format!(
            "  [WARN] No workload TOMLs found in {}",
            workloads_dir.display()
        ));
        return;
    }

    for toml_path in &toml_paths {
        execute_with_provenance(cfg, host, toml_path, results_dir, state).await;
    }
}

async fn execute_with_provenance(
    cfg: &NucleusConfig,
    host: &str,
    toml_path: &Path,
    results_dir: &Path,
    state: &mut ProvenanceState,
) {
    let workload_name = toml_path
        .file_stem()
        .map_or_else(|| "unknown".into(), |s| s.to_string_lossy().to_string());

    log(&format!("  [{workload_name}] Executing..."));

    let toadstool = std::env::var("TOADSTOOL").unwrap_or_else(|_| {
        cfg.plasmidbin_dir
            .join("primals/toadstool")
            .display()
            .to_string()
    });

    let start = std::time::Instant::now();
    let output_file = results_dir.join(format!("{workload_name}.stdout"));

    let result = Command::new(&toadstool)
        .arg("execute")
        .arg(toml_path)
        .output()
        .await;

    #[allow(clippy::cast_possible_truncation)]
    let elapsed_ms = start.elapsed().as_millis() as u64;

    if let Ok(o) = result {
        let stdout = String::from_utf8_lossy(&o.stdout);
        let _ = fs::write(&output_file, stdout.as_bytes()).await;

        let check_count = stdout.matches("[OK]").count();
        let fail_count = stdout.matches("[FAIL]").count();

        let exit_code = if o.status.success() && !stdout.contains("Status:") {
            0
        } else if stdout.contains("Status: Failed") {
            1
        } else {
            i32::from(!o.status.success())
        };

        let output_hash = blake3_hash(&output_file).await.unwrap_or_default();

        let _ = rpc::send_jsonrpc(
            host,
            cfg.port_for("nestgate"),
            &rpc::jsonrpc_request_with_params(
                "storage.store",
                &json!({
                    "key": format!("workload:{workload_name}:output"),
                    "value": format!("blake3:{output_hash}")
                }),
                state.event_idx + 500,
            ),
        )
        .await;

        state.event_idx += 1;

        let status = if exit_code != 0 {
            "FAIL"
        } else if check_count == 0 && fail_count == 0 {
            "RUN"
        } else {
            "PASS"
        };

        let hash_prefix = if output_hash.len() >= 16 {
            &output_hash[..16]
        } else {
            &output_hash
        };

        state.workload_results.push(WorkloadRecord {
            name: workload_name.clone(),
            checks: format!("{check_count}/{}", check_count + fail_count),
            duration_ms: elapsed_ms,
            status: status.to_string(),
            hash_prefix: format!("blake3:{hash_prefix}"),
        });

        log(&format!(
            "  [{workload_name}] {status} — {check_count} checks, {elapsed_ms}ms, blake3:{hash_prefix}…"
        ));
    } else {
        log(&format!("  [{workload_name}] EXECUTION ERROR"));
        state.event_idx += 1;
        state.workload_results.push(WorkloadRecord {
            name: workload_name,
            checks: "ERROR".into(),
            duration_ms: 0,
            status: "ERROR".into(),
            hash_prefix: "-".into(),
        });
    }
}

// ── Phase 6: Merkle Root ─────────────────────────────────────────────────

async fn phase_merkle_root(host: &str, rhizocrypt_port: u16, state: &ProvenanceState) -> String {
    log("");
    log("── Phase 6: Dehydrate → Sign → Commit → Braid ──");

    let jsonrpc_port = rhizocrypt_port + 1;
    let req = rpc::jsonrpc_request_with_params(
        "dag.merkle.root",
        &json!({"session_id": state.session_id}),
        900,
    );

    match rpc::send_jsonrpc(host, jsonrpc_port, &req).await {
        Ok(r) => {
            let result = r.result().cloned().unwrap_or(Value::Null);
            let hex = value_to_hex(&result);
            log(&format!("  [OK] Merkle root: {hex}"));
            hex
        }
        Err(e) => {
            let fallback = format!("{:0>64}", "0");
            log(&format!("  [WARN] Merkle root unavailable: {e}"));
            fallback
        }
    }
}

fn value_to_hex(v: &Value) -> String {
    crate::util::value_to_hex(v)
}

// ── Phase 7: LoamSpine Commit ────────────────────────────────────────────

async fn phase_loamspine_commit(
    host: &str,
    loamspine_port: u16,
    state: &ProvenanceState,
    merkle_hex: &str,
) -> String {
    let merkle_bytes = hex_to_byte_array(merkle_hex);
    let payload = json!({
        "jsonrpc": "2.0",
        "method": "entry.append",
        "params": {
            "spine_id": state.spine_id,
            "entry_type": {
                "SessionCommit": {
                    "session_id": state.session_id,
                    "merkle_root": merkle_bytes,
                    "vertex_count": state.event_idx,
                    "committer": "did:primal:ecoPrimal",
                }
            },
            "committer": "did:primal:ecoPrimal",
            "data": {
                "merkle_hex": merkle_hex,
                "event_count": state.event_idx,
            }
        },
        "id": 901
    })
    .to_string();

    let url = format!("http://{host}:{loamspine_port}");
    let output = Command::new("curl")
        .args([
            "-sf",
            &url,
            "-X",
            "POST",
            "-H",
            "Content-Type: application/json",
            "-d",
            &payload,
        ])
        .output()
        .await;

    if let Ok(o) = output {
        let body = String::from_utf8_lossy(&o.stdout);
        if let Ok(parsed) = serde_json::from_str::<Value>(body.trim()) {
            let index = parsed
                .pointer("/result/index")
                .and_then(Value::as_u64)
                .map_or_else(|| "?".into(), |i| i.to_string());
            log(&format!("  [OK] LoamSpine commit: index={index}"));
            return index;
        }
    }

    log("  [WARN] LoamSpine commit: could not confirm");
    "?".into()
}

fn hex_to_byte_array(hex: &str) -> Vec<u8> {
    crate::util::hex_to_bytes(hex)
}

// ── Phase 8: SweetGrass Braid ────────────────────────────────────────────

async fn phase_sweetgrass_braid(
    host: &str,
    sweetgrass_port: u16,
    state: &ProvenanceState,
    session_name: &str,
    merkle_hex: &str,
) -> (String, String) {
    let req = rpc::jsonrpc_request_with_params(
        "braid.create",
        &json!({
            "data_hash": merkle_hex,
            "name": session_name,
            "mime_type": "application/x-provenance-pipeline",
            "description": format!(
                "ABG Full Pipeline — provenance braid for {} events across wetSpring validators",
                state.event_idx
            ),
            "size": state.event_idx,
        }),
        902,
    );

    match rpc::send_jsonrpc(host, sweetgrass_port, &req).await {
        Ok(r) => {
            let result = r.result().cloned().unwrap_or(Value::Null);
            let braid_id = result
                .get("@id")
                .and_then(Value::as_str)
                .unwrap_or("?")
                .to_string();
            let witness = result.get("witness").map_or_else(
                || "?".into(),
                |w| {
                    let algo = w.get("algorithm").and_then(Value::as_str).unwrap_or("?");
                    let evidence = w.get("evidence").and_then(Value::as_str).unwrap_or("?");
                    let prefix = if evidence.len() > 32 {
                        &evidence[..32]
                    } else {
                        evidence
                    };
                    format!("{algo}: {prefix}...")
                },
            );

            log(&format!("  [OK] Braid: {braid_id}"));
            log(&format!("       Witness: {witness}"));
            (braid_id, witness)
        }
        Err(e) => {
            log(&format!("  [WARN] Braid creation failed: {e}"));
            ("?".into(), "?".into())
        }
    }
}

use manifest::phase_write_manifest;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn value_to_hex_from_array() {
        let v = json!([0, 255, 16, 1]);
        assert_eq!(value_to_hex(&v), "00ff1001");
    }

    #[test]
    fn value_to_hex_from_string() {
        let v = json!("abcdef0123456789");
        assert_eq!(value_to_hex(&v), "abcdef0123456789");
    }

    #[test]
    fn hex_to_byte_array_roundtrip() {
        let hex = "00ff1001";
        let bytes = hex_to_byte_array(hex);
        assert_eq!(bytes, vec![0, 255, 16, 1]);
    }

    #[test]
    fn hex_to_byte_array_empty() {
        assert!(hex_to_byte_array("").is_empty());
    }

    #[test]
    fn hex_to_byte_array_odd_length() {
        let bytes = hex_to_byte_array("0ff");
        assert_eq!(bytes, vec![0x0f]);
    }
}
