# Phase 59 Absorption — Downstream Evolution Handoff

> **Fossil record**: Superseded by Phase 60 enforcement (`REVALIDATION_PHASE60_MAY08_2026.md`)
> and post-interstadial completion. Retained as lineage record.

**Date**: 2026-05-06
**From**: primalSpring v0.9.24 (Phase 59 — Full Ecosystem Convergence)
**For**: projectNUCLEUS team

---

## What Happened Upstream

primalSpring completed a full ecosystem convergence pass. All 13/13 NUCLEUS primals
now have BTSP Phase 3 AEAD, Wire Standard L3 (`protocol` + `transport` on
`capabilities.list`), BufReader post-negotiate audit, and discovery escalation
hierarchy support. Every upstream blurb sent to primal teams has been resolved and
absorbed back into primalSpring. The composition layer has been refactored
(`deploy/validation.rs`, `coordination/probes.rs`), the workspace has zero clippy
warnings across 84 experiments, and 588 tests pass. skunkBat TCP port corrected
(9750→9140). fieldMouse reclassified as deployment class. primalSpring is clean,
composable, and ready for downstream absorption.

## Composition Layer Evolution

- **`DeployGraph`** now carries `security_model`, `args`, `transport`,
  `composition_model`, and `bonding_policy` fields. Deploy TOMLs can declare these
  and primalSpring will validate them structurally.
- **`PrimalDeployProfile`** generates actionable profiles with `discovery_tier`,
  `tcp_fallback_port`, and `env_keys` per primal. Use `deploy_profiles()` instead of
  hardcoding operational knowledge.
- **`validate_deployment_readiness()`** checks structure + binary discovery + env
  vars + bonding consistency in one call.
- **`topological_waves()`** computes Kahn's-algorithm startup ordering from
  `depends_on` edges.
- **`merge_graphs()`** lets you compose base + overlay deploy graphs.

## Discovery — 5-Tier Escalation

`CompositionContext::discover()` is the canonical entry point. Tiers 1–5:

| Tier | Mechanism | When to use |
|------|-----------|-------------|
| **1** | Songbird `ipc.resolve` | Full NUCLEUS, cross-gate, transport-agnostic |
| **2** | biomeOS Neural API `capability.discover` | Local orchestration |
| **3** | UDS filesystem convention (`primal-family.sock`) | Local machine, no orchestrator |
| **4** | Socket registry / primal manifests | Self-registered primals |
| **5** | TCP probing on well-known ports | Containers, no UDS |

Compositions should prefer capability-based discovery (`by_capability` field on graph
nodes) over identity-based discovery (`name` field).

## BTSP Phase 3 — Encrypted by Default

All 13 primals implement `btsp.negotiate` with ChaCha20-Poly1305 AEAD. Compositions
get encrypted channels automatically when BearDog is present. Graph nodes with
`security_model = "btsp"` are validated against `bonding_policy` at the graph level.

## Wire Standard L3 — Universal

Every primal's `capabilities.list` now returns `protocol` and `transport` fields.

## TCP Fallback Ports (Tier 5)

| Primal | Port | Primal | Port |
|--------|------|--------|------|
| BearDog | 9100 | loamSpine | 9700 |
| Songbird | 9200 | coralReef | 9730 |
| Squirrel | 9300 | barraCuda | 9740 |
| toadStool | 9400 | skunkBat | **9140** |
| NestGate | 9500 | biomeOS | 9800 |
| rhizoCrypt | 9601 | sweetGrass | 9850 |
| petalTongue | 9900 | | |

## Known Issues

| Issue | Workaround |
|-------|-----------|
| BearDog requires `BEARDOG_FAMILY_SEED` env | Export before launch |
| NestGate refuses without JWT | `export NESTGATE_JWT_SECRET="$(head -c 32 /dev/urandom \| base64)"` |
| sweetGrass requires BTSP handshake for TCP | Use UDS, or complete BTSP negotiation first |
| BearDog resets connection without BTSP | Expected — BTSP handshake required for crypto calls |
| Songbird/petalTongue speak HTTP on UDS | Use `is_protocol_error()` or classify as SKIP |
| fieldMouse is NOT a primal | Deployment class (biomeOS chimeras for edge/IoT) |

## Key References

| Topic | Location |
|-------|----------|
| Gap registry | `primalSpring/docs/PRIMAL_GAPS.md` |
| Composition handoff (Phase 58) | `primalSpring/wateringHole/PHASE58_COMPOSITION_HANDOFF_MAY03_2026.md` |
| Depot workflow | `primalSpring/wateringHole/PLASMINBIN_DEPOT_PATTERN.md` |
| Crypto consumption hierarchy | `primalSpring/wateringHole/CRYPTO_CONSUMPTION_HIERARCHY.md` |
| Ecosystem contracts | `primalSpring/wateringHole/UPSTREAM_CROSSTALK_AND_DOWNSTREAM_ABSORPTION.md` |
| Deploy graphs | `primalSpring/graphs/` (71 TOMLs) |
| Binary fetch script | `primalSpring/tools/fetch_primals.sh` |

## Absorption Actions Taken (projectNUCLEUS)

- [x] Fixed TCP port misassignments in deploy.sh (NestGate 9300→9500, barraCuda 9500→9740)
- [x] Updated deploy.sh compositions to include all primals per atomic
- [x] Updated irongate.toml port table
- [x] Added tcp_fallback_port fields to deploy graph nodes
- [x] Added validate_deployment_readiness() pre-launch check
- [x] Updated gate_manifest.toml with live active-gate entry
- [x] Updated README.md for Phase 59 convergence
- [x] Updated graphs/README.md
- [x] Added BIND_ADDRESS variable to deploy.sh for primal bind control
- [x] Phase 2a external validation pipeline (external_validation.sh)
- [x] ABG tiered access (abg_accounts.sh, jupyterhub_config.py pre_spawn_hook)
- [x] Three-layer security pen testing (security_validation.sh)
- [x] Security handback for upstream primal teams
- [x] Notebook elevation (wetspring-validation-viz.ipynb)

## Phase 59 Security Convergence (absorbed 2026-05-06)

All 5 security gaps from our Phase 2a pen test resolved upstream:

| PG | Resolution |
|----|-----------|
| PG-55 | All 13 primals default `127.0.0.1`. Deploy scripts drop explicit `--bind 0.0.0.0` overrides. |
| PG-56 | NestGate BTSP method-level auth gating. 10-method exempt whitelist. |
| PG-57 | skunkBat multi-dimensional anomaly detection. 12 normal + 7 attack patterns seeded. |
| PG-58 | Songbird `--bind` for HTTP, `--listen` for IPC (separate concerns). |
| PG-59 | sweetGrass `--http-address` and `--port` both accept `host:port`. |

- [x] Updated SECURITY_HANDBACK_MAY06_2026.md — all PG items marked resolved
- [x] Updated VALIDATION_RESULTS.md — bind status → 13/13 default localhost
- [x] Updated PHASES.md — security posture reflects zero open gaps
- [x] Updated README.md — security status line

**Ecosystem state**: 13/13 BTSP Phase 3 FULL AEAD, 13/13 default `127.0.0.1` bind, zero open security gaps, 5-tier discovery escalation hierarchy live, 85 experiments, 661 tests, 74 deploy graphs.

---

**The stack is encrypted. The wire standard is universal. The discovery hierarchy is
live. The composition layer is validated. Security is converged. Pull, validate, absorb, deploy.**
