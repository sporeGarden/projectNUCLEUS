# projectNUCLEUS Wave 56 Absorption — May 27, 2026

**From**: projectNUCLEUS (ironGate)
**To**: primalSpring, cellMembrane, biomeOS, all spring teams
**Scope**: VPS deployment standard (`--uds-only`), cell graph awareness, deploy tooling modernization

---

## What Was Absorbed

### 1. `--uds-only` VPS Standard (MEDIUM priority, DONE)

Deploy tooling now supports the Wave 56 zero-TCP-port VPS standard:

| File | Change |
|------|--------|
| `deploy.sh` | Added `--uds-only` CLI flag, transport banner display |
| `deploy_primal_start.sh` | All 13 primal cases refactored: conditional port args via `(( PORT > 0 ))` array pattern |
| `deploy_graph.sh` | `start_primal_from_graph()` respects `UDS_ONLY` override, replaced inline nohup with `args` array |
| `deploy_health_check.sh` | New `socket_health_check()` — UDS socket existence probe replaces TCP health in `--uds-only` mode |
| `nucleus_config.sh` | Documented as desktop/debug defaults; VPS uses `--uds-only` |

**Usage**: `bash deploy.sh --composition nest --uds-only`

The deploy scripts now follow the same pattern as `nucleus_launcher --uds-only`:
- When `UDS_ONLY=true`, all `*_PORT` variables are zeroed
- Port args (`--port`, `--listen`, `--bind`, `--rpc-bind`) are omitted when port=0
- Socket args (`--socket`, `--socket-only`) are always passed
- Health checks use socket file existence instead of TCP probes

### 2. Cell Graph Awareness

primalSpring cell graphs reviewed and consumed:
- `cells_manifest.toml` v1.1.0 — 9 cells indexed, `vps_standard` field
- 6 VPS-ready spring cells (`spawn=false`, `transport = "uds_only"`)
- 3 desktop-only cells (nucleus_desktop, ludospring, esotericwebb)
- `primal_launch_profiles.toml` — per-primal CLI/env wiring reference
- `seed_fingerprints.toml` — BLAKE3 tier 0 provenance for 12 primals

### 3. Validation

| Check | Result |
|-------|--------|
| darkforest unit tests | **44 PASS** (0 fail) |
| tunnelKeeper unit tests | **21 PASS** (0 fail) |
| Total Rust tests | **65 PASS** |
| Clippy pedantic+nursery | zero warnings |
| Deploy scripts | `bash -n` syntax-valid |

### 4. Documentation Updates

All docs updated from Wave 55 → Wave 56:

| Document | Key Update |
|----------|-----------|
| `README.md` | Wave 56 banner, `--uds-only` in Quick Start, deployment standard bullet |
| `PHASES.md` | Wave 56 context, `--uds-only`, cell graph tagging, primalSpring test counts |
| `specs/EVOLUTION_GAPS.md` | Wave 56 changelog entry (8-point), tier 2 API updated |
| `experiments/README.md` | Wave 56 wave ref, `--uds-only` noted |
| `whitePaper/baseCamp/README.md` | Wave 56 context, VPS deployment standard |
| `specs/LIVE_SCIENCE_API.md` | Wave 56 registry, cell graph tagging |
| `gates/irongate.toml` | `primalspring_wave = 56`, `vps_standard = "uds-only"`, `cell_graphs_consumed = true` |
| `graphs/sovereignty_shadow.toml` | Wave 38/56 header |

---

## What Changed Upstream (Wave 56 artifacts consumed)

| Artifact | Status |
|----------|--------|
| `nucleus_launcher --uds-only` | Consumed via local deploy scripts |
| Cell graph `vps_standard` tagging | Consumed — 6 VPS-ready, 3 desktop |
| `primalspring checksums` subcommand | Replaces `check_method_coverage.sh` etc. |
| `primalspring registry` subcommand | Replaces `check_method_gate.sh` etc. |
| 12 primordial scripts archived | Acknowledged — `fossilRecord/scripts_wave55b_may2026/` |
| `env_keys.rs` centralization | Already consumed Wave 55 |
| 797 lib tests, 56 scenarios | Upstream validation reference |

---

## What projectNUCLEUS Does NOT Need (per audit)

- No shell script changes for spring consumption
- No new primalSpring dependency
- No TCP port allocation for VPS path
- No blocking items — deep debt resolved

---

## VPS Deployment Flow (Wave 56 standard)

```
1. deploy.sh --uds-only --composition nest    → NUCLEUS base (13 primals, UDS-only)
2. biomeos deploy graphs/cells/{spring}_cell.toml  → spring overlay (spawn=false)
3. Spring uses CompositionContext::from_live_discovery()  → UDS tiers 2-4
```

---

## NC-1→NC-5 Status (unchanged from Wave 55)

| Gap | Status | Blocker |
|-----|--------|---------|
| NC-1 (spore gateway) | WIRED | biomeOS NC-1.4 |
| NC-2 (multi-gate mesh) | IN PROGRESS | southGate ops stabilization |
| NC-3 (cellMembrane sovereignty) | ADVANCING | Forgejo + NS cutover remaining |
| NC-4 (spring NUCLEUS depth) | MIXED | east/iron OK, south/biome partial |
| NC-5 (lithoSpore emission) | GATED | on NC-1 live deploy |

---

## Notes for Upstream Teams

### cellMembrane
- `deploy_membrane.sh` should pass `--uds-only` to `nucleus_launcher` (HIGH priority per audit)
- `biomeos deploy graphs/cells/{spring}_cell.toml` for spring overlays (HIGH)
- Socket-based health checks available if you source `deploy_health_check.sh` with `UDS_ONLY=true`

### primalSpring
- Wave 56 artifacts fully consumed on projectNUCLEUS side
- `primalspring checksums` + `primalspring registry` acknowledged — no shell validation scripts remain in our critical path
- CHANGELOG.md Wave 56 entry still pending upstream

### biomeOS
- `biomeos deploy` CLI surface reviewed — graph-driven deploy operational
- NC-1.4 (`nucleus_ingest.rs` → `pseudospore-core`) still the stadial blocker

### Spring Teams
- Cell graphs consumed but not yet tested via live `biomeos deploy` (gated on NC-1)
- `CompositionContext::from_live_discovery()` is the target runtime path

---

*projectNUCLEUS at Wave 56. Deploy tooling aligned. Stadial gate readiness tracked.*
