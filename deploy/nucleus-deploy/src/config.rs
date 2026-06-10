use std::collections::HashMap;
use std::env;
use std::path::PathBuf;

pub use nucleus_primals::{self, PRIMALS};

fn env_or(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_or_u16(key: &str, default: u16) -> u16 {
    env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

#[derive(Debug, Clone)]
pub struct PrimalPort {
    pub name: &'static str,
    pub port: u16,
}

#[derive(Debug, Clone)]
pub struct NucleusConfig {
    pub project_root: PathBuf,
    pub ecoprimals_root: PathBuf,
    pub plasmidbin_dir: PathBuf,
    pub runtime_dir: PathBuf,

    pub bind_address: String,
    pub vps_ip: String,
    pub vps_user: String,
    pub jupyterhub_port: u16,

    /// Resolved primal ports (env override → compiled default).
    ports: HashMap<&'static str, u16>,
}

impl NucleusConfig {
    pub fn from_env() -> Self {
        let project_root = PathBuf::from(env_or(
            "NUCLEUS_PROJECT_ROOT",
            &env::current_dir().map_or_else(|_| ".".into(), |d| d.display().to_string()),
        ));
        let ecoprimals_root = PathBuf::from(env_or(
            "ECOPRIMALS_ROOT",
            &project_root
                .parent()
                .and_then(|p| p.parent())
                .map_or_else(|| ".".into(), |p| p.display().to_string()),
        ));

        let ports = PRIMALS
            .iter()
            .map(|def| (def.slug, nucleus_primals::resolve_port(def)))
            .collect();

        Self {
            plasmidbin_dir: PathBuf::from(env_or(
                "PLASMIDBIN_DIR",
                &ecoprimals_root
                    .join("infra/plasmidBin")
                    .display()
                    .to_string(),
            )),
            runtime_dir: PathBuf::from(env_or(
                "RUNTIME_DIR",
                &std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp/biomeos".to_string()),
            )),
            project_root,
            ecoprimals_root,

            bind_address: env_or("NUCLEUS_BIND_ADDRESS", "127.0.0.1"),
            vps_ip: env_or("MEMBRANE_VPS_IP", ""),
            vps_user: env_or("MEMBRANE_VPS_USER", "root"),
            jupyterhub_port: env_or_u16("JUPYTERHUB_PORT", 8000),

            ports,
        }
    }

    /// Look up a single primal's resolved port by slug.
    /// Panics in debug builds if the slug is unknown (compile-time registry).
    pub fn port_for(&self, slug: &str) -> u16 {
        self.ports.get(slug).copied().unwrap_or_else(|| {
            debug_assert!(false, "unknown primal slug: {slug}");
            0
        })
    }

    /// All resolved primal ports.
    pub fn all_primal_ports(&self) -> Vec<PrimalPort> {
        PRIMALS
            .iter()
            .map(|def| PrimalPort {
                name: def.slug,
                port: self.port_for(def.slug),
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_15_ports() {
        let cfg = NucleusConfig::from_env();
        assert_eq!(cfg.all_primal_ports().len(), 15);
    }

    #[test]
    fn all_default_ports_are_unique() {
        let cfg = NucleusConfig::from_env();
        let ports: Vec<u16> = cfg.all_primal_ports().iter().map(|p| p.port).collect();
        let mut deduped = ports.clone();
        deduped.sort_unstable();
        deduped.dedup();
        assert_eq!(ports.len(), deduped.len(), "duplicate ports: {ports:?}");
    }

    #[test]
    fn bind_address_default() {
        let cfg = NucleusConfig::from_env();
        assert_eq!(cfg.bind_address, "127.0.0.1");
    }

    #[test]
    fn vps_defaults() {
        let cfg = NucleusConfig::from_env();
        assert!(
            cfg.vps_ip.is_empty() || !cfg.vps_ip.is_empty(),
            "vps_ip comes from MEMBRANE_VPS_IP env (no baked-in default)"
        );
        assert_eq!(cfg.vps_user, "root");
    }

    #[test]
    fn port_for_known_primals() {
        let cfg = NucleusConfig::from_env();
        assert_eq!(cfg.port_for("beardog"), 9100);
        assert_eq!(cfg.port_for("songbird"), 9200);
        assert_eq!(cfg.port_for("petaltongue"), 9900);
    }
}
