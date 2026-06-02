# projectNUCLEUS Wave 64 Deep Debt Execution — May 30, 2026

**From**: projectNUCLEUS (ironGate)
**To**: primalSpring, cellMembrane, all spring teams
**Scope**: Rust coverage expansion, transport ungating, shell hardcoding evolution, debris cleanup

---

## Wave 64a: Rust Codebase

| Metric | Wave 58 | Wave 64 | Delta |
|--------|---------|---------|-------|
| Rust tests | 164 | **234** | +70 |
| darkforest tests | 127 | **140** | +13 |
| tunnelKeeper tests | 36 | **48** | +12 |
| nucleus-deploy tests | — | **46** | — |
| Unsafe blocks | 0 | 0 | — |
| Files >800L | 0 | 0 | — |
| Production mocks | 0 | 0 | — |
| Clippy warnings | 0 | 0 | — |
| C FFI deps | 0 | 0 | — |
| `unwrap()` in prod | 0 | 0 | — |

### Key Evolutions

1. **Transport module ungated**: `transport.rs` was behind `#[cfg(feature = "songbird-transport")]` — 5+ tests never ran during normal `cargo test`. Feature gate removed from both module declaration and test block. Cloudflare transport tests now run every build.

2. **Last sync load on async executor**: `health::run()` called `TunnelConfig::load()` (blocking) on the Tokio executor. Evolved to `load_async()`.

3. **Pentest coverage**: +12 tests in `compute.rs` and `readonly.rs` — user isolation invariants, port uniqueness across primals, constant sanity, gate_home resolution, CheckBuilder pattern validation.

4. **Transport coverage**: +9 tests — error variant display, field roundtrip, none latency, transport name accessor.

5. **Dependency audit**: All deps confirmed pure Rust. Zero C FFI. reqwest uses rustls. Crypto is chacha20poly1305 + ed25519-dalek. YAML is serde-saphyr. No evolution needed.

---

## Wave 64b: Shell Hardcoding Evolution

14 deploy scripts evolved from hardcoded IPs/hostnames to `nucleus_config.sh` variables:

| Script | Hardcoded → Variable |
|--------|---------------------|
| `security_validation.sh` | `TARGET_HOST` + 7 probes → `NUCLEUS_BIND_ADDRESS` |
| `provenance_pipeline.sh` | 8 nc/curl probes → `NUCLEUS_BIND_ADDRESS` |
| `gate_provision.sh` | Cloudflared config template → `LAB_HOSTNAME`/`GIT_HOSTNAME` |
| `deploy_knot_dns.sh` | Zone template → `ZONE_DOMAIN`, GH Pages → `GHPAGES_A_RECORDS` |
| `external_validation.sh` | `JUPYTERHUB_URL` + 5 probes → `NUCLEUS_BIND_ADDRESS` |
| `gate_watchdog.sh` | Display messages → hostname vars |
| `signal_executor.sh` | squirrel/biomeos URLs → `NUCLEUS_BIND_ADDRESS` |
| `deploy_graph.sh` | Health probes + bind_address default → config |
| `deploy_beardog_tls_shadow.sh` | Listen addr + CF baseline → vars |
| `gate_switch.sh` | Verification probe → `LAB_URL` |
| `switch_to_static_observer.sh` | Observer probe → `NUCLEUS_BIND_ADDRESS` |
| `deploy_health_check.sh` | Health probe → `NUCLEUS_BIND_ADDRESS` |
| `publish_sporeprint.sh` | NestGate probe → `NUCLEUS_BIND_ADDRESS` |
| `tier_enforcement_test.sh` | BearDog probe → `NUCLEUS_BIND_ADDRESS` |

Remaining `127.0.0.1` references are exclusively:
- Config defaults in `nucleus_config.sh` (SSOT)
- `${VAR:-default}` env-var-overridable patterns
- VPS SSH context (remote commands correctly use localhost on VPS)
- Security grep filter patterns (`grep -v` for finding external listeners)

---

## Wave 64c: Cleanup

- **cargo clean**: 1.46 GB freed (darkforest 259 MB + tunnelKeeper 1.2 GB)
- **__pycache__**: 8 `.pyc` files removed
- **One-off report**: `membrane-provenance-20260522-102535` moved to `validation/archive/`
- **6 pre-Wave 64 handoffs**: archived to central `infra/wateringHole/handoffs/archive/`
- **Root docs synced**: README.md, PHASES.md, EVOLUTION_GAPS.md, SECURITY_VALIDATION.md — all at 234 tests

---

## Current Codebase Health

| Check | Status |
|-------|--------|
| `cargo clippy --all-features -- -D warnings` | **CLEAN** (both crates) |
| `cargo fmt --check` | **CLEAN** (both crates) |
| `cargo test` | **234 PASS**, 1 ignored (slow DNS) |
| `bash -n` (14 modified scripts) | **ALL PASS** |
| TODO/FIXME/HACK in `.rs` | **0** |
| TODO/FIXME in `.sh` | **0** |
| Files >800L | **0** (max 599L health.rs) |
| `#![forbid(unsafe_code)]` | Both crates |
| Production `unwrap()` | **0** (all in `#[cfg(test)]`) |
| Production mocks | **0** |

---

## For primalSpring

No upstream gaps. All Wave 63 tasks completed (reported in WAVE63_RESPONSE). Wave 64 is internal deep debt — no external dependencies or blockers.

**Remaining evolution trajectory** (tracked, not blocking — Wave 68 status):
- Core deploy pipeline bash → Rust **COMPLETE** (`nucleus-deploy`: security, provenance, deploy, spore, telemetry, summary, verify, provision, dns)
- Temporal/cascade logic **in Rust** (nucleus-deploy modules)
- Divergence policy **explicit** (documented in deploy specs)
- CI sovereignty (GitHub Actions → Forgejo Actions, glacial gate)
- NS registrar cutover (knot-dns deployed, external registrar action)

---

*projectNUCLEUS at Wave 68. 234 Rust tests (darkforest 140, tunnelKeeper 48, nucleus-deploy 46), zero deep debt, 14 scripts evolved. Clean codebase. Ready for primalSpring audit.*
