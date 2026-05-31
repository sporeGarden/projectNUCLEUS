mod config;
mod deploy;
mod provenance;
mod rpc;
mod security;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use config::NucleusConfig;
use deploy::{Composition, DeployAction};
use provenance::ProvenanceArgs;
use security::{Layer, SecurityArgs};

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

        /// Stop all running primals
        #[arg(long)]
        stop: bool,

        /// Show status of running primals
        #[arg(long)]
        status: bool,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    let cfg = NucleusConfig::from_env();

    let exit_code = match cli.command {
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
            match security::run(&cfg, &args).await {
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
            match provenance::run(&cfg, &args).await {
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
                }
            };
            match deploy::run(&cfg, &action).await {
                Ok(()) => 0,
                Err(e) => {
                    eprintln!("ERROR: {e}");
                    1
                }
            }
        }
    };

    process::exit(exit_code);
}
