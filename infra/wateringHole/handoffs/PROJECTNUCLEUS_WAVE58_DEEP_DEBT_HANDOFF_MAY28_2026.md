# projectNUCLEUS Wave 58 Deep Debt Execution — May 28, 2026

**From**: projectNUCLEUS (ironGate)
**To**: primalSpring, cellMembrane, biomeOS, all spring teams
**Scope**: Async correctness, discovery transport evolution, coverage climb, dependency reduction, hardcoding cleanup

---

## Current Metrics

| Metric | Before (Wave 56) | After (Wave 58) |
|--------|-------------------|-----------------|
| darkforest unit tests | 44 | **125** |
| tunnelKeeper unit tests | 21 | **41** |
| Total Rust tests | 65 | **166** |
| darkforest coverage (llvm-cov) | 35.56% | **40.77%** |
| tunnelKeeper coverage (llvm-cov) | 51.81% | **52.67%** |
| tunnelKeeper transitive deps | 175 | **173** |
| darkforest transitive deps | 28 | **28** |
| darkforest C dependencies | zero | **zero** |
| TODO/FIXME/HACK in source | zero | **zero** |
| Clippy warnings | zero | **zero** |
| Files >800 LOC | zero | **zero** (max 599) |
| Test runtime (darkforest) | ~68s | **3s** |
| Test runtime (tunnelKeeper) | ~66s | **1s** |

---

## What Was Done

### 1. Async/Blocking Fix (tunnelKeeper, CRITICAL)

`health.rs` ran blocking I/O (`std::process::Command`, `TcpStream::connect_timeout`, `std::fs::read_to_string`) directly on the Tokio async executor. Fixed:

- Wrapped `check_process()`, `check_connectivity()`, `check_dns()`, `check_config()` in a single `tokio::task::spawn_blocking` block
- Migrated `try_cf_api_check()` credential read to `tokio::fs::read_to_string`
- Added `fs` feature to tokio

### 2. Discovery Transport Evolution (darkforest, HIGH)

Discovery was sending HTTP POST JSON-RPC (`send_jsonrpc`) to biomeOS and primals, but all NUCLEUS primals (except loamSpine) use newline-delimited JSON-RPC. Fixed:

- `try_biomeos_discovery()` → `send_jsonrpc_newline()`
- `probe_liveness()` → `send_jsonrpc_newline()`
- `probe_capabilities()` → `send_jsonrpc_newline()`
- `send_jsonrpc_newline` promoted from `#[allow(dead_code)]` to live code

### 3. Silent Error Bugs Fixed (tunnelKeeper, CRITICAL)

- `serde_json::to_string_pretty(&report).unwrap_or_default()` → `?` propagation (silent empty output on serialization failure)
- `filter_map(|r| to_value(r).ok())` → `collect::<Result<_, _>>()?` (silent ingress rule drop)
- `Command::new("date")` → pure `std::time::SystemTime` (subprocess elimination)

### 4. Dependency Reduction (tunnelKeeper)

| Dependency | Action | Reason |
|------------|--------|--------|
| `chrono` | **Removed** | Zero source references |
| tokio `process` feature | **Removed** | No `tokio::process` usage |
| reqwest | **Retained** | 4 HTTP calls (GET/PUT), TLS+async+JSON stack makes replacement marginal |

Remaining C/ASM deps: `ring` + `aws-lc-sys` (rustls TLS backend). Tracked in `deny.toml`.

### 5. Deploy Script Hardcoding Cleanup

| Script | Changes |
|--------|---------|
| `membrane_provenance.sh` | 5 port literals → `$RHIZOCRYPT_RPC_PORT`, `$LOAMSPINE_PORT`, `$SWEETGRASS_PORT`, `$BEARDOG_PORT`, `$TURN_PORT` |
| `deploy_health_check.sh` | Added `nucleus_config.sh` sourcing |
| `switch_to_static_observer.sh` | `8866` → `$OBSERVER_STATIC_PORT` |
| `membrane_telemetry.sh` | `3478` → `$TURN_PORT` |

### 6. Coverage Climb (+101 tests)

New test modules added to previously uncovered areas:

| Module | New tests | Coverage target |
|--------|-----------|-----------------|
| `fuzz.rs` | 6 | Payload construction, JSON validity, binary probes |
| `crypto/protocol.rs` | 7 | Ionic tokens, BTSP probes, JSON-RPC payloads |
| `pentest/external.rs` | 3 | Traversal patterns, admin paths, host injection |
| `pentest/compute.rs` | 2 | Port ranges, RPC probe targets |
| `pentest/readonly.rs` | 2 | User isolation, port resolution |
| `discovery.rs` | 3 | Port resolution, primal roster |
| `net.rs` | 3 | http_method, http_post, sudo_cmd edge cases |
| tunnelKeeper health | 3 | DNS hostname extraction, localhost resolution, fast alternatives |
| tunnelKeeper config/api/crypto | 17 | Route ops, serialization, crypto roundtrips |

Coverage note: pentest/fuzz/crypto-protocol modules are integration-test code that probes live services. Uncovered lines are service-dependent branches executable only against a live VPS. All testable helper logic within those modules is now covered.

### 7. DNS Test Fix

The `dns_health_returns_a_result` test took 65 seconds (3 DNS tools timing out on non-existent domain). Split into:
- `dns_hostname_extracted_from_first_ingress` — instant (no DNS)
- `dns_resolves_localhost` — fast (localhost resolves instantly)
- `dns_health_returns_a_result_slow` — `#[ignore]` (available with `--ignored`)

---

## Build Metrics

| Metric | darkforest | tunnelKeeper |
|--------|-----------|--------------|
| Release build (clean) | 10.2s | 34.4s |
| Binary size | 1.1 MB | 6.5 MB |
| `#[forbid(unsafe_code)]` | Yes | Yes |
| `cargo fmt` | PASS | PASS |
| `cargo clippy --all-features` | PASS (0 warnings) | PASS (0 warnings) |
| `cargo doc --no-deps` | PASS | PASS |

---

## Upstream Notes for primalSpring Audit

### What to review

1. **Transport alignment**: darkforest discovery now uses newline-delimited JSON-RPC (matching primal wire format). Verify primal.list/capability.list/health.liveness responses are correct for this framing
2. **Coverage gaps**: pentest/fuzz/crypto-protocol remain low coverage because they're live-service integration tests. Consider whether primalSpring's benchScale Docker topologies could enable offline testing
3. **reqwest weight**: 115/173 transitive deps come from reqwest. If tunnelKeeper evolves toward Songbird native transport, reqwest can be eliminated entirely
4. **Deploy script config**: All hardcoded ports now reference `nucleus_config.sh`. Verify `deploy_membrane.sh` in cellMembrane also uses these variables

### No upstream blockers from this work

All changes are internal correctness and coverage improvements. No new upstream dependencies, no API surface changes, no primal contract changes.

---

## NC-1→NC-5 Status (unchanged)

| Gap | Status | Blocker |
|-----|--------|---------|
| NC-1 (spore gateway) | WIRED | biomeOS NC-1.4 |
| NC-2 (multi-gate mesh) | IN PROGRESS | southGate ops stabilization |
| NC-3 (cellMembrane sovereignty) | ADVANCING | Forgejo + NS cutover remaining |
| NC-4 (spring NUCLEUS depth) | MIXED | east/iron OK, south/biome partial |
| NC-5 (lithoSpore emission) | GATED | on NC-1 live deploy |

---

## Archive Audit

| Category | Count | Status |
|----------|-------|--------|
| TODO/FIXME/HACK in `.rs` | 0 | Clean |
| TODO/FIXME in `.sh` | 0 | Clean |
| Stale `.bak`/`.old`/`.swp` files | 0 | Clean |
| `.canvas.tsx` files | 0 | Clean |
| Loose test output dirs | 0 | Clean (all archived) |
| `validation/archive/` | 131 files | Intentional fossil record |
| `deploy/legacy/` | 2 files | Fossil record (superseded scripts) |
| `infra/benchScale/archive/` | 6 files | Fossil record (baseline runs) |

---

*projectNUCLEUS at Wave 58. 166 Rust tests, async-correct, wire-native discovery, zero debt. Ready for primalSpring audit.*
