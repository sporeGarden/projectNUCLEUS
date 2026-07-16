use std::path::{Path, PathBuf};

use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::config::NucleusConfig;
use crate::process;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("deployment readiness: {count} issue(s) found")]
    ReadinessCheckFailed { count: u32 },

    #[error("plasmidBin not found at {0}")]
    PlasmidBinMissing(PathBuf),

    #[error("JSON-RPC over UDS: {0}")]
    JsonRpc(#[from] JsonRpcError),
}

#[derive(Debug, Error)]
pub enum JsonRpcError {
    #[error("connect: {0}")]
    Connect(std::io::Error),

    #[error("serialize: {0}")]
    Serialize(serde_json::Error),

    #[error("write: {0}")]
    Write(std::io::Error),

    #[error("shutdown: {0}")]
    Shutdown(std::io::Error),

    #[error("read: {0}")]
    Read(std::io::Error),

    #[error("response not valid UTF-8: {0}")]
    Utf8(std::string::FromUtf8Error),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum Composition {
    Tower,
    Agent,
    Node,
    Nest,
    Full,
}

impl std::fmt::Display for Composition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tower => write!(f, "tower"),
            Self::Agent => write!(f, "agent"),
            Self::Node => write!(f, "node"),
            Self::Nest => write!(f, "nest"),
            Self::Full => write!(f, "full"),
        }
    }
}

pub enum DeployAction {
    Start {
        composition: Composition,
        gate: Option<String>,
        family_name: Option<String>,
        uds_only: bool,
        graph_deploy: bool,
    },
    Stop,
    Status,
}

fn primals_for_composition(comp: Composition) -> Vec<&'static str> {
    match comp {
        Composition::Tower => nucleus_primals::COMP_TOWER.to_vec(),
        Composition::Agent => nucleus_primals::COMP_AGENT.to_vec(),
        Composition::Node => nucleus_primals::COMP_NODE.to_vec(),
        Composition::Nest => nucleus_primals::COMP_NEST.to_vec(),
        Composition::Full => nucleus_primals::COMP_FULL.to_vec(),
    }
}

fn graph_for_composition(project_root: &Path, comp: Composition) -> PathBuf {
    let name = match comp {
        Composition::Tower => "tower_atomic.toml",
        Composition::Agent => "tower_agent.toml",
        Composition::Node => "node_atomic_compute.toml",
        Composition::Nest => "nest_atomic.toml",
        Composition::Full => "nucleus_complete.toml",
    };
    project_root.join("graphs").join(name)
}

pub async fn run(cfg: &NucleusConfig, action: &DeployAction) -> Result<(), DeployError> {
    match action {
        DeployAction::Stop => stop_all(cfg).await,
        DeployAction::Status => status_all(cfg).await,
        DeployAction::Start {
            composition,
            gate,
            family_name,
            uds_only,
            graph_deploy,
        } => {
            if *graph_deploy {
                graph_deploy_via_biomeos(cfg, *composition, gate.as_deref()).await
            } else {
                start_composition(
                    cfg,
                    *composition,
                    gate.as_deref(),
                    family_name.as_deref(),
                    *uds_only,
                )
                .await
            }
        }
    }
}

// ── Graph Deploy (biomeOS orchestrated) ──────────────────────────────────

#[expect(
    clippy::too_many_lines,
    reason = "multi-phase deploy protocol with user-facing diagnostics"
)]
async fn graph_deploy_via_biomeos(
    cfg: &NucleusConfig,
    composition: Composition,
    gate: Option<&str>,
) -> Result<(), DeployError> {
    let graph_file = graph_for_composition(&cfg.project_root, composition);
    let graph_id = graph_file.file_stem().map_or_else(
        || composition.to_string(),
        |s| s.to_string_lossy().to_string(),
    );
    let hostname = process::hostname().await;
    let gate_name = gate.unwrap_or(&hostname);

    eprintln!();
    eprintln!("╔══════════════════════════════════════════════╗");
    eprintln!("║  projectNUCLEUS — Graph Deploy via biomeOS");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  Gate:        {gate_name}");
    eprintln!("  Graph:       {}", graph_file.display());
    eprintln!("  Graph ID:    {graph_id}");
    eprintln!();

    let runtime_dir = &cfg.runtime_dir;
    let biomeos_dir = runtime_dir.join("biomeos");
    let neural_sock = biomeos_dir.join("neural-api-default.sock");

    if !neural_sock.exists() {
        eprintln!(
            "ERROR: biomeOS neural-api socket not found at {}",
            neural_sock.display()
        );
        eprintln!("  biomeOS must be running for --graph-deploy.");
        eprintln!("  Start with: nucleus-deploy deploy --composition agent");
        eprintln!("  Then retry: nucleus-deploy deploy --composition full --graph-deploy");
        return Err(DeployError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "biomeOS neural-api socket not found",
        )));
    }

    eprintln!("=== Phase 1: Probe biomeOS ===");
    match jsonrpc_uds(&neural_sock, "health.liveness", serde_json::json!({})).await {
        Ok(resp) => eprintln!("  biomeOS alive: {resp}"),
        Err(e) => {
            eprintln!("  biomeOS unreachable: {e}");
            return Err(e.into());
        }
    }

    let graph_content = if graph_file.exists() {
        std::fs::read_to_string(&graph_file).ok()
    } else {
        None
    };

    let primals = primals_for_composition(composition);
    eprintln!();
    eprintln!("=== Phase 2: Deploy graph via composition.deploy ===");
    let mut params = serde_json::json!({
        "graph_id": graph_id,
        "graph_path": graph_file.to_string_lossy(),
        "gate": gate_name,
        "primals": primals,
    });
    if let Some(content) = graph_content {
        params["graph_content"] = serde_json::Value::String(content);
    }
    match jsonrpc_uds(&neural_sock, "composition.deploy", params).await {
        Ok(ref resp) if resp.contains("\"error\"") => {
            eprintln!("  composition.deploy returned error: {resp}");
            eprintln!();
            if resp.contains("capability token") || resp.contains("Permission denied") {
                eprintln!("  AUTH GATE: biomeOS requires a BTSP capability token.");
                eprintln!("  Use direct deploy (without --graph-deploy) until auth is integrated.");
                return Err(DeployError::Io(std::io::Error::new(
                    std::io::ErrorKind::PermissionDenied,
                    "composition.deploy requires capability token",
                )));
            } else if resp.contains("not found") || resp.contains("Not found") {
                eprintln!("  GRAPH RESOLUTION: biomeOS cannot find the graph.");
                eprintln!("  Ensure graph definitions exist in the biomeOS graph directory.");
                return Err(DeployError::Io(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("graph '{graph_id}' not found by biomeOS"),
                )));
            }
            eprintln!("  biomeOS returned an unexpected error.");
            return Err(DeployError::Io(std::io::Error::other(format!(
                "composition.deploy error: {resp}"
            ))));
        }
        Ok(resp) => {
            eprintln!("  composition.deploy result: {resp}");

            let execution_id = serde_json::from_str::<serde_json::Value>(&resp)
                .ok()
                .and_then(|v| v["result"]["execution_id"].as_str().map(String::from));

            eprintln!();
            eprintln!("=== Phase 3: Verify via graph.status ===");
            let status_params = execution_id.as_ref().map_or_else(
                || serde_json::json!({ "graph_id": graph_id }),
                |eid| serde_json::json!({ "graph_id": graph_id, "execution_id": eid }),
            );
            match jsonrpc_uds(&neural_sock, "graph.status", status_params).await {
                Ok(s) => eprintln!("  graph.status: {s}"),
                Err(e) => eprintln!("  graph.status unavailable: {e}"),
            }
        }
        Err(e) => {
            eprintln!("  composition.deploy failed: {e}");
            eprintln!("  Falling back to direct process launch...");
            eprintln!();
            return Err(e.into());
        }
    }

    eprintln!();
    eprintln!("╔══════════════════════════════════════════════╗");
    eprintln!("║  Graph deploy complete via biomeOS           ║");
    eprintln!("╚══════════════════════════════════════════════╝");

    Ok(())
}

async fn jsonrpc_uds(
    sock_path: &Path,
    method: &str,
    params: serde_json::Value,
) -> Result<String, JsonRpcError> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixStream;

    let mut stream = UnixStream::connect(sock_path)
        .await
        .map_err(JsonRpcError::Connect)?;

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "params": params,
        "id": 1
    });
    let payload = serde_json::to_vec(&request).map_err(JsonRpcError::Serialize)?;

    stream
        .write_all(&payload)
        .await
        .map_err(JsonRpcError::Write)?;
    stream.shutdown().await.map_err(JsonRpcError::Shutdown)?;

    let mut buf = Vec::new();
    stream
        .read_to_end(&mut buf)
        .await
        .map_err(JsonRpcError::Read)?;

    String::from_utf8(buf).map_err(JsonRpcError::Utf8)
}

// ── Stop ─────────────────────────────────────────────────────────────────

async fn stop_all(cfg: &NucleusConfig) -> Result<(), DeployError> {
    eprintln!("Stopping all primals...");

    let bin_dir = cfg.plasmidbin_dir.join("primals");
    if let Ok(mut entries) = fs::read_dir(&bin_dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            let pattern = format!("{}/primals/{name}", cfg.plasmidbin_dir.display());
            let _ = Command::new("pkill").args(["-f", &pattern]).output().await;
        }
    }

    sleep(Duration::from_secs(1)).await;
    eprintln!("Done.");
    Ok(())
}

// ── Status ───────────────────────────────────────────────────────────────

async fn status_all(cfg: &NucleusConfig) -> Result<(), DeployError> {
    eprintln!("=== NUCLEUS Status ===");

    let bin_dir = cfg.plasmidbin_dir.join("primals");
    let mut running_count = 0u32;

    if let Ok(mut entries) = fs::read_dir(&bin_dir).await {
        let mut names = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            names.push(entry.file_name().to_string_lossy().to_string());
        }
        names.sort();

        for name in &names {
            let pattern = format!("{}/primals/{name}", cfg.plasmidbin_dir.display());
            let output = Command::new("pgrep").args(["-f", &pattern]).output().await;

            if let Ok(o) = output {
                if o.status.success() {
                    let pid = String::from_utf8_lossy(&o.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    eprintln!("  {name}: PID {pid} — RUNNING");
                    running_count += 1;
                }
            }
        }
    }

    eprintln!("  Total: {running_count} primal(s) running");
    Ok(())
}

// ── Start ────────────────────────────────────────────────────────────────

async fn start_composition(
    cfg: &NucleusConfig,
    composition: Composition,
    gate: Option<&str>,
    family_name: Option<&str>,
    uds_only: bool,
) -> Result<(), DeployError> {
    if !cfg.plasmidbin_dir.exists() {
        return Err(DeployError::PlasmidBinMissing(cfg.plasmidbin_dir.clone()));
    }

    let primals = primals_for_composition(composition);
    let graph_file = graph_for_composition(&cfg.project_root, composition);

    let hostname = process::hostname().await;
    let gate_name = gate.unwrap_or(&hostname);
    let family = family_name.unwrap_or(&hostname);
    let transport = if uds_only {
        "UDS-only (VPS standard)"
    } else {
        "UDS + TCP fallback"
    };

    eprintln!();
    eprintln!("╔══════════════════════════════════════════════╗");
    eprintln!("║  projectNUCLEUS — Deploy {composition}");
    eprintln!("╚══════════════════════════════════════════════╝");
    eprintln!();
    eprintln!("  Gate:        {gate_name}");
    eprintln!("  Composition: {composition} ({})", primals.join(", "));
    eprintln!("  Transport:   {transport}");
    eprintln!("  plasmidBin:  {}", cfg.plasmidbin_dir.display());
    eprintln!("  Family:      {family}");
    eprintln!();

    // Readiness checks
    let issues = readiness_check(cfg, &primals, &graph_file);
    if issues > 0 {
        let skip = std::env::var("NUCLEUS_SKIP_READINESS").is_ok_and(|v| v == "1");
        if !skip {
            return Err(DeployError::ReadinessCheckFailed { count: issues });
        }
        eprintln!("  NUCLEUS_SKIP_READINESS=1 — proceeding despite issues.");
    }

    // Phase 1: Family seed
    let family_dir = dirs_family();
    let family_id = phase_family_seed(cfg, &family_dir, family, gate_name).await;

    // Phase 2: Stop existing
    eprintln!("=== Phase 2: Clean slate ===");
    for p in &primals {
        let pattern = format!("{}/primals/{p}", cfg.plasmidbin_dir.display());
        let _ = Command::new("pkill").args(["-f", &pattern]).output().await;
    }
    sleep(Duration::from_secs(1)).await;
    eprintln!("  Previous instances stopped.");
    eprintln!();

    // Phase 3: Start primals
    eprintln!("=== Phase 3: Start primals ===");
    let runtime_dir = &cfg.runtime_dir;
    let _ = fs::create_dir_all(runtime_dir.join("biomeos")).await;

    let bind = &cfg.bind_address;
    let beacon_seed = family_dir.join(".beacon.seed");
    let beardog_socket = runtime_dir.join(format!("biomeos/beardog-{family_id}.sock"));

    let ctx = process::PrimalContext {
        cfg,
        bind,
        family_id: &family_id,
        node_id: gate_name,
        beacon_seed: &beacon_seed,
        beardog_socket: &beardog_socket,
        uds_only,
    };

    for p in &primals {
        process::start_primal(&ctx, p).await;
    }

    eprintln!();

    // Phase 4: Verify
    eprintln!("=== Phase 4: Verify ===");
    let all_ok = process::verify_primals(cfg, &primals, uds_only, runtime_dir).await;
    eprintln!();

    if all_ok {
        eprintln!("╔══════════════════════════════════════════════╗");
        eprintln!("║  All primals running. Gate is live.          ║");
        eprintln!("╚══════════════════════════════════════════════╝");
    } else {
        eprintln!("WARNING: Some primals failed to start. Check logs in /tmp/");
    }

    eprintln!();
    eprintln!("  Family ID:   {family_id}");
    eprintln!("  Node ID:     {gate_name}");
    eprintln!("  Graph:       {}", graph_file.display());
    eprintln!("  Composition: {composition} ({} primals)", primals.len());
    eprintln!();
    eprintln!("  To stop:   nucleus-deploy deploy --stop");
    eprintln!("  To check:  nucleus-deploy deploy --status");

    Ok(())
}

fn readiness_check(cfg: &NucleusConfig, primals: &[&str], graph_file: &Path) -> u32 {
    eprintln!("=== Deployment Readiness ===");
    eprintln!("  Graph: {}", graph_file.display());

    let mut issues = 0u32;

    if !graph_file.exists() {
        eprintln!(
            "  [Structure] Graph file not found: {}",
            graph_file.display()
        );
        issues += 1;
    }

    for p in primals {
        let bin = cfg.plasmidbin_dir.join(format!("primals/{p}"));
        if !bin.exists() {
            eprintln!(
                "  [BinaryMissing] {p} — run 'bash {}/fetch.sh --all' first",
                cfg.plasmidbin_dir.display()
            );
            issues += 1;
        }
    }

    if primals.contains(&"beardog") {
        let family_dir = dirs_family();
        let has_seed = std::env::var("BEARDOG_FAMILY_SEED").is_ok()
            || family_dir.join(".beacon.seed").exists();
        if !has_seed {
            eprintln!("  [EnvMissing] BEARDOG_FAMILY_SEED not set and no .beacon.seed found");
            issues += 1;
        }
    }

    if !primals.contains(&"beardog") {
        eprintln!("  [BondingInconsistent] BTSP required but beardog not in composition");
        issues += 1;
    }

    issues
}

fn dirs_family() -> PathBuf {
    let config_home = std::env::var("XDG_CONFIG_HOME").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_else(|_| "/root".into());
        format!("{home}/.config")
    });
    PathBuf::from(format!("{config_home}/biomeos/family"))
}

async fn phase_family_seed(
    cfg: &NucleusConfig,
    family_dir: &Path,
    family_name: &str,
    node_id: &str,
) -> String {
    eprintln!("=== Phase 1: Family seed ===");

    let id_file = family_dir.join("family_id");
    if let Ok(id) = fs::read_to_string(&id_file).await {
        let trimmed = id.trim().to_string();
        eprintln!("  Existing family: {trimmed}");
        return trimmed;
    }

    eprintln!("  Initializing new family: {family_name}");
    let seed_script = cfg.plasmidbin_dir.join("seed_workflow.sh");
    let _ = Command::new("bash")
        .args([
            seed_script.to_string_lossy().as_ref(),
            "init",
            "--family-name",
            family_name,
        ])
        .output()
        .await;

    let family_id = fs::read_to_string(&id_file)
        .await
        .map_or_else(|_| "unknown".into(), |s| s.trim().to_string());

    let lineage_file = family_dir.join(format!("nodes/{node_id}.lineage.seed"));
    if !lineage_file.exists() {
        eprintln!("  Adding node: {node_id}");
        let _ = Command::new("bash")
            .args([
                seed_script.to_string_lossy().as_ref(),
                "add-node",
                "--node-id",
                node_id,
            ])
            .output()
            .await;
    }

    eprintln!("  Family ID: {family_id}");
    eprintln!();
    family_id
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition_display() {
        assert_eq!(Composition::Tower.to_string(), "tower");
        assert_eq!(Composition::Full.to_string(), "full");
    }

    #[test]
    fn tower_has_beardog_songbird_skunkbat() {
        let p = primals_for_composition(Composition::Tower);
        assert!(p.contains(&"beardog"));
        assert!(p.contains(&"songbird"));
        assert!(p.contains(&"skunkbat"));
        assert_eq!(p.len(), 3);
    }

    #[test]
    fn full_has_all_13_primals() {
        let p = primals_for_composition(Composition::Full);
        assert_eq!(p.len(), 13);
        assert!(p.contains(&"beardog"));
        assert!(p.contains(&"petaltongue"));
    }

    #[test]
    fn node_includes_compute_primals() {
        let p = primals_for_composition(Composition::Node);
        assert!(p.contains(&"toadstool"));
        assert!(p.contains(&"barracuda"));
        assert!(p.contains(&"coralreef"));
    }

    #[test]
    fn nest_includes_provenance_primals() {
        let p = primals_for_composition(Composition::Nest);
        assert!(p.contains(&"nestgate"));
        assert!(p.contains(&"rhizocrypt"));
        assert!(p.contains(&"loamspine"));
        assert!(p.contains(&"sweetgrass"));
    }

    #[test]
    fn graph_file_paths_are_toml() {
        let root = PathBuf::from("/tmp/test");
        for comp in [
            Composition::Tower,
            Composition::Agent,
            Composition::Node,
            Composition::Nest,
            Composition::Full,
        ] {
            let path = graph_for_composition(&root, comp);
            assert!(
                path.extension().is_some_and(|ext| ext == "toml"),
                "Graph for {comp} should be .toml: {path:?}"
            );
        }
    }

    #[test]
    fn dirs_family_returns_valid_path() {
        let path = dirs_family();
        assert!(path.to_string_lossy().contains("biomeos/family"));
    }
}
