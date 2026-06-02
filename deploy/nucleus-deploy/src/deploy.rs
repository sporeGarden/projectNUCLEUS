use std::path::{Path, PathBuf};

use thiserror::Error;
use tokio::fs;
use tokio::process::Command;
use tokio::time::{sleep, Duration};

use crate::config::NucleusConfig;

#[derive(Debug, Error)]
pub enum DeployError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("deployment readiness: {count} issue(s) found")]
    ReadinessCheckFailed { count: u32 },

    #[error("plasmidBin not found at {0}")]
    PlasmidBinMissing(PathBuf),
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
    },
    Stop,
    Status,
}

fn primals_for_composition(comp: Composition) -> Vec<&'static str> {
    match comp {
        Composition::Tower => vec!["beardog", "songbird"],
        Composition::Agent => vec!["beardog", "songbird", "skunkbat", "biomeos", "squirrel"],
        Composition::Node => vec![
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "skunkbat",
        ],
        Composition::Nest => vec![
            "beardog",
            "songbird",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "skunkbat",
        ],
        Composition::Full => vec![
            "beardog",
            "songbird",
            "toadstool",
            "barracuda",
            "coralreef",
            "nestgate",
            "rhizocrypt",
            "loamspine",
            "sweetgrass",
            "squirrel",
            "skunkbat",
            "biomeos",
            "petaltongue",
        ],
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
        } => {
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

    let hostname = hostname().await;
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

    let ctx = PrimalContext {
        cfg,
        bind,
        family_id: &family_id,
        node_id: gate_name,
        beacon_seed: &beacon_seed,
        beardog_socket: &beardog_socket,
        uds_only,
    };

    for p in &primals {
        start_primal(&ctx, p).await;
    }

    eprintln!();

    // Phase 4: Verify
    eprintln!("=== Phase 4: Verify ===");
    let all_ok = verify_primals(cfg, &primals, uds_only, runtime_dir).await;
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

struct PrimalContext<'a> {
    cfg: &'a NucleusConfig,
    bind: &'a str,
    family_id: &'a str,
    node_id: &'a str,
    beacon_seed: &'a Path,
    beardog_socket: &'a Path,
    uds_only: bool,
}

async fn start_primal(ctx: &PrimalContext<'_>, name: &str) {
    let bin = ctx.cfg.plasmidbin_dir.join(format!("primals/{name}"));
    if !bin.exists() {
        eprintln!("  SKIP {name} — no binary found");
        return;
    }

    let port_for = |port: u16| -> u16 {
        if ctx.uds_only {
            0
        } else {
            port
        }
    };

    let mut cmd = Command::new(&bin);
    configure_primal_cmd(&mut cmd, ctx, name, port_for);

    let log_path = format!("/tmp/{name}.log");
    let log_file = std::fs::File::create(&log_path).ok();

    if let Some(f) = log_file {
        if let Ok(clone) = f.try_clone() {
            cmd.stdout(clone);
        }
        cmd.stderr(f);
    }

    let auth_mode = std::env::var("NUCLEUS_AUTH_MODE").unwrap_or_else(|_| "enforced".into());
    cmd.env(format!("{}_AUTH_MODE", name.to_uppercase()), &auth_mode);

    match cmd.spawn() {
        Ok(child) => {
            let pid = child.id().unwrap_or(0);
            eprintln!("    PID: {pid}");
        }
        Err(e) => eprintln!("    FAILED: {e}"),
    }

    let delay = match name {
        "beardog" | "songbird" | "biomeos" | "nestgate" => 2,
        _ => 1,
    };
    sleep(Duration::from_secs(delay)).await;
}

fn configure_primal_cmd(
    cmd: &mut Command,
    ctx: &PrimalContext<'_>,
    name: &str,
    port_for: impl Fn(u16) -> u16,
) {
    let cfg = ctx.cfg;
    let bind = ctx.bind;
    let family_id = ctx.family_id;
    let runtime = &cfg.runtime_dir;

    match name {
        "beardog" => {
            let port = port_for(cfg.port_for("beardog"));
            let transport = if port > 0 {
                format!("UDS + TCP {port}")
            } else {
                "UDS-only".into()
            };
            eprintln!("  Starting beardog ({transport})...");
            cmd.env("BEARDOG_FAMILY_SEED", ctx.beacon_seed);
            cmd.args(["server", "--socket"]);
            cmd.arg(ctx.beardog_socket);
            cmd.args(["--family-id", family_id]);
            if port > 0 {
                cmd.args(["--listen", &format!("{bind}:{port}")]);
            }
        }
        "songbird" => {
            let port = port_for(cfg.port_for("songbird"));
            let transport = if port > 0 {
                format!("HTTP {port}")
            } else {
                "UDS-only".into()
            };
            eprintln!("  Starting songbird ({transport})...");
            cmd.env("BEARDOG_SOCKET", ctx.beardog_socket);
            cmd.env("BEARDOG_MODE", "direct");
            cmd.env("SONGBIRD_SECURITY_PROVIDER", "beardog");
            let sock = runtime.join(format!("biomeos/songbird-{family_id}.sock"));
            cmd.args(["server", "--socket"]);
            cmd.arg(&sock);
            if port > 0 {
                cmd.args(["--port", &port.to_string()]);
            }
        }
        "toadstool" => {
            let port = port_for(cfg.port_for("toadstool"));
            eprintln!("  Starting toadstool (TCP {port})...");
            cmd.env("TOADSTOOL_FAMILY_ID", family_id);
            cmd.env("TOADSTOOL_NODE_ID", ctx.node_id);
            cmd.env("TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED", "1");
            cmd.args(["server", "--family-id", family_id]);
            if port > 0 {
                cmd.args(["--port", &port.to_string()]);
            }
        }
        "nestgate" => {
            let port = port_for(cfg.port_for("nestgate"));
            eprintln!("  Starting nestgate (TCP {port})...");
            cmd.env("NESTGATE_FAMILY_ID", family_id);
            cmd.args(["daemon", "--socket-only"]);
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--bind", bind]);
            }
        }
        "rhizocrypt" => {
            let port = port_for(cfg.port_for("rhizocrypt"));
            eprintln!("  Starting rhizocrypt (TCP {port})...");
            cmd.env("FAMILY_SEED", ctx.beacon_seed);
            cmd.arg("server");
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--host", bind]);
            }
        }
        "loamspine" => {
            let port = port_for(cfg.port_for("loamspine"));
            eprintln!("  Starting loamspine (TCP {port})...");
            cmd.arg("server");
            if port > 0 {
                cmd.args(["--port", &port.to_string(), "--bind-address", bind]);
            }
        }
        "sweetgrass" => {
            let port = port_for(cfg.port_for("sweetgrass"));
            eprintln!("  Starting sweetgrass (TCP {port})...");
            cmd.arg("server");
            if port > 0 {
                let http_port = port + 1;
                cmd.args([
                    "--port",
                    &port.to_string(),
                    "--http-address",
                    &format!("{bind}:{http_port}"),
                ]);
            }
        }
        _ => {
            let port = port_for_name(cfg, name);
            let p = port_for(port);
            eprintln!("  Starting {name} (TCP {p})...");
            cmd.arg("server");
            if p > 0 {
                cmd.args(["--port", &p.to_string()]);
            }
        }
    }
}

fn port_for_name(cfg: &NucleusConfig, name: &str) -> u16 {
    cfg.port_for(name)
}

async fn verify_primals(
    cfg: &NucleusConfig,
    primals: &[&str],
    uds_only: bool,
    runtime_dir: &Path,
) -> bool {
    let mut all_ok = true;

    for &p in primals {
        let pattern = format!("{}/primals/{p}", cfg.plasmidbin_dir.display());
        let output = Command::new("pgrep").args(["-f", &pattern]).output().await;

        let pid = output.ok().filter(|o| o.status.success()).map(|o| {
            String::from_utf8_lossy(&o.stdout)
                .lines()
                .next()
                .unwrap_or("")
                .trim()
                .to_string()
        });

        match pid {
            Some(pid) if !pid.is_empty() => {
                if uds_only {
                    let biomeos_dir = runtime_dir.join("biomeos");
                    let has_socket = has_socket_for(&biomeos_dir, p).await;
                    if has_socket {
                        eprintln!("  {p}: PID {pid}, SOCKET LIVE");
                    } else {
                        eprintln!("  {p}: PID {pid}, SOCKET ABSENT — running (socket pending)");
                    }
                } else {
                    let port = port_for_name(cfg, p);
                    if port > 0 {
                        let healthy = crate::rpc::check_liveness(&cfg.bind_address, port).await;
                        if healthy {
                            eprintln!("  {p}: PID {pid}, TCP {port} — HEALTHY");
                        } else {
                            eprintln!(
                                "  {p}: PID {pid}, TCP {port} — running (health probe pending)"
                            );
                        }
                    } else {
                        eprintln!("  {p}: PID {pid} — running");
                    }
                }
            }
            _ => {
                eprintln!("  {p}: NOT RUNNING — check /tmp/{p}.log");
                all_ok = false;
            }
        }
    }

    all_ok
}

async fn has_socket_for(dir: &Path, primal: &str) -> bool {
    if let Ok(mut entries) = fs::read_dir(dir).await {
        while let Ok(Some(entry)) = entries.next_entry().await {
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with(primal)
                && Path::new(&name)
                    .extension()
                    .is_some_and(|ext| ext.eq_ignore_ascii_case("sock"))
            {
                return true;
            }
        }
    }
    false
}

async fn hostname() -> String {
    Command::new("hostname")
        .arg("-s")
        .output()
        .await
        .ok()
        .map_or_else(
            || "unknown".into(),
            |o| String::from_utf8_lossy(&o.stdout).trim().to_string(),
        )
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
    fn tower_has_beardog_and_songbird() {
        let p = primals_for_composition(Composition::Tower);
        assert!(p.contains(&"beardog"));
        assert!(p.contains(&"songbird"));
        assert_eq!(p.len(), 2);
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
