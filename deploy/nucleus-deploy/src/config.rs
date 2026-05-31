use std::env;
use std::path::PathBuf;

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
    pub jupyterhub_port: u16,

    pub beardog_port: u16,
    pub songbird_port: u16,
    pub squirrel_port: u16,
    pub toadstool_port: u16,
    pub nestgate_port: u16,
    pub rhizocrypt_port: u16,
    pub rhizocrypt_rpc_port: u16,
    pub loamspine_port: u16,
    pub coralreef_port: u16,
    pub barracuda_port: u16,
    pub biomeos_port: u16,
    pub sweetgrass_port: u16,
    pub petaltongue_port: u16,
    pub skunkbat_port: u16,
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

        Self {
            plasmidbin_dir: PathBuf::from(env_or(
                "PLASMIDBIN_DIR",
                &ecoprimals_root
                    .join("infra/plasmidBin")
                    .display()
                    .to_string(),
            )),
            runtime_dir: PathBuf::from(env_or("RUNTIME_DIR", "/tmp/biomeos")),
            project_root,
            ecoprimals_root,

            bind_address: env_or("NUCLEUS_BIND_ADDRESS", "127.0.0.1"),
            jupyterhub_port: env_or_u16("JUPYTERHUB_PORT", 8000),

            beardog_port: env_or_u16("BEARDOG_PORT", 9100),
            songbird_port: env_or_u16("SONGBIRD_PORT", 9200),
            squirrel_port: env_or_u16("SQUIRREL_PORT", 9300),
            toadstool_port: env_or_u16("TOADSTOOL_PORT", 9400),
            nestgate_port: env_or_u16("NESTGATE_PORT", 9500),
            rhizocrypt_port: env_or_u16("RHIZOCRYPT_PORT", 9601),
            rhizocrypt_rpc_port: env_or_u16("RHIZOCRYPT_RPC_PORT", 9602),
            loamspine_port: env_or_u16("LOAMSPINE_PORT", 9700),
            coralreef_port: env_or_u16("CORALREEF_PORT", 9730),
            barracuda_port: env_or_u16("BARRACUDA_PORT", 9740),
            biomeos_port: env_or_u16("BIOMEOS_PORT", 9800),
            sweetgrass_port: env_or_u16("SWEETGRASS_PORT", 9850),
            petaltongue_port: env_or_u16("PETALTONGUE_PORT", 9900),
            skunkbat_port: env_or_u16("SKUNKBAT_PORT", 9140),
        }
    }

    pub fn all_primal_ports(&self) -> Vec<PrimalPort> {
        vec![
            PrimalPort {
                name: "beardog",
                port: self.beardog_port,
            },
            PrimalPort {
                name: "songbird",
                port: self.songbird_port,
            },
            PrimalPort {
                name: "squirrel",
                port: self.squirrel_port,
            },
            PrimalPort {
                name: "toadstool",
                port: self.toadstool_port,
            },
            PrimalPort {
                name: "nestgate",
                port: self.nestgate_port,
            },
            PrimalPort {
                name: "rhizocrypt",
                port: self.rhizocrypt_port,
            },
            PrimalPort {
                name: "rhizocrypt-rpc",
                port: self.rhizocrypt_rpc_port,
            },
            PrimalPort {
                name: "loamspine",
                port: self.loamspine_port,
            },
            PrimalPort {
                name: "coralreef",
                port: self.coralreef_port,
            },
            PrimalPort {
                name: "barracuda",
                port: self.barracuda_port,
            },
            PrimalPort {
                name: "biomeos",
                port: self.biomeos_port,
            },
            PrimalPort {
                name: "sweetgrass",
                port: self.sweetgrass_port,
            },
            PrimalPort {
                name: "petaltongue",
                port: self.petaltongue_port,
            },
            PrimalPort {
                name: "skunkbat",
                port: self.skunkbat_port,
            },
        ]
    }

    #[expect(
        dead_code,
        reason = "used by provenance and deploy subcommands (Wave 65)"
    )]
    pub fn primal_port_map(&self) -> std::collections::HashMap<&'static str, u16> {
        self.all_primal_ports()
            .into_iter()
            .map(|pp| (pp.name, pp.port))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_all_14_ports() {
        let cfg = NucleusConfig::from_env();
        assert_eq!(cfg.all_primal_ports().len(), 14);
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
}
