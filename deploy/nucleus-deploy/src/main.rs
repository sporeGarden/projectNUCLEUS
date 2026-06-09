mod config;
mod deploy;
mod dns;
mod provenance;
mod provision;
mod rpc;
mod security;
mod spore;
mod summary;
mod telemetry;
mod util;
mod verify;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use config::NucleusConfig;
use deploy::{Composition, DeployAction};
use dns::{DnsArgs, DnsMode};
use provenance::ProvenanceArgs;
use provision::ProvisionArgs;
use security::{Layer, SecurityArgs};
use spore::SporeArgs;
use summary::SummaryArgs;
use telemetry::{TelemetryArgs, TelemetryMode};
use verify::VerifyArgs;

#[derive(Parser)]
#[command(
    name = "nucleus-deploy",
    about = "Deploy, validate, and prove provenance for NUCLEUS gate compositions",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Five-layer security validation pipeline
    Security {
        /// Which security layer(s) to test
        #[arg(long, value_enum, default_value = "all")]
        layer: Layer,

        /// Tunnel URL for TLS/HSTS verification (layer 3d)
        #[arg(long)]
        tunnel_url: Option<String>,

        /// Target host to probe (default: `NUCLEUS_BIND_ADDRESS`)
        #[arg(long)]
        target: Option<String>,

        /// Directory for results output
        #[arg(long)]
        results: Option<PathBuf>,
    },

    /// Provenance pipeline — full rigor through the Nest Atomic
    Provenance {
        /// Directory containing workload TOML files
        #[arg(long)]
        workloads_dir: Option<PathBuf>,

        /// Directory for results output
        #[arg(long)]
        results_dir: Option<PathBuf>,
    },

    /// Deploy a NUCLEUS composition to a gate
    Deploy {
        /// Composition to deploy
        #[arg(long, value_enum, default_value = "node")]
        composition: Composition,

        /// Gate name (matches `gates/<name>.toml`)
        #[arg(long)]
        gate: Option<String>,

        /// Family name for seed initialization
        #[arg(long)]
        family_name: Option<String>,

        /// VPS standard: no TCP ports
        #[arg(long)]
        uds_only: bool,

        /// Delegate to biomeOS graph.deploy instead of direct process spawning
        #[arg(long)]
        graph_deploy: bool,

        /// Stop all running primals
        #[arg(long)]
        stop: bool,

        /// Show status of running primals
        #[arg(long)]
        status: bool,
    },

    /// Execute workloads and emit pseudoSpores — replicable data objects
    Spore {
        /// Single workload TOML to process
        #[arg(long)]
        workload: Option<PathBuf>,

        /// Directory of workload TOMLs to batch-process
        #[arg(long)]
        workloads_dir: Option<PathBuf>,

        /// Output directory for pseudoSpore artifacts
        #[arg(long, default_value = "./spores")]
        output: PathBuf,

        /// Skip trio provenance (DAG/spine/braid) — development only.
        /// `PseudoSpores` emitted without provenance are NOT deployment-grade.
        #[arg(long)]
        skip_provenance: bool,

        /// Path to litho binary (auto-discovered if not set)
        #[arg(long)]
        litho_bin: Option<PathBuf>,
    },

    /// Collect membrane telemetry — probe external VPS and internal gate
    Telemetry {
        /// Which membrane(s) to probe
        #[arg(long, value_enum, default_value = "all")]
        mode: TelemetryMode,

        /// Override telemetry output directory
        #[arg(long)]
        telemetry_dir: Option<PathBuf>,
    },

    /// Generate membrane 7-day summary from telemetry data
    Summary {
        /// Number of days to summarize
        #[arg(long, default_value = "7")]
        days: u32,

        /// Override telemetry input directory
        #[arg(long)]
        telemetry_dir: Option<PathBuf>,

        /// Override output path for the TOML summary
        #[arg(long)]
        output: Option<PathBuf>,
    },

    /// Verify remote provenance trio (`NestGate`, `rhizoCrypt`, `loamSpine`, `sweetGrass`)
    Verify {
        /// Skip SSH (offline mode)
        #[arg(long)]
        skip_ssh: bool,

        /// Override VPS IP
        #[arg(long)]
        vps_ip: Option<String>,
    },

    /// Provision a gate — sovereign mesh (SSH + plasmidBin + Songbird)
    Provision {
        /// SSH-reachable target (user@host or host)
        target: String,

        /// Show what would happen without executing
        #[arg(long)]
        dry_run: bool,

        /// Full mode (primary gate with all services)
        #[arg(long)]
        full: bool,

        /// Path to plasmidBin directory with pre-built binaries
        #[arg(long)]
        plasmid_bin: Option<PathBuf>,
    },

    /// Deploy or manage knot-dns authoritative server
    Dns {
        /// Action to perform
        #[arg(long, value_enum, default_value = "deploy")]
        mode: DnsMode,

        /// Show what would happen without executing
        #[arg(long)]
        dry_run: bool,

        /// Override VPS IP
        #[arg(long)]
        vps_ip: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg = NucleusConfig::from_env();
    process::exit(dispatch(cli.command, &cfg).await);
}

async fn dispatch_original(cmd: Commands, cfg: &NucleusConfig) -> i32 {
    match cmd {
        Commands::Security {
            layer,
            tunnel_url,
            target,
            results,
        } => {
            let args = SecurityArgs {
                layer,
                tunnel_url,
                target_host: target,
                results_dir: results,
            };
            match security::run(cfg, &args).await {
                Ok(true) => 0,
                Ok(false) => 1,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    2
                }
            }
        }
        Commands::Provenance {
            workloads_dir,
            results_dir,
        } => {
            let args = ProvenanceArgs {
                workloads_dir,
                results_dir,
            };
            match provenance::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        Commands::Deploy {
            composition,
            gate,
            family_name,
            uds_only,
            graph_deploy,
            stop,
            status,
        } => {
            let action = if stop {
                DeployAction::Stop
            } else if status {
                DeployAction::Status
            } else {
                DeployAction::Start {
                    composition,
                    gate,
                    family_name,
                    uds_only,
                    graph_deploy,
                }
            };
            match deploy::run(cfg, &action).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        Commands::Spore {
            workload,
            workloads_dir,
            output,
            skip_provenance,
            litho_bin,
        } => {
            let args = SporeArgs {
                workload,
                workloads_dir,
                output,
                skip_provenance,
                litho_bin,
            };
            match spore::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!("ERROR: command routed to wrong dispatch function");
            1
        }
    }
}

async fn dispatch_extended(cmd: Commands, cfg: &NucleusConfig) -> i32 {
    match cmd {
        Commands::Telemetry {
            mode,
            telemetry_dir,
        } => {
            let args = TelemetryArgs {
                mode,
                telemetry_dir,
            };
            match telemetry::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        Commands::Summary {
            days,
            telemetry_dir,
            output,
        } => {
            let args = SummaryArgs {
                days,
                telemetry_dir,
                output,
            };
            match summary::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        Commands::Verify { skip_ssh, vps_ip } => {
            let args = VerifyArgs { skip_ssh, vps_ip };
            match verify::run(cfg, &args).await {
                Ok(true) => 0,
                Ok(false) => 1,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    2
                }
            }
        }
        Commands::Provision {
            target,
            dry_run,
            full,
            plasmid_bin,
        } => {
            let args = ProvisionArgs {
                target,
                dry_run,
                full,
                plasmid_bin,
            };
            match provision::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        Commands::Dns {
            mode,
            dry_run,
            vps_ip,
        } => {
            let args = DnsArgs {
                mode,
                dry_run,
                vps_ip,
            };
            match dns::run(cfg, &args).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
        _ => {
            eprintln!("ERROR: command routed to wrong dispatch function");
            1
        }
    }
}

async fn dispatch(cmd: Commands, cfg: &NucleusConfig) -> i32 {
    match cmd {
        Commands::Security { .. }
        | Commands::Provenance { .. }
        | Commands::Deploy { .. }
        | Commands::Spore { .. } => dispatch_original(cmd, cfg).await,
        _ => dispatch_extended(cmd, cfg).await,
    }
}
