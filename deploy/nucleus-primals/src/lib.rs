// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! Shared primal registry for projectNUCLEUS.
//!
//! Single source of truth for primal slug names, environment variable keys,
//! and default TCP port assignments. Consumed by `nucleus-deploy` and
//! `darkforest` to eliminate triplicated port tables.

/// A primal endpoint definition.
#[derive(Debug, Clone, Copy)]
pub struct PrimalDef {
    /// Lowercase slug used in IPC, deploy graphs, and gate manifests.
    pub slug: &'static str,
    /// Environment variable that overrides the default port.
    pub env_key: &'static str,
    /// Compiled default port (last-resort fallback after env and discovery).
    pub default_port: u16,
}

/// All known primal endpoints in the NUCLEUS ecosystem.
///
/// Order: Tower base → Agent → Nest provenance → Compute → Meta → Symbiotic.
pub const PRIMALS: &[PrimalDef] = &[
    PrimalDef {
        slug: "beardog",
        env_key: "BEARDOG_PORT",
        default_port: 9100,
    },
    PrimalDef {
        slug: "songbird",
        env_key: "SONGBIRD_PORT",
        default_port: 9200,
    },
    PrimalDef {
        slug: "skunkbat",
        env_key: "SKUNKBAT_PORT",
        default_port: 9140,
    },
    PrimalDef {
        slug: "squirrel",
        env_key: "SQUIRREL_PORT",
        default_port: 9300,
    },
    PrimalDef {
        slug: "toadstool",
        env_key: "TOADSTOOL_PORT",
        default_port: 9400,
    },
    PrimalDef {
        slug: "nestgate",
        env_key: "NESTGATE_PORT",
        default_port: 9500,
    },
    PrimalDef {
        slug: "rhizocrypt",
        env_key: "RHIZOCRYPT_PORT",
        default_port: 9601,
    },
    PrimalDef {
        slug: "rhizocrypt-rpc",
        env_key: "RHIZOCRYPT_RPC_PORT",
        default_port: 9602,
    },
    PrimalDef {
        slug: "loamspine",
        env_key: "LOAMSPINE_PORT",
        default_port: 9700,
    },
    PrimalDef {
        slug: "coralreef",
        env_key: "CORALREEF_PORT",
        default_port: 9730,
    },
    PrimalDef {
        slug: "barracuda",
        env_key: "BARRACUDA_PORT",
        default_port: 9740,
    },
    PrimalDef {
        slug: "biomeos",
        env_key: "BIOMEOS_PORT",
        default_port: 9800,
    },
    PrimalDef {
        slug: "sweetgrass",
        env_key: "SWEETGRASS_PORT",
        default_port: 9850,
    },
    PrimalDef {
        slug: "petaltongue",
        env_key: "PETALTONGUE_PORT",
        default_port: 9900,
    },
];

/// Look up a primal by slug. Returns `None` if the slug is unknown.
#[must_use]
pub fn lookup(slug: &str) -> Option<&'static PrimalDef> {
    PRIMALS.iter().find(|p| p.slug == slug)
}

/// Resolve a primal's port: env var override → compiled default.
#[must_use]
pub fn resolve_port(def: &PrimalDef) -> u16 {
    std::env::var(def.env_key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(def.default_port)
}

/// All primal slugs as a sorted slice (useful for iteration/display).
#[must_use]
pub fn all_slugs() -> Vec<&'static str> {
    PRIMALS.iter().map(|p| p.slug).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_14_entries() {
        assert_eq!(PRIMALS.len(), 14);
    }

    #[test]
    fn all_ports_are_unique() {
        let mut ports: Vec<u16> = PRIMALS.iter().map(|p| p.default_port).collect();
        ports.sort_unstable();
        ports.dedup();
        assert_eq!(ports.len(), PRIMALS.len(), "duplicate default ports");
    }

    #[test]
    fn all_slugs_are_unique() {
        let mut slugs: Vec<&str> = all_slugs();
        slugs.sort_unstable();
        slugs.dedup();
        assert_eq!(slugs.len(), PRIMALS.len(), "duplicate slugs");
    }

    #[test]
    fn all_env_keys_are_unique() {
        let mut keys: Vec<&str> = PRIMALS.iter().map(|p| p.env_key).collect();
        keys.sort_unstable();
        keys.dedup();
        assert_eq!(keys.len(), PRIMALS.len(), "duplicate env keys");
    }

    #[test]
    fn lookup_known_slug() {
        let def = lookup("beardog").expect("beardog should exist");
        assert_eq!(def.default_port, 9100);
        assert_eq!(def.env_key, "BEARDOG_PORT");
    }

    #[test]
    fn lookup_unknown_slug() {
        assert!(lookup("nonexistent").is_none());
    }

    #[test]
    fn resolve_port_uses_default_without_env() {
        let def = lookup("songbird").expect("songbird should exist");
        assert_eq!(resolve_port(def), 9200);
    }
}
