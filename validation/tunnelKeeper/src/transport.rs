//! Transport abstraction for dual-architecture tunnel evolution.
//!
//! v0.1: `CloudflareTunnelTransport` — manages the `cloudflared` process
//! v0.2: `SongbirdTransport` — songbird-quic + songbird-tls as library deps
//! v0.3: `BearDogAuthTransport` — full primal auth via beardog-auth
//!
//! Cloudflare remains primary until Songbird/BearDog reach parity.

use serde::Serialize;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Clone, Serialize)]
pub struct TunnelHandle {
    pub transport_name: String,
    pub status: String,
    pub endpoint: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct TransportHealth {
    pub transport_name: String,
    pub healthy: bool,
    pub latency_ms: Option<u64>,
    pub detail: String,
}

/// Core trait for tunnel transport backends.
///
/// Each implementation manages a different transport mechanism:
/// - `CloudflareTunnelTransport`: wraps the `cloudflared` binary (current)
/// - `SongbirdTransport`: pure Rust QUIC via `songbird-quic` (planned)
/// - `BearDogAuthTransport`: primal-native auth + crypto (planned)
///
/// The dual-architecture pattern: external service (Cloudflare) is primary,
/// primal implementation (Songbird/BearDog) runs as shadow, until parity
/// metrics confirm the primal path is equivalent or better.
pub trait TunnelTransport {
    fn name(&self) -> &str;

    fn establish(
        &self,
        config: &crate::config::TunnelConfig,
    ) -> Pin<Box<dyn Future<Output = Result<TunnelHandle, TransportError>> + Send + '_>>;

    fn health(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<TransportHealth, TransportError>> + Send + '_>>;
}

#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    #[error("transport unavailable: {0}")]
    Unavailable(String),
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("process error: {0}")]
    Process(String),
}

// ─── v0.1: Cloudflare Tunnel Transport ─────────────────────────────

pub struct CloudflareTunnelTransport {
    cloudflared_bin: String,
}

impl Default for CloudflareTunnelTransport {
    fn default() -> Self {
        Self {
            cloudflared_bin: "cloudflared".to_string(),
        }
    }
}

impl CloudflareTunnelTransport {
    #[must_use]
    pub fn new(bin_path: &str) -> Self {
        Self {
            cloudflared_bin: bin_path.to_string(),
        }
    }

    fn check_binary(&self) -> Result<String, TransportError> {
        let output = std::process::Command::new(&self.cloudflared_bin)
            .arg("--version")
            .output()
            .map_err(|e| TransportError::Unavailable(format!("cloudflared not found: {e}")))?;

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }
}

impl TunnelTransport for CloudflareTunnelTransport {
    fn name(&self) -> &'static str {
        "cloudflare"
    }

    fn establish(
        &self,
        config: &crate::config::TunnelConfig,
    ) -> Pin<Box<dyn Future<Output = Result<TunnelHandle, TransportError>> + Send + '_>> {
        let tunnel_name = config.tunnel.clone();
        Box::pin(async move {
            let version = self.check_binary()?;

            // Check if already running
            let pgrep = std::process::Command::new("pgrep")
                .args(["-f", "cloudflared.*tunnel"])
                .output();

            let status = match pgrep {
                Ok(o) if o.status.success() => "running".to_string(),
                _ => "stopped".to_string(),
            };

            Ok(TunnelHandle {
                transport_name: format!("cloudflare ({version})"),
                status,
                endpoint: tunnel_name,
            })
        })
    }

    fn health(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<TransportHealth, TransportError>> + Send + '_>> {
        Box::pin(async move {
            let version = self.check_binary()?;

            let running = std::process::Command::new("pgrep")
                .args(["-f", "cloudflared.*tunnel"])
                .output()
                .is_ok_and(|o| o.status.success());

            Ok(TransportHealth {
                transport_name: format!("cloudflare ({version})"),
                healthy: running,
                latency_ms: None,
                detail: if running {
                    "cloudflared process running".into()
                } else {
                    "cloudflared process not found".into()
                },
            })
        })
    }
}

// ─── v0.2: Songbird Transport (planned) ────────────────────────────
//
// When songbird-quic and songbird-tls are wired as library dependencies:
//
// ```rust
// pub struct SongbirdTransport {
//     quic_config: songbird_quic::QuicConfig,
//     tls_config: songbird_tls::TlsConfig,
//     crypto_provider: songbird_crypto_provider::Provider,
// }
//
// impl TunnelTransport for SongbirdTransport {
//     fn name(&self) -> &str { "songbird" }
//     // QUIC connection establishment via songbird-quic
//     // TLS 1.3 handshake via songbird-tls
//     // Crypto delegation to BearDog via songbird-crypto-provider IPC
// }
// ```

// ─── v0.3: BearDog Auth Transport (planned) ────────────────────────
//
// Full primal-native transport with BearDog handling:
// - Identity (beardog-auth): Ed25519/X25519 ionic tokens
// - Tunnel crypto (beardog-tunnel): ChaCha20-Poly1305 tunnel encryption
// - Threat detection (beardog-threat): Dark Forest challenge-response
//
// ```rust
// pub struct BearDogAuthTransport {
//     auth: beardog_auth::AuthClient,
//     tunnel: beardog_tunnel::TunnelClient,
//     songbird: SongbirdTransport,
// }
// ```
