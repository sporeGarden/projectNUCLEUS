mod api;
mod config;
mod crypto;
mod health;
mod transport;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "tunnelKeeper",
    about = "Tunnel Keeper — Pure Rust Cloudflare tunnel manager for NUCLEUS",
    version,
    long_about = "Manages Cloudflare tunnel configuration, health probes, and credential \
                  encryption. Designed for dual-architecture evolution: Cloudflare primary, \
                  Songbird/BearDog secondary, until primal parity is reached."
)]
struct Cli {
    /// cloudflared config file path
    #[arg(long, env = "CLOUDFLARED_CONFIG", default_value = "~/.cloudflared/config.yml")]
    config: PathBuf,

    /// Cloudflare API token (or set CF_API_TOKEN env var)
    #[arg(long, env = "CF_API_TOKEN")]
    api_token: Option<String>,

    /// JSON output instead of human-readable
    #[arg(long, default_value_t = false)]
    json: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Probe tunnel process, CF edge connectivity, DNS resolution, cert expiry
    Health,

    /// Configuration management
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Ingress route management
    Route {
        #[command(subcommand)]
        action: RouteAction,
    },

    /// Credential encryption at rest (BearDog-pattern ChaCha20-Poly1305)
    Creds {
        #[command(subcommand)]
        action: CredsAction,
    },
}

#[derive(Subcommand)]
enum ConfigAction {
    /// Display current config.yml as structured JSON
    Show,
    /// Push local config to CF API (or pull remote state)
    Sync {
        /// Pull remote config instead of pushing local
        #[arg(long, default_value_t = false)]
        pull: bool,
    },
}

#[derive(Subcommand)]
enum RouteAction {
    /// List current ingress rules
    List,
    /// Add a new ingress rule
    Add {
        /// Hostname (e.g. lab.primals.eco)
        #[arg(long)]
        hostname: String,
        /// Path regex (e.g. "/new/.*")
        #[arg(long)]
        path: Option<String>,
        /// Backend service URL (e.g. http://127.0.0.1:8867)
        #[arg(long)]
        service: String,
    },
    /// Remove an ingress rule by path
    Rm {
        /// Path regex to remove
        #[arg(long)]
        path: String,
    },
}

#[derive(Subcommand)]
enum CredsAction {
    /// Encrypt tunnel credentials at rest
    Encrypt {
        /// Path to credentials JSON
        #[arg(long)]
        creds_path: Option<PathBuf>,
    },
    /// Decrypt credentials for cloudflared consumption
    Decrypt {
        /// Path to encrypted credentials
        #[arg(long)]
        creds_path: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Health => health::run(&cli.config, cli.api_token.as_deref(), cli.json).await,
        Commands::Config { action } => match action {
            ConfigAction::Show => config::show(&cli.config, cli.json),
            ConfigAction::Sync { pull } => {
                config::sync(&cli.config, cli.api_token.as_deref(), pull, cli.json).await
            }
        },
        Commands::Route { action } => match action {
            RouteAction::List => config::route_list(&cli.config, cli.json),
            RouteAction::Add {
                hostname,
                path,
                service,
            } => config::route_add(&cli.config, &hostname, path.as_deref(), &service, cli.json),
            RouteAction::Rm { path } => config::route_rm(&cli.config, &path, cli.json),
        },
        Commands::Creds { action } => match action {
            CredsAction::Encrypt { creds_path } => crypto::encrypt_creds(creds_path.as_deref(), cli.json)
                .map_err(config::ConfigError::Crypto),
            CredsAction::Decrypt { creds_path } => crypto::decrypt_creds(creds_path.as_deref(), cli.json)
                .map_err(config::ConfigError::Crypto),
        },
    };

    if let Err(e) = result {
        if cli.json {
            let err = serde_json::json!({"error": e.to_string()});
            eprintln!("{}", serde_json::to_string_pretty(&err).unwrap_or_default());
        } else {
            eprintln!("ERROR: {e}");
        }
        std::process::exit(1);
    }
}
