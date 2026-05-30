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
#[expect(
    dead_code,
    reason = "Planned transport evolution: common interface for Cloudflare, Songbird, and BearDog backends"
)]
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

#[expect(
    dead_code,
    reason = "Planned transport evolution: error surface for Songbird/BearDog transport backends"
)]
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
    #[cfg_attr(
        not(test),
        expect(
            dead_code,
            reason = "Planned transport evolution: constructor for custom cloudflared binary path"
        )
    )]
    pub fn new(bin_path: &str) -> Self {
        Self {
            cloudflared_bin: bin_path.to_string(),
        }
    }

    fn check_binary_blocking(bin: &str) -> Result<String, TransportError> {
        let output = std::process::Command::new(bin)
            .arg("--version")
            .output()
            .map_err(|e| TransportError::Unavailable(format!("cloudflared not found: {e}")))?;

        let version = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok(version)
    }

    fn is_running_blocking() -> bool {
        std::process::Command::new("pgrep")
            .args(["-f", "cloudflared.*tunnel"])
            .output()
            .is_ok_and(|o| o.status.success())
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
        let bin = self.cloudflared_bin.clone();
        Box::pin(async move {
            let (version, running) = tokio::task::spawn_blocking(move || {
                let v = Self::check_binary_blocking(&bin)?;
                let r = Self::is_running_blocking();
                Ok::<_, TransportError>((v, r))
            })
            .await
            .map_err(|e| TransportError::Process(format!("task panicked: {e}")))??;

            let status = if running { "running" } else { "stopped" };
            Ok(TunnelHandle {
                transport_name: format!("cloudflare ({version})"),
                status: status.to_string(),
                endpoint: tunnel_name,
            })
        })
    }

    fn health(
        &self,
    ) -> Pin<Box<dyn Future<Output = Result<TransportHealth, TransportError>> + Send + '_>> {
        let bin = self.cloudflared_bin.clone();
        Box::pin(async move {
            let (version, running) = tokio::task::spawn_blocking(move || {
                let v = Self::check_binary_blocking(&bin)?;
                let r = Self::is_running_blocking();
                Ok::<_, TransportError>((v, r))
            })
            .await
            .map_err(|e| TransportError::Process(format!("task panicked: {e}")))??;

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

// Planned transports (see specs/TUNNEL_EVOLUTION.md):
// - v0.2 SongbirdTransport: Pure Rust QUIC + TLS via songbird-quic/songbird-tls
// - v0.3 BearDogAuthTransport: Primal-native auth (Ed25519 ionic) + tunnel crypto (ChaCha20-Poly1305)

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cloudflare_transport_default_has_cloudflared_bin() {
        let t = CloudflareTunnelTransport::default();
        assert_eq!(t.cloudflared_bin, "cloudflared");
    }

    #[test]
    fn cloudflare_transport_new_sets_bin_path() {
        let t = CloudflareTunnelTransport::new("/usr/local/bin/cloudflared");
        assert_eq!(t.cloudflared_bin, "/usr/local/bin/cloudflared");
    }

    #[test]
    fn transport_error_display() {
        let e = TransportError::Unavailable("test".to_string());
        assert!(e.to_string().contains("test"));
    }

    #[test]
    fn tunnel_handle_serialization() {
        let h = TunnelHandle {
            transport_name: "cloudflare (2024.1.0)".to_string(),
            status: "running".to_string(),
            endpoint: "test-tunnel".to_string(),
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(json.contains("cloudflare"));
    }

    #[test]
    fn transport_health_serialization() {
        let h = TransportHealth {
            transport_name: "cloudflare".to_string(),
            healthy: true,
            latency_ms: Some(5),
            detail: "ok".to_string(),
        };
        let json = serde_json::to_string(&h).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["healthy"], true);
    }
}
