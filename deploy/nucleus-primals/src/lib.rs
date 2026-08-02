// SPDX-License-Identifier: AGPL-3.0-or-later
#![forbid(unsafe_code)]

//! Shared primal registry for projectNUCLEUS.
//!
//! Single source of truth for primal slug names, transport characteristics,
//! and default TCP port assignments. Consumed by `nucleus-deploy` and
//! `darkforest` to eliminate triplicated port tables.
//!
//! Port resolution hierarchy (first match wins):
//!   1. Gate TOML `[ports]` section
//!   2. Environment variable (`{SLUG}_PORT`)
//!   3. Compiled default (this registry)

/// JSON-RPC framing style — determines how health probes reach the primal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Framing {
    /// Newline-delimited JSON-RPC (most primals). Probe via `nc`.
    Newline,
    /// HTTP POST JSON-RPC (songbird, loamspine). Probe via `curl`.
    Http,
}

/// Transport capabilities advertised by a primal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transport {
    Uds,
    Tcp,
    Http,
}

/// A primal endpoint definition with transport metadata.
#[derive(Debug, Clone)]
pub struct PrimalDef {
    /// Lowercase slug used in IPC, deploy graphs, and gate manifests.
    pub slug: &'static str,
    /// Environment variable that overrides the default port.
    pub env_key: &'static str,
    /// Compiled default port (last-resort fallback after env and gate TOML).
    pub default_port: u16,
    /// JSON-RPC framing style for health probes.
    pub framing: Framing,
    /// Supported transports (ordered by preference).
    pub transports: &'static [Transport],
    /// Whether BTSP negotiation is required before IPC.
    pub btsp_required: bool,
}

/// All known primal endpoints in the NUCLEUS ecosystem.
///
/// Order: Tower base → Defense → Agent → Compute → Nest provenance → Meta → Viz.
pub const PRIMALS: &[PrimalDef] = &[
    PrimalDef {
        slug: "beardog",
        env_key: "BEARDOG_PORT",
        default_port: 9100,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: false,
    },
    PrimalDef {
        slug: "songbird",
        env_key: "SONGBIRD_PORT",
        default_port: 9200,
        framing: Framing::Http,
        transports: &[Transport::Uds, Transport::Tcp, Transport::Http],
        btsp_required: true,
    },
    PrimalDef {
        slug: "skunkbat",
        env_key: "SKUNKBAT_PORT",
        default_port: 9140,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "squirrel",
        env_key: "SQUIRREL_PORT",
        default_port: 9300,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "toadstool",
        env_key: "TOADSTOOL_PORT",
        default_port: 9400,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "nestgate",
        env_key: "NESTGATE_PORT",
        default_port: 9500,
        framing: Framing::Newline,
        transports: &[Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "rhizocrypt",
        env_key: "RHIZOCRYPT_PORT",
        default_port: 9601,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "rhizocrypt-rpc",
        env_key: "RHIZOCRYPT_RPC_PORT",
        default_port: 9602,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "loamspine",
        env_key: "LOAMSPINE_PORT",
        default_port: 9700,
        framing: Framing::Http,
        transports: &[Transport::Uds, Transport::Tcp, Transport::Http],
        btsp_required: true,
    },
    PrimalDef {
        slug: "coralreef",
        env_key: "CORALREEF_PORT",
        default_port: 9730,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "barracuda",
        env_key: "BARRACUDA_PORT",
        default_port: 9740,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "biomeos",
        env_key: "BIOMEOS_PORT",
        default_port: 9800,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "sweetgrass",
        env_key: "SWEETGRASS_PORT",
        default_port: 9850,
        framing: Framing::Newline,
        transports: &[Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "petaltongue",
        env_key: "PETALTONGUE_PORT",
        default_port: 9900,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
    },
    PrimalDef {
        slug: "primalspring",
        env_key: "PRIMALSPRING_PORT",
        default_port: 9990,
        framing: Framing::Newline,
        transports: &[Transport::Uds, Transport::Tcp],
        btsp_required: true,
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

/// All primal slugs (useful for iteration/display).
#[must_use]
pub fn all_slugs() -> Vec<&'static str> {
    PRIMALS.iter().map(|p| p.slug).collect()
}

/// All deployable primal slugs (excludes aliases like rhizocrypt-rpc
/// and validation tools like primalspring).
#[must_use]
pub fn deployable_slugs() -> Vec<&'static str> {
    COMP_FULL.to_vec()
}

/// Discover the UDS socket path for a primal using the biomeOS runtime
/// directory convention. Checks `XDG_RUNTIME_DIR/biomeos/` first, then `/tmp/`.
#[must_use]
pub fn discover_socket(slug: &str) -> Option<std::path::PathBuf> {
    let xdg = std::env::var("XDG_RUNTIME_DIR").ok();
    [
        xdg.as_deref().map(|d| {
            std::path::PathBuf::from(d)
                .join("biomeos")
                .join(format!("{slug}.sock"))
        }),
        Some(std::path::PathBuf::from(format!("/tmp/{slug}.sock"))),
    ]
    .into_iter()
    .flatten()
    .find(|p| p.exists())
}

/// Returns true if the primal uses HTTP framing for health probes.
#[must_use]
pub fn uses_http_framing(slug: &str) -> bool {
    lookup(slug).is_some_and(|def| def.framing == Framing::Http)
}

// ── Composition constants (atomic groupings) ──────────────────────

/// Tower Atomic (electron): security + discovery + defense.
pub const COMP_TOWER: &[&str] = &["beardog", "songbird", "skunkbat"];

/// Node Atomic (proton): tower + compute trio + defense.
pub const COMP_NODE: &[&str] = &[
    "beardog",
    "songbird",
    "toadstool",
    "barracuda",
    "coralreef",
    "skunkbat",
];

/// Nest Atomic (neutron): tower + storage + provenance trio + defense.
pub const COMP_NEST: &[&str] = &[
    "beardog",
    "songbird",
    "nestgate",
    "rhizocrypt",
    "loamspine",
    "sweetgrass",
    "skunkbat",
];

/// Agent composition: tower + defense + orchestration + agent.
pub const COMP_AGENT: &[&str] = &["beardog", "songbird", "skunkbat", "biomeos", "squirrel"];

/// Full NUCLEUS: all 13 deployable primals (excludes rhizocrypt-rpc alias
/// and primalspring which is a validation spring, not a deployable primal).
pub const COMP_FULL: &[&str] = &[
    "beardog",
    "songbird",
    "toadstool",
    "barracuda",
    "coralreef",
    "nestgate",
    "rhizocrypt",
    "loamspine",
    "sweetgrass",
    "squirrel",
    "skunkbat",
    "biomeos",
    "petaltongue",
];

// ── Infrastructure service defaults ──────────────────────────────

/// `JupyterHub` default port (env: `JUPYTERHUB_PORT`).
pub const JUPYTERHUB_DEFAULT_PORT: u16 = 8000;

/// Observer static server default port (env: `OBSERVER_PORT`).
pub const OBSERVER_DEFAULT_PORT: u16 = 8866;

/// Songbird federation default port (env: `SONGBIRD_FEDERATION_PORT`).
/// Distinct from the songbird registry TCP port (9200).
pub const SONGBIRD_FEDERATION_DEFAULT_PORT: u16 = 7700;

/// `BearDog` TLS shadow default port (env: `BTSP_SHADOW_PORT`).
pub const BTSP_SHADOW_DEFAULT_PORT: u16 = 8443;

/// `RustDesk` hbbs default port (env: `RUSTDESK_HBBS_PORT`).
pub const RUSTDESK_HBBS_DEFAULT_PORT: u16 = 21116;

/// Membrane HTTP default port (env: `MEMBRANE_HTTP_PORT`).
pub const MEMBRANE_HTTP_DEFAULT_PORT: u16 = 80;

/// Forgejo SSH default port (env: `DARKFOREST_FORGE_PORT`).
pub const FORGEJO_SSH_DEFAULT_PORT: u16 = 2222;

/// `WireGuard` default port (env: `DARKFOREST_WG_PORT`).
pub const WIREGUARD_DEFAULT_PORT: u16 = 51820;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_has_15_entries() {
        assert_eq!(PRIMALS.len(), 15);
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
        assert!(!def.btsp_required);
        assert_eq!(def.framing, Framing::Newline);
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

    #[test]
    fn http_framing_primals() {
        assert!(uses_http_framing("songbird"));
        assert!(uses_http_framing("loamspine"));
        assert!(!uses_http_framing("beardog"));
        assert!(!uses_http_framing("biomeos"));
        assert!(!uses_http_framing("nonexistent"));
    }

    #[test]
    fn transport_metadata_populated() {
        for def in PRIMALS {
            assert!(!def.transports.is_empty(), "{} has no transports", def.slug);
        }
    }

    #[test]
    fn beardog_is_btsp_origin() {
        let def = lookup("beardog").expect("beardog");
        assert!(
            !def.btsp_required,
            "beardog is the BTSP origin, not a consumer"
        );
    }

    #[test]
    fn all_non_beardog_require_btsp() {
        for def in PRIMALS {
            if def.slug != "beardog" {
                assert!(def.btsp_required, "{} should require BTSP", def.slug);
            }
        }
    }

    #[test]
    fn deployable_slugs_has_13() {
        assert_eq!(deployable_slugs().len(), 13);
    }

    #[test]
    fn comp_tower_is_subset_of_full() {
        for slug in COMP_TOWER {
            assert!(COMP_FULL.contains(slug), "{slug} not in COMP_FULL");
        }
    }

    #[test]
    fn comp_node_is_subset_of_full() {
        for slug in COMP_NODE {
            assert!(COMP_FULL.contains(slug), "{slug} not in COMP_FULL");
        }
    }

    #[test]
    fn comp_nest_is_subset_of_full() {
        for slug in COMP_NEST {
            assert!(COMP_FULL.contains(slug), "{slug} not in COMP_FULL");
        }
    }

    #[test]
    fn comp_full_has_13_primals() {
        assert_eq!(
            COMP_FULL.len(),
            13,
            "COMP_FULL should list all 13 deployable primals"
        );
    }

    #[test]
    fn all_comp_slugs_exist_in_registry() {
        for comp in [COMP_TOWER, COMP_NODE, COMP_NEST, COMP_AGENT, COMP_FULL] {
            for slug in comp {
                assert!(
                    lookup(slug).is_some(),
                    "composition slug '{slug}' not in PRIMALS registry"
                );
            }
        }
    }

    #[test]
    fn infra_ports_are_non_zero() {
        assert_ne!(JUPYTERHUB_DEFAULT_PORT, 0);
        assert_ne!(OBSERVER_DEFAULT_PORT, 0);
        assert_ne!(SONGBIRD_FEDERATION_DEFAULT_PORT, 0);
        assert_ne!(BTSP_SHADOW_DEFAULT_PORT, 0);
        assert_ne!(RUSTDESK_HBBS_DEFAULT_PORT, 0);
        assert_ne!(MEMBRANE_HTTP_DEFAULT_PORT, 0);
        assert_ne!(FORGEJO_SSH_DEFAULT_PORT, 0);
        assert_ne!(WIREGUARD_DEFAULT_PORT, 0);
    }

    #[test]
    fn infra_ports_dont_collide_with_primal_ports() {
        let primal_ports: Vec<u16> = PRIMALS.iter().map(|p| p.default_port).collect();
        for port in [
            JUPYTERHUB_DEFAULT_PORT,
            OBSERVER_DEFAULT_PORT,
            SONGBIRD_FEDERATION_DEFAULT_PORT,
            BTSP_SHADOW_DEFAULT_PORT,
            RUSTDESK_HBBS_DEFAULT_PORT,
            MEMBRANE_HTTP_DEFAULT_PORT,
            FORGEJO_SSH_DEFAULT_PORT,
            WIREGUARD_DEFAULT_PORT,
        ] {
            assert!(
                !primal_ports.contains(&port),
                "infra port {port} collides with a primal port"
            );
        }
    }
}
