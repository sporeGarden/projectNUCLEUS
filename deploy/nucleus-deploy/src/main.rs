mod config;
mod provenance;
mod rpc;
mod security;

use std::path::PathBuf;
use std::process;

use clap::{Parser, Subcommand};

use config::NucleusConfig;
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
    };

    process::exit(exit_code);
}
