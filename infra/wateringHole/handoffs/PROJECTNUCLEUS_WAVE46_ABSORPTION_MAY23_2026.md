# projectNUCLEUS — Wave 46 Absorption Handoff

**Date**: 2026-05-23
**Upstream**: primalSpring v0.9.27 (Wave 46)
**Product**: projectNUCLEUS (gardens/projectNUCLEUS)
**Owner**: ironGate

## What was absorbed

### From primalSpring v0.9.27 (Wave 46)

1. **Registry 445→458 methods** — updated across 6 docs (EVOLUTION_GAPS, LIVE_SCIENCE_API, README, PHASES, baseCamp, whitePaper)
2. **Typed error awareness** — `DispatchError`, `IonicProtocolError`, `PhasedIpcError` documented as upstream patterns. darkforest/tunnelKeeper already use thiserror-derived typed errors
3. **env_keys alignment** — canonical constants documented (`MATRIX_CELL`, `REMOTE_GATE_HOST`, `PRIMAL_TRANSPORT`, `DEPLOY_ARCH`). NUCLEUS `nucleus_config.sh` uses compatible patterns
4. **Deploy graphs 12/12 `secure_by_default`** — 5 fragments hardened (tower_atomic, node_atomic, nest_atomic, nucleus, rootpulse_commit)
5. **49 scenarios / 44-cell deployment matrix** — referenced in EVOLUTION_GAPS; `flockgate` cell identified as `nucleus_status = not_deployed`
6. **TEAM_OWNERSHIP_MATRIX v1.1 boundaries** — projectNUCLEUS retains deploy/, gates/, darkforest, Forgejo, genomeBin; cellMembrane owns VPS provisioning

### From Wave 38 (Nest Atomic deployment, May 22)

1. **CM-1**: Nest Atomic deployed to VPS (NestGate v2.1.0, rhizoCrypt v0.14.0, loamSpine v0.9.16, sweetGrass v0.7.34)
2. **CM-2**: Provenance trio E2E verified (10/10 PASS)
3. **CM-4**: darkforest membrane 21 PASS, 0 FAIL, 1 SKIP
4. **H2-17**: knot-dns deployed on VPS (DNSSEC ECDSAP256SHA256)
5. **Shadow orchestrator**: 6 PASS, 0 FAIL, 0 SKIP — all 5 tracks operational

### Deep debt (May 22)

1. **`serde_yaml` → `yaml_serde 0.10`** — pure Rust YAML parser (eliminates unsafe-libyaml C binding)
2. **`deny.toml` created** for both Rust crates — bans openssl, native-tls; ring allowed as transitive only
3. **darkforest discovery suite** — `DISC-01` capability-based primal resolution wired into binary
4. **55 Rust tests PASS** (34 darkforest + 21 tunnelKeeper), clippy pedantic+nursery zero warnings

## Current state

| Metric | Value |
|--------|-------|
| Registry | 458 methods (Wave 46) |
| Rust tests | 55 PASS (darkforest 34, tunnelKeeper 21) |
| Bash checks | 267 PASS (5-layer security) |
| Membrane checks | 21 PASS, 0 FAIL, 1 SKIP |
| Shadow tracks | 6 PASS, 0 FAIL, 0 SKIP (S1-S5 + DNS) |
| VPS services | 11 services, 7 primals (Nest Atomic) |
| Deploy graphs | 12/12 `secure_by_default` |
| Unsafe code | Zero (`#![forbid(unsafe_code)]`) |
| Clippy warnings | Zero (pedantic + nursery) |
| TODO/FIXME | Zero |

## Gaps for upstream attention

### For primalSpring

1. **toadStool branch mismatch** — `git pull --ff-only` fails: configured to merge `refs/heads/master` but remote uses `main`. Needs branch rename or upstream config fix
2. **DOWNSTREAM_PATTERN_GUIDE.md** still references 445 methods — needs Wave 46 sync to 458
3. **`deployment_matrix.toml` flockgate cell** — `nucleus_status = not_deployed`. projectNUCLEUS has gate manifest + covalent graph; deploy is blocked on hardware provisioning

### For bearDog

1. **SP-4 BTSP scope** — `publish_sporeprint.sh` blocked: `auth.issue_session` returns fixed scopes without `content.*`. NestGate MethodGate rejects. Need: bearDog auth to support content scope on pipeline tokens

### For cellMembrane

1. **MEM-09** — `b3sum` not installed on VPS; binary integrity check SKIPs
2. **VPS sweetGrass ephemeral ports** — sweetGrass binds to random tarpc ports. darkforest_membrane.sh adapted; may need sweetGrass `--port` flag

## Open projectNUCLEUS gaps (local work, no upstream blocker)

| ID | Gap | Status |
|----|-----|--------|
| H2-02 | Token distribution UX (CLI) | OPEN |
| H2-03 | 7-day dual-auth shadow | OPEN (ops) |
| H2-04 | Cutover criteria | OPEN |
| H2-06 | petalTongue production content | UNBLOCKED |
| H2-08 | 7-day content shadow | OPEN (ops) |
| H2-09 | petalTongue cutover | OPEN |
| H2-18 | NS transfer (registrar action) | OPEN |
| H2-19 | BTSP direct resolution | OPEN |
| H2-20 | unbound recursive resolver | OPEN |
| H3-07 | CompositionContext wiring | UNBLOCKED |
| H3-08 | skunkBat audit in deploy graphs | UNBLOCKED |
| H3-11 | FlockGate cross-WAN deployment | DESIGNED |

## Cleanup performed

- 3 superseded provenance run directories archived
- 5 incomplete security run directories removed (no SECURITY_RESULTS.md)
- `deploy_songbird_relay.sh` + `switch_to_voila_observer.sh` moved to `deploy/legacy/`
- `.gitignore` updated for membrane-provenance, security runs, benchScale reports
- All root docs (README, PHASES, baseCamp, experiments, graphs) updated to current state
- Stale 445→458 method count corrected across all live documentation
