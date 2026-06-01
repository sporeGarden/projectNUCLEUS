use chrono::Local;
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs;
use tokio::process::Command;

use crate::config::NucleusConfig;
use crate::rpc;

#[derive(Debug, Error)]
pub enum SporeError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("workload TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("workload not found: {path}")]
    WorkloadNotFound { path: String },

    #[error("litho binary not found (searched: {searched})")]
    LithoNotFound { searched: String },

    #[error("toadstool binary not found (searched: {searched})")]
    ToadstoolNotFound { searched: String },

    #[error("toadstool execution failed for {workload}: {reason}")]
    ExecutionFailed { workload: String, reason: String },

    #[error("litho emit-pseudospore failed: {0}")]
    EmitFailed(String),

    #[error("output directory creation failed: {0}")]
    OutputDir(std::io::Error),
}

pub struct SporeArgs {
    pub workload: Option<PathBuf>,
    pub workloads_dir: Option<PathBuf>,
    pub output: PathBuf,
    pub skip_provenance: bool,
    pub litho_bin: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct WorkloadToml {
    metadata: WorkloadMetadata,
    #[allow(dead_code)]
    execution: WorkloadExecution,
}

#[derive(Debug, Deserialize)]
struct WorkloadMetadata {
    name: String,
    description: String,
    version: String,
    spring: Option<String>,
    #[serde(default)]
    #[allow(dead_code)]
    compute_tier: Option<String>,
}

#[derive(Debug, Deserialize)]
struct WorkloadExecution {
    #[serde(rename = "type")]
    #[allow(dead_code)]
    exec_type: String,
    #[allow(dead_code)]
    command: Option<String>,
    #[allow(dead_code)]
    working_dir: Option<String>,
}

struct SporeResult {
    workload_name: String,
    spring: String,
    version: String,
    spore_dir: PathBuf,
    checks_passed: usize,
    checks_total: usize,
    provenance_braid: Option<String>,
}

fn log(msg: &str) {
    crate::util::tlog(msg);
}

pub async fn run(cfg: &NucleusConfig, args: &SporeArgs) -> Result<(), SporeError> {
    log("═══════════════════════════════════════════════════════════");
    log("  Workload → pseudoSpore Pipeline");
    log("═══════════════════════════════════════════════════════════");

    let litho = resolve_litho(cfg, args)?;
    log(&format!("  litho binary: {}", litho.display()));

    fs::create_dir_all(&args.output)
        .await
        .map_err(SporeError::OutputDir)?;

    let toml_paths = collect_workloads(args).await?;
    if toml_paths.is_empty() {
        log("  [WARN] No workload TOML files found.");
        return Ok(());
    }

    log(&format!(
        "  {} workload(s) queued for pseudoSpore emission",
        toml_paths.len()
    ));

    let mut results = Vec::new();

    for toml_path in &toml_paths {
        match process_workload(cfg, args, &litho, toml_path).await {
            Ok(result) => {
                log(&format!(
                    "  [OK] {} → {}",
                    result.workload_name,
                    result.spore_dir.display()
                ));
                results.push(result);
            }
            Err(e) => {
                log(&format!("  [FAIL] {} — {e}", toml_path.display()));
            }
        }
    }

    log("");
    log("═══════════════════════════════════════════════════════════");
    log("  pseudoSpore Emission Summary");
    log("═══════════════════════════════════════════════════════════");

    for r in &results {
        let checks = format!("{}/{}", r.checks_passed, r.checks_total);
        let prov = r.provenance_braid.as_deref().unwrap_or("skipped");
        log(&format!(
            "  {} v{} ({}) — checks: {checks}, braid: {prov}",
            r.workload_name, r.version, r.spring
        ));
        log(&format!("    → {}", r.spore_dir.display()));
    }

    log(&format!(
        "  {}/{} workloads emitted as pseudoSpores",
        results.len(),
        toml_paths.len()
    ));

    Ok(())
}

fn resolve_litho(cfg: &NucleusConfig, args: &SporeArgs) -> Result<PathBuf, SporeError> {
    if let Some(ref bin) = args.litho_bin {
        if bin.exists() {
            return Ok(bin.clone());
        }
    }

    let candidates = [
        cfg.plasmidbin_dir.join("primals/litho"),
        cfg.ecoprimals_root
            .join("gardens/lithoSpore/target/release/litho"),
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
            .join(".local/bin/litho"),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    let which = std::process::Command::new("which").arg("litho").output();

    if let Ok(o) = which {
        if o.status.success() {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    let searched = candidates
        .iter()
        .map(|c| c.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(SporeError::LithoNotFound { searched })
}

fn resolve_toadstool(cfg: &NucleusConfig) -> Result<PathBuf, SporeError> {
    if let Ok(ts) = std::env::var("TOADSTOOL") {
        let p = PathBuf::from(&ts);
        if p.exists() {
            return Ok(p);
        }
    }

    let candidates = [
        cfg.plasmidbin_dir.join("primals/toadstool"),
        PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| "/tmp".into()))
            .join(".local/bin/toadstool"),
    ];

    for c in &candidates {
        if c.exists() {
            return Ok(c.clone());
        }
    }

    let which = std::process::Command::new("which")
        .arg("toadstool")
        .output();

    if let Ok(o) = which {
        if o.status.success() {
            let path = String::from_utf8_lossy(&o.stdout).trim().to_string();
            if !path.is_empty() {
                return Ok(PathBuf::from(path));
            }
        }
    }

    let searched = candidates
        .iter()
        .map(|c| c.display().to_string())
        .collect::<Vec<_>>()
        .join(", ");
    Err(SporeError::ToadstoolNotFound { searched })
}

async fn collect_workloads(args: &SporeArgs) -> Result<Vec<PathBuf>, SporeError> {
    if let Some(ref single) = args.workload {
        if !single.exists() {
            return Err(SporeError::WorkloadNotFound {
                path: single.display().to_string(),
            });
        }
        return Ok(vec![single.clone()]);
    }

    if let Some(ref dir) = args.workloads_dir {
        let mut paths = Vec::new();
        let Ok(mut entries) = fs::read_dir(dir).await else {
            return Ok(paths);
        };
        while let Ok(Some(entry)) = entries.next_entry().await {
            let p = entry.path();
            if p.extension().is_some_and(|e| e == "toml") {
                paths.push(p);
            }
        }
        paths.sort();
        return Ok(paths);
    }

    Ok(Vec::new())
}

async fn process_workload(
    cfg: &NucleusConfig,
    args: &SporeArgs,
    litho: &Path,
    toml_path: &Path,
) -> Result<SporeResult, SporeError> {
    let raw = fs::read_to_string(toml_path).await?;
    let workload: WorkloadToml = toml::from_str(&raw)?;

    let spring = infer_spring(&workload.metadata, toml_path);
    let name = &workload.metadata.name;
    let version = &workload.metadata.version;

    log("");
    log(&format!("── {name} ({spring}) ──"));
    log(&format!("   {}", workload.metadata.description));

    // Stage 1: Execute via toadStool (or direct execution)
    let staging_dir = args.output.join(format!("staging-{name}"));
    fs::create_dir_all(&staging_dir).await?;

    let outputs_dir = staging_dir.join("outputs");
    let configs_dir = staging_dir.join("configs");
    let braids_dir = staging_dir.join("braids");
    fs::create_dir_all(&outputs_dir).await?;
    fs::create_dir_all(&configs_dir).await?;
    fs::create_dir_all(&braids_dir).await?;

    let exec_result = execute_workload(cfg, toml_path, &outputs_dir).await?;

    // Save workload TOML as config artifact
    fs::copy(toml_path, configs_dir.join(format!("{name}.toml"))).await?;

    // Stage 2: Optional provenance capture
    let (session_id, provenance_braid) = if args.skip_provenance {
        log("   [SKIP] Provenance — trio not required");
        (None, None)
    } else {
        capture_provenance(cfg, name, &outputs_dir, &braids_dir).await
    };

    // Stage 3: Locate domain profile
    let domain_profile = find_domain_profile(cfg, &spring);

    // Stage 4: Emit pseudoSpore via litho
    let emit_ctx = EmitContext {
        litho,
        name,
        version,
        spring: &spring,
        dest: &args.output,
        captured_outputs: &outputs_dir,
        captured_configs: &configs_dir,
        captured_braids: &braids_dir,
        domain_profile: domain_profile.as_deref(),
    };
    let spore_dir = emit_pseudospore(&emit_ctx).await?;

    // Stage 5: Inject real provenance into ferment transcript (if available)
    if let Some(ref sid) = session_id {
        inject_provenance(&spore_dir, sid, provenance_braid.as_deref(), &spring).await;
    }

    // Clean up staging
    let _ = fs::remove_dir_all(&staging_dir).await;

    Ok(SporeResult {
        workload_name: name.clone(),
        spring,
        version: version.clone(),
        spore_dir,
        checks_passed: exec_result.checks_passed,
        checks_total: exec_result.checks_total,
        provenance_braid,
    })
}

struct ExecResult {
    checks_passed: usize,
    checks_total: usize,
}

async fn execute_workload(
    cfg: &NucleusConfig,
    toml_path: &Path,
    outputs_dir: &Path,
) -> Result<ExecResult, SporeError> {
    let workload_name = toml_path
        .file_stem()
        .map_or_else(|| "unknown".into(), |s| s.to_string_lossy().to_string());

    let toadstool = resolve_toadstool(cfg);

    let output = if let Ok(ref ts) = toadstool {
        log(&format!("   Executing via toadStool: {workload_name}"));
        Command::new(ts)
            .arg("execute")
            .arg(toml_path)
            .output()
            .await
    } else {
        log("   [INFO] toadStool not found — executing workload directly");
        direct_execute(cfg, toml_path).await
    };

    let output = output.map_err(|e| SporeError::ExecutionFailed {
        workload: workload_name.clone(),
        reason: e.to_string(),
    })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let combined = format!("{stdout}\n{stderr}");

    let stdout_file = outputs_dir.join(format!("{workload_name}.stdout"));
    fs::write(&stdout_file, stdout.as_bytes()).await?;

    if !stderr.is_empty() {
        let stderr_file = outputs_dir.join(format!("{workload_name}.stderr"));
        fs::write(&stderr_file, stderr.as_bytes()).await?;
    }

    let checks_passed = combined.matches("[OK]").count();
    let checks_failed = combined.matches("[FAIL]").count();
    let checks_total = checks_passed + checks_failed;

    let status = if !output.status.success() {
        "FAIL"
    } else if checks_total == 0 {
        "RUN"
    } else {
        "PASS"
    };

    log(&format!(
        "   [{status}] {checks_passed}/{checks_total} checks, exit={}",
        output.status.code().unwrap_or(-1)
    ));

    Ok(ExecResult {
        checks_passed,
        checks_total,
    })
}

async fn direct_execute(
    cfg: &NucleusConfig,
    toml_path: &Path,
) -> std::io::Result<std::process::Output> {
    let raw = fs::read_to_string(toml_path).await?;
    let workload: WorkloadToml = toml::from_str(&raw)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let Some(ref cmd_str) = workload.execution.command else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "no command in workload TOML",
        ));
    };

    let parts: Vec<&str> = cmd_str.split_whitespace().collect();
    if parts.is_empty() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "empty command",
        ));
    }

    let springs_root = std::env::var("SPRINGS_ROOT").unwrap_or_else(|_| {
        std::env::var("ECOPRIMALS_ROOT").map_or_else(
            |_| cfg.ecoprimals_root.join("springs").display().to_string(),
            |r| format!("{r}/springs"),
        )
    });

    let resolved_cmd = parts[0].replace("$SPRINGS_ROOT", &springs_root);
    let resolved_args: Vec<String> = parts[1..]
        .iter()
        .map(|a| a.replace("$SPRINGS_ROOT", &springs_root))
        .collect();

    let mut cmd = std::process::Command::new(&resolved_cmd);
    cmd.args(&resolved_args);

    if let Some(ref wd) = workload.execution.working_dir {
        let resolved_wd = wd.replace("$SPRINGS_ROOT", &springs_root);
        cmd.current_dir(&resolved_wd);
    }

    cmd.output()
}

fn infer_spring(metadata: &WorkloadMetadata, toml_path: &Path) -> String {
    if let Some(ref s) = metadata.spring {
        return s.clone();
    }

    if let Some(parent) = toml_path.parent() {
        let dir_name = parent
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();

        let spring_map = [
            ("airspring", "airSpring"),
            ("hotspring", "hotSpring"),
            ("wetspring", "wetSpring"),
            ("healthspring", "healthSpring"),
            ("neuralspring", "neuralSpring"),
            ("groundspring", "groundSpring"),
            ("ludospring", "ludoSpring"),
        ];

        for (prefix, spring) in &spring_map {
            if dir_name == *prefix {
                return (*spring).to_string();
            }
        }
    }

    let name_lower = metadata.name.to_lowercase();
    let spring_prefixes = [
        ("airspring", "airSpring"),
        ("hotspring", "hotSpring"),
        ("wetspring", "wetSpring"),
        ("healthspring", "healthSpring"),
        ("neuralspring", "neuralSpring"),
        ("groundspring", "groundSpring"),
        ("ludospring", "ludoSpring"),
    ];

    for (prefix, spring) in &spring_prefixes {
        if name_lower.starts_with(prefix) {
            return (*spring).to_string();
        }
    }

    "unknown".into()
}

fn find_domain_profile(cfg: &NucleusConfig, spring: &str) -> Option<PathBuf> {
    let springs_root = cfg.ecoprimals_root.join("springs");
    let profile = springs_root.join(spring).join("domain_profile.toml");
    if profile.exists() {
        Some(profile)
    } else {
        log(&format!(
            "   [INFO] No domain_profile.toml for {spring} at {}",
            profile.display()
        ));
        None
    }
}

struct EmitContext<'a> {
    litho: &'a Path,
    name: &'a str,
    version: &'a str,
    spring: &'a str,
    dest: &'a Path,
    captured_outputs: &'a Path,
    captured_configs: &'a Path,
    captured_braids: &'a Path,
    domain_profile: Option<&'a Path>,
}

async fn emit_pseudospore(ctx: &EmitContext<'_>) -> Result<PathBuf, SporeError> {
    log("   Emitting pseudoSpore via litho...");

    let mut cmd = Command::new(ctx.litho);
    cmd.arg("emit-pseudospore")
        .args(["--name", ctx.name])
        .args(["--version", ctx.version])
        .args(["--spring", ctx.spring])
        .args(["--output", &ctx.dest.display().to_string()])
        .args(["--outputs", &ctx.captured_outputs.display().to_string()])
        .args(["--configs", &ctx.captured_configs.display().to_string()]);

    cmd.args(["--braids", &ctx.captured_braids.display().to_string()]);

    if let Some(profile) = ctx.domain_profile {
        cmd.args(["--profile", &profile.display().to_string()]);
    }

    let output = cmd
        .output()
        .await
        .map_err(|e| SporeError::EmitFailed(format!("failed to run litho: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !output.status.success() {
        log(&format!("   litho stderr: {stderr}"));
        return Err(SporeError::EmitFailed(format!(
            "exit code {}: {stderr}",
            output.status.code().unwrap_or(-1)
        )));
    }

    if !stdout.is_empty() {
        for line in stdout.lines().take(5) {
            log(&format!("   litho: {line}"));
        }
    }

    let spore_dir_name = format!("pseudoSpore_{}_v{}", ctx.name, ctx.version);
    let spore_dir = ctx.dest.join(&spore_dir_name);

    if spore_dir.exists() {
        log(&format!("   [OK] pseudoSpore emitted: {spore_dir_name}"));
    } else {
        log(&format!(
            "   [WARN] Expected directory not found: {spore_dir_name} — scanning..."
        ));
        let mut found = None;
        if let Ok(mut entries) = fs::read_dir(ctx.dest).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let p = entry.path();
                if p.is_dir() {
                    let dirname = p
                        .file_name()
                        .map_or_else(String::new, |n| n.to_string_lossy().to_string());
                    if dirname.starts_with("pseudoSpore_") {
                        found = Some(p);
                        break;
                    }
                }
            }
        }
        if let Some(actual) = found {
            log(&format!("   [OK] Found: {}", actual.display()));
            return Ok(actual);
        }
    }

    Ok(spore_dir)
}

async fn capture_provenance(
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

    // DAG session
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

    // Hash outputs and store in NestGate
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

    // Merkle root
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

    // SweetGrass braid
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

            // Export braid to braids_dir
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

async fn inject_provenance(
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

fn extract_hex(v: &Value) -> String {
    crate::util::value_to_hex(v)
}

async fn blake3_hash(path: &Path) -> Option<String> {
    crate::util::blake3_hash(path).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn infer_spring_from_metadata() {
        let meta = WorkloadMetadata {
            name: "airspring-et0-validation".into(),
            description: String::new(),
            version: "0.1.0".into(),
            spring: Some("airSpring".into()),
            compute_tier: None,
        };
        assert_eq!(
            infer_spring(&meta, Path::new("workloads/airspring/test.toml")),
            "airSpring"
        );
    }

    #[test]
    fn infer_spring_from_directory() {
        let meta = WorkloadMetadata {
            name: "test".into(),
            description: String::new(),
            version: "0.1.0".into(),
            spring: None,
            compute_tier: None,
        };
        assert_eq!(
            infer_spring(&meta, Path::new("workloads/wetspring/test.toml")),
            "wetSpring"
        );
    }

    #[test]
    fn infer_spring_from_name_prefix() {
        let meta = WorkloadMetadata {
            name: "hotspring-md-validation".into(),
            description: String::new(),
            version: "0.1.0".into(),
            spring: None,
            compute_tier: None,
        };
        assert_eq!(
            infer_spring(&meta, Path::new("/tmp/test.toml")),
            "hotSpring"
        );
    }

    #[test]
    fn infer_spring_unknown_fallback() {
        let meta = WorkloadMetadata {
            name: "custom-workload".into(),
            description: String::new(),
            version: "0.1.0".into(),
            spring: None,
            compute_tier: None,
        };
        assert_eq!(infer_spring(&meta, Path::new("/tmp/test.toml")), "unknown");
    }

    #[test]
    fn parse_workload_toml() {
        let toml_str = r#"
[metadata]
name = "test-workload"
description = "A test"
version = "0.1.0"

[execution]
type = "native"
command = "/bin/echo hello"
working_dir = "/tmp"

[resources]
max_memory_bytes = 1073741824
max_cpu_percent = 80.0

[security]
isolation_level = "None"
"#;
        let workload: WorkloadToml = toml::from_str(toml_str).expect("valid TOML");
        assert_eq!(workload.metadata.name, "test-workload");
        assert_eq!(workload.execution.exec_type, "native");
    }

    #[test]
    fn extract_hex_from_array() {
        let v = serde_json::json!([0, 255, 16]);
        assert_eq!(extract_hex(&v), "00ff10");
    }

    #[test]
    fn extract_hex_from_string() {
        let v = serde_json::json!("abcdef");
        assert_eq!(extract_hex(&v), "abcdef");
    }
}
