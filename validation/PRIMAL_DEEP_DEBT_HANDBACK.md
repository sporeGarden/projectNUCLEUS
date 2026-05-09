> **Canonical copy**: This handback has been archived to `foundation/validation/handbacks/PRIMAL_DEEP_DEBT_HANDBACK.md`.
> This copy remains in projectNUCLEUS as a reference.

# Primal Deep Debt Audit — Upstream Handback

**Date**: 2026-05-07
**From**: projectNUCLEUS (ironGate)
**For**: All primal teams, primalSpring
**Scope**: Code quality audit across all 15 locally-cloned primal repos.
Files >800 lines, unsafe code, bare `#[allow]`, hardcoded primal names,
production mocks, `unwrap()`/`expect()` in non-test code, `eprintln!`/`println!`
in library code.

---

## Methodology

Automated scan of all `**/src/**/*.rs` files in each primal repo at
`/home/irongate/Development/ecoPrimals/primals/`. Findings below reflect
the state as of 2026-05-07 after Phase 60 pulls.

---

## Per-Primal Findings

### barraCuda

| Category | Finding |
|----------|---------|
| **>800L** | `crates/barracuda-core/src/ipc/methods/math.rs` — **892 lines** |
| **unwrap()** | Heavy in `session/mod.rs` and `ops/*` — panic risk in long-running compute |
| **println!** | Doc examples only — clean |

**Recommendation**: Refactor `math.rs` into domain-grouped submodules. Systematic
`unwrap()` → `?` with typed errors in `session` and `ops`.

---

### bearDog

| Category | Finding |
|----------|---------|
| **>800L** | `showcase/04-advanced-features/05-post-quantum-readiness/src/main.rs` — **962** |
| | `showcase/04-advanced-features/03-hardware-attestation/src/main.rs` — **832** |
| | `showcase/04-advanced-features/04-zero-knowledge-proofs/src/main.rs` — **817** |
| | `crates/beardog-types/src/tests/coverage_gap_tests_7.rs` — **860** |
| | `crates/beardog-utils/src/tests/zero_copy_comprehensive_tests.rs` — **826** |
| **eprintln!** | `beardog-cli/src/lib.rs` uses `eprintln!("FATAL: {e}")` — should use `tracing::error!` |
| **Mocks** | `crates/beardog-tunnel/src/tunnel/hsm/hsm_provider_mocks.rs` — 550 lines. Verify not linked in production feature sets |
| **unwrap()** | Heavy in `ionic_bond/persistence.rs` — tighten in persistence paths |

**Recommendation**: Showcase files are examples (acceptable size). HSM mock gating
is the priority — ensure `#[cfg(test)]` or feature-gated.

---

### biomeOS

| Category | Finding |
|----------|---------|
| **>800L** | `crates/biomeos-types/src/constants/mod.rs` — **852** |
| | `crates/biomeos/src/main_tests.rs` — **882** |
| **Mocks** | `biomeos-core/registry_queries.rs` has `spawn_neural_api_mock` next to production code |
| **println!** | `biomeos-cli` handlers and `tools/src/lib.rs` — prefer `tracing` for library parts |

**Recommendation**: Extract `constants/mod.rs` into per-domain submodules.
Move mock helpers to a test module or `dev-dependencies` crate.

---

### coralReef

| Category | Finding |
|----------|---------|
| **>800L** | `crates/coral-driver/src/error.rs` — **928** |
| | `crates/coral-driver/src/nv/ioctl/mod.rs` — **929** |
| | `crates/coral-driver/src/nv/mod.rs` — **857** |
| | `crates/coral-driver/src/vfio/channel/mod.rs` — **896** |
| | `crates/coral-driver/src/nv/vfio_compute/vbios_devinit.rs` — **836** |
| **unsafe** | Heavy in `coral-driver` (VFIO, UVM, DRM) — **expected** for hardware drivers |
| **eprintln!** | **Biggest logging debt**: `open_kmod.rs`, `open_userspace.rs`, `compute_trait.rs` have extensive `eprintln!` debug traces |
| **Mocks** | `pri.rs` has `MockRegs` — verify `#[cfg(test)]` gating |

**Recommendation**: `eprintln!` → `tracing` is the highest-ROI change.
Driver `unsafe` is expected and should be audited for invariants, not removed.
Large files in driver code are acceptable given hardware register maps.

---

### loamSpine

| Category | Finding |
|----------|---------|
| **>800L** | `crates/loam-spine-api/src/jsonrpc/tests.rs` — **807** |
| **unsafe** | Small count in `loam-spine-core/src/lib.rs`, `traits/mod.rs` — verify soundness |
| **Mocks** | `transport/mock.rs` — verify feature-gated so production builds skip it |
| | `integration/mocks.rs` — 616 lines, integration-test oriented |
| **primal_names** | `crates/loam-spine-core/src/primal_names.rs` — 69 lines, centralized (good pattern) |

**Recommendation**: Verify mock transport is behind `#[cfg(test)]` or a `mock` feature flag.

---

### nestGate

| Category | Finding |
|----------|---------|
| **>800L** | Vendor files dominate (rustls-webpki `verify_cert.rs` — 1373, `dns_name.rs` — 1059) — **upstream**, not actionable |
| **unsafe** | `nestgate-core/safe_optimizations.rs`, `nestgate-zfs`, `nestgate-platform`, SIMD — expected for perf |
| **unwrap()** | Clusters in `nestgate-rpc` `model_cache_handlers.rs`, `nestgate-discovery` registry, `jsonrpc_client.rs`, `audit_storage.rs` |
| **Hardcoded** | `nestgate-config/constants/network_hardcoded.rs` — 239 lines, intentional centralization |

**Recommendation**: Systematic `unwrap()` → `?` in `nestgate-rpc` and `nestgate-discovery`
service paths. Vendor files are fork/upgrade work.

---

### petalTongue

| Category | Finding |
|----------|---------|
| **>800L** | `src/web_mode.rs` — **814** |
| **#[allow]** | `src/main.rs` ~line 376: `#[allow(clippy::too_many_arguments)]` — **no reason string** |
| **unsafe** | `petal-tongue-ipc/src/unix_socket_server.rs`, `socket_path.rs` — 1 hit each |
| **Mocks** | `sandbox/mock-biomeos` — scoped sandbox |

**Recommendation**: Add `reason` to the bare `#[allow]` or refactor the function.
`web_mode.rs` at 814L is borderline — consider extracting handler functions.

---

### rhizoCrypt

| Category | Finding |
|----------|---------|
| **>800L** | None above 801 (largest: `handler_tests.rs` at 794) |
| **unsafe** | `rhizo-crypt-core/src/config.rs` — 1 hit |
| **Coupling** | Client module tree (`clients/toadstool_http.rs`, `songbird/*`, `nestgate_http.rs`, `beardog_http.rs`) — filesystem/module naming for integrations, tight coupling visible in layout |

**Recommendation**: Consider capability-based client abstraction instead of per-primal client modules.

---

### skunkBat

| Category | Finding |
|----------|---------|
| **>800L** | None (largest: `negotiate.rs` at 790) |
| **unsafe** | None found |
| **dead_code** | `negotiate.rs` has `#[allow(dead_code, reason = "...")]` — documented future work |

**Recommendation**: Clean. No action needed.

---

### songBird

| Category | Finding |
|----------|---------|
| **>800L** | `crates/songbird-orchestrator/src/bin_interface/server.rs` — **800** (exact threshold) |
| **unsafe** | `songbird-universal-ipc` platform code, `songbird-types` zero-copy — expected small blocks |
| **Mocks** | `songbird-test-utils/src/mocks/*` — proper dev-deps pattern |

**Recommendation**: Clean. Monitor `server.rs` as it approaches threshold.

---

### sourDough

| Category | Finding |
|----------|---------|
| **>800L** | None |
| **unsafe** | `sourdough-genomebin/src/lib.rs` — 1 hit (likely FFI/signing) |

**Recommendation**: Clean. Verify the single `unsafe` block.

---

### squirrel

| Category | Finding |
|----------|---------|
| **>800L** | `crates/universal-patterns/src/security/providers/tests.rs` — **1105** |
| | `crates/tools/cli/src/mcp/tests.rs` — **915** |
| **unsafe** | `main/unix_socket.rs`, `universal/mod.rs`, `plugins/manager.rs` — few hits |
| **primal_names** | `universal-constants/src/primal_names.rs` — 138 lines, centralized (good) |
| **unwrap()** | Heavy in test modules; also in `main/src` non-test paths |

**Recommendation**: Split the 1105-line test file. The `primal_names.rs` centralization
is the correct pattern.

---

### sweetGrass

| Category | Finding |
|----------|---------|
| **>800L** | None (largest: `btsp/transport.rs` at 763) |
| **unsafe** | `bootstrap.rs` — 1 hit |
| **Mocks** | `crypto_delegate.rs` has `start_mock_beardog` — verify `#[cfg(test)]` |
| **primal_names** | `sweet-grass-core/src/primal_names.rs` — 203 lines, centralized (good) |

**Recommendation**: Verify mock BearDog is test-only. Otherwise clean.

---

### toadStool

| Category | Finding |
|----------|---------|
| **>800L** | None |
| **unsafe** | Large footprint in `hw-safe`, `runtime/gpu`, `runtime/display`, `plugin_system/ffi_loader.rs`, `akida-driver` — **expected** for hardware/FFI |
| **Mocks** | `server/src/mocks.rs` — 207 lines. Confirm test/dev only |
| **eprintln!** | `runtime/adaptive/src/lib.rs` has `eprintln!` for GPU note — library noise |

**Recommendation**: `eprintln!` → `tracing` in library code. Hardware `unsafe` is expected.

---

## Cross-Cutting Patterns

### Positive Patterns (ecosystem-wide)

1. **`primal_names.rs` centralization**: loamSpine, squirrel, sweetGrass all centralize primal name constants — good pattern, should be ecosystem standard
2. **`#[expect(reason)]` over `#[allow]`**: Most primals have migrated — petalTongue has one remaining bare `#[allow]`
3. **`#![forbid(unsafe_code)]`**: wetSpring, healthSpring, ludoSpring all use workspace-level forbid — primals with hardware needs (coralReef, toadStool) are appropriately exempt

### Debt Priorities (highest ROI)

| Priority | Primal | Action | Impact |
|----------|--------|--------|--------|
| 1 | coralReef | `eprintln!` → `tracing` in coral-driver | Biggest logging debt, affects production filtering |
| 2 | barraCuda | `unwrap()` → `?` in session/ops | Panic risk in long-running compute |
| 3 | nestGate | `unwrap()` → `?` in rpc/discovery | Service reliability |
| 4 | biomeOS | Move mock helpers out of core | Code hygiene |
| 5 | bearDog | HSM mock feature gating | Build hygiene |
| 6 | petalTongue | Add `reason` to bare `#[allow]` | Zero bare `#[allow]` ecosystem target |
| 7 | squirrel | Split 1105-line test file | Maintainability |

### Files >800 Lines Summary

| Primal | Count | Context |
|--------|-------|---------|
| coralReef | 5 | Hardware driver — register maps, acceptable |
| bearDog | 5 | Showcase examples (3) + test files (2) |
| biomeOS | 2 | Constants + tests |
| squirrel | 2 | Test files |
| barraCuda | 1 | IPC methods — should split |
| petalTongue | 1 | web_mode — borderline |
| loamSpine | 1 | Test file |
| **Total** | **17** | |
