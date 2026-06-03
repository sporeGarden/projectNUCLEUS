use crate::net::send_jsonrpc_newline;

const DISCOVERY_TIMEOUT_MS: u64 = 3000;
const LIVENESS_TIMEOUT_MS: u64 = 2000;
const CAPABILITY_TIMEOUT_MS: u64 = 2000;

/// Primal endpoint resolved through discovery or fallback
#[derive(Debug, Clone)]
pub struct ResolvedPrimal {
    pub name: String,
    pub port: u16,
    pub source: DiscoverySource,
    pub capabilities: Vec<String>,
    pub live: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiscoverySource {
    /// Port discovered at runtime via biomeOS or Songbird
    Discovered,
    /// Port set explicitly via environment variable
    EnvOverride,
    /// Compiled default used as last resort
    CompiledDefault,
}

impl std::fmt::Display for DiscoverySource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Discovered => write!(f, "discovered"),
            Self::EnvOverride => write!(f, "env"),
            Self::CompiledDefault => write!(f, "default"),
        }
    }
}

/// Resolve all known primals using the discovery cascade:
/// 1. Try biomeOS `primal.list` for runtime topology
/// 2. Fall back to env vars for each primal
/// 3. Last resort: compiled defaults from `nucleus-primals` shared registry
pub fn resolve_primals(host: &str) -> Vec<ResolvedPrimal> {
    let discovered = try_biomeos_discovery(host);
    if !discovered.is_empty() {
        return discovered;
    }

    nucleus_primals::PRIMALS
        .iter()
        .map(|def| {
            let (port, source) = std::env::var(def.env_key)
                .ok()
                .and_then(|v| v.parse().ok())
                .map_or((def.default_port, DiscoverySource::CompiledDefault), |p| {
                    (p, DiscoverySource::EnvOverride)
                });
            ResolvedPrimal {
                name: def.slug.to_string(),
                port,
                source,
                capabilities: Vec::new(),
                live: false,
            }
        })
        .collect()
}

/// Resolve a single primal's port via env var or compiled default.
pub fn port_for(primal: &str) -> u16 {
    nucleus_primals::lookup(primal).map_or(0, nucleus_primals::resolve_port)
}

/// Ask biomeOS for the live primal topology via `primal.list`.
///
/// Wave 20 canonical response: `{ "primals": [...], "count": N }`
/// Also accepts legacy raw-array form for backward compat.
fn try_biomeos_discovery(host: &str) -> Vec<ResolvedPrimal> {
    let biomeos_port = port_for("biomeos");

    let payload = r#"{"jsonrpc":"2.0","method":"primal.list","params":{},"id":1}"#;
    let Some(body) = send_jsonrpc_newline(host, biomeos_port, payload, DISCOVERY_TIMEOUT_MS) else {
        return Vec::new();
    };

    let Ok(value) = serde_json::from_str::<serde_json::Value>(&body) else {
        return Vec::new();
    };

    let Some(result) = value.get("result") else {
        return Vec::new();
    };

    // Canonical: { "primals": [...], "count": N } — prefer envelope, fall back to raw array
    let Some(primals) = result
        .get("primals")
        .and_then(|p| p.as_array())
        .or_else(|| result.as_array())
    else {
        return Vec::new();
    };

    primals
        .iter()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?;
            // Canonical entry: { "name", "socket", optional "port", "capabilities", "pid", "status", "version" }
            let port = entry
                .get("port")
                .and_then(serde_json::Value::as_u64)
                .and_then(|p| u16::try_from(p).ok())?;
            let capabilities = entry
                .get("capabilities")
                .and_then(|c| c.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            Some(ResolvedPrimal {
                name: name.to_string(),
                port,
                source: DiscoverySource::Discovered,
                capabilities,
                live: true,
            })
        })
        .collect()
}

/// Probe a single primal for liveness via JSON-RPC `health.liveness`
pub fn probe_liveness(host: &str, primal: &mut ResolvedPrimal) {
    let payload = r#"{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":1}"#;
    if let Some(body) = send_jsonrpc_newline(host, primal.port, payload, LIVENESS_TIMEOUT_MS)
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&body)
    {
        primal.live = v.get("result").is_some();
    }
}

/// Query a primal's capabilities via JSON-RPC `capability.list`.
///
/// Wave 20 canonical response: `{ "capabilities": [...], "count": N, "primal": "name" }`
/// Also accepts legacy `{ "methods": [...] }` or raw array for backward compat.
pub fn probe_capabilities(host: &str, primal: &mut ResolvedPrimal) {
    let payload = r#"{"jsonrpc":"2.0","method":"capability.list","params":{},"id":1}"#;
    if let Some(body) = send_jsonrpc_newline(host, primal.port, payload, CAPABILITY_TIMEOUT_MS)
        && let Ok(v) = serde_json::from_str::<serde_json::Value>(&body)
        && let Some(result) = v.get("result")
    {
        // Canonical: { "capabilities": [...] } — fall back to "methods" or raw array
        let methods = result
            .get("capabilities")
            .or_else(|| result.get("methods"))
            .and_then(|m| m.as_array())
            .or_else(|| result.as_array());

        if let Some(arr) = methods {
            primal.capabilities = arr
                .iter()
                .filter_map(|m| m.as_str().map(String::from))
                .collect();
        }
    }
}

/// Find primals that advertise a specific capability
pub fn by_capability<'a>(
    primals: &'a [ResolvedPrimal],
    capability: &str,
) -> Vec<&'a ResolvedPrimal> {
    primals
        .iter()
        .filter(|p| p.capabilities.iter().any(|c| c == capability))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_all_have_env_keys() {
        for def in nucleus_primals::PRIMALS {
            assert!(!def.slug.is_empty());
            assert!(
                def.env_key.ends_with("_PORT"),
                "{} missing _PORT suffix",
                def.env_key
            );
            assert!(
                def.default_port > 1024,
                "{} port {} in privileged range",
                def.slug,
                def.default_port
            );
        }
    }

    #[test]
    fn resolve_primals_returns_all_defaults_offline() {
        let primals = resolve_primals("192.0.2.1");
        assert_eq!(primals.len(), 14);
        for p in &primals {
            assert!(
                matches!(
                    p.source,
                    DiscoverySource::CompiledDefault | DiscoverySource::EnvOverride
                ),
                "{} should be default or env, not {:?}",
                p.name,
                p.source,
            );
        }
    }

    #[test]
    fn discovery_source_display() {
        assert_eq!(format!("{}", DiscoverySource::Discovered), "discovered");
        assert_eq!(format!("{}", DiscoverySource::EnvOverride), "env");
        assert_eq!(format!("{}", DiscoverySource::CompiledDefault), "default");
    }

    #[test]
    fn by_capability_filters_correctly() {
        let primals = vec![
            ResolvedPrimal {
                name: "beardog".to_string(),
                port: 9100,
                source: DiscoverySource::CompiledDefault,
                capabilities: vec!["crypto.sign".to_string(), "crypto.verify".to_string()],
                live: true,
            },
            ResolvedPrimal {
                name: "songbird".to_string(),
                port: 9200,
                source: DiscoverySource::CompiledDefault,
                capabilities: vec!["discovery.query".to_string()],
                live: true,
            },
        ];
        let crypto = by_capability(&primals, "crypto.sign");
        assert_eq!(crypto.len(), 1);
        assert_eq!(crypto[0].name, "beardog");

        let none = by_capability(&primals, "nonexistent");
        assert!(none.is_empty());
    }

    #[test]
    fn port_for_returns_known_primal_port() {
        assert_eq!(port_for("beardog"), 9100);
        assert_eq!(port_for("biomeos"), 9800);
        assert_eq!(port_for("sweetgrass"), 9850);
    }

    #[test]
    fn port_for_returns_zero_for_unknown() {
        assert_eq!(port_for("nonexistent"), 0);
    }

    #[test]
    fn resolve_primals_all_have_names_and_ports() {
        let primals = resolve_primals("192.0.2.1");
        for p in &primals {
            assert!(!p.name.is_empty(), "primal has empty name");
            assert!(p.port > 1024, "{} port {} too low", p.name, p.port);
        }
    }

    #[test]
    fn shared_registry_has_14_primals() {
        assert_eq!(nucleus_primals::PRIMALS.len(), 14);
    }

    #[test]
    fn resolve_uses_shared_registry_defaults() {
        let primals = resolve_primals("192.0.2.1");
        for def in nucleus_primals::PRIMALS {
            let resolved = primals.iter().find(|p| p.name == def.slug);
            assert!(
                resolved.is_some(),
                "shared registry slug '{}' missing from resolve output",
                def.slug
            );
            assert_eq!(
                resolved.unwrap().port,
                def.default_port,
                "{} port mismatch with shared registry",
                def.slug
            );
        }
    }

    #[test]
    fn resolved_primal_debug_format() {
        let p = ResolvedPrimal {
            name: "test".to_string(),
            port: 1234,
            source: DiscoverySource::Discovered,
            capabilities: vec![],
            live: false,
        };
        let debug = format!("{p:?}");
        assert!(debug.contains("test"));
        assert!(debug.contains("1234"));
    }
}
