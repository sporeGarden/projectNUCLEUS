# projectNUCLEUS Deep Debt Resolution — Handoff to Upstream

**From**: projectNUCLEUS (garden product)
**To**: primalSpring, upstream primals teams, sibling springs
**Date**: 2026-05-16
**Wave**: 18 (post-audit, post-debt-resolution)

## Summary

projectNUCLEUS completed a deep debt resolution pass covering code quality,
test coverage, dependency evolution, capability-based discovery, and graph
synchronization. This handoff documents findings, evolution patterns, and
gaps that require upstream attention.

---

## What projectNUCLEUS Resolved Locally

### Test Coverage (0 → 55 tests)

| Crate | Before | After | Modules Covered |
|-------|--------|-------|-----------------|
| darkforest | 0 | 34 | crypto (entropy, hex), check (builder, serde), report (summary, JSON), discovery (resolve, probe, by_capability) |
| tunnelKeeper | 0 | 21 | config (YAML roundtrip, save/load), crypto (ChaCha20 roundtrip, Ed25519, EncryptedBlob), health (config eval, connectivity, JSON serialization) |

### Capability-Based Discovery Module

darkforest replaced its hardcoded `DEFAULT_PRIMALS` port table with a
3-tier resolution cascade:

1. **biomeOS `primal.list`** — live topology via JSON-RPC
2. **Per-primal env vars** — ops override (`BEARDOG_PORT`, etc.)
3. **Compiled defaults** — last-resort fallback

Each resolved primal is probed via:
- `health.liveness` — confirms the primal is responsive
- `capability.list` — discovers available methods

**Pattern for upstream**: This cascade should become the standard for any
downstream consumer that discovers primals. The `by_capability()` function
filters primals by advertised method — consumers should never hardcode
primal identity, only capability requirements.

### Dependency Evolution

| Dependency | Before | After | Impact |
|-----------|--------|-------|--------|
| reqwest | 0.12 (ring via rustls) | 0.13.3 (aws-lc-rs via rustls) | `ring` eliminated from tree |
| aws-lc-sys | — | Transitive via rustls | C/asm — tracked ecoBin gap |

**For upstream**: Full ecoBin compliance requires `rustls-rustcrypto` to
replace `aws-lc-rs`. Currently alpha (0.0.2). When it stabilizes, use
reqwest's `rustls-no-provider` feature + explicit provider install.
darkforest has zero C dependencies.

### Graph Fragment Synchronization

All local graph fragments synchronized to primalSpring v3.0.0:
- `tower_atomic.toml`: 3 primals (BearDog + Songbird + skunkBat)
- `node_atomic.toml`: 6 primals (Tower + compute trio)
- `nest_atomic.toml`: 7 primals (Tower + storage + provenance)
- `nucleus.toml`: 10 domain primals
- `nucleus_complete.toml`: Fixed duplicate order (sweetgrass=10, skunkbat=11)

### Code Quality

- `#![forbid(unsafe_code)]` on both crates
- Zero clippy warnings (pedantic + nursery)
- cargo fmt clean
- All files under 800 LOC (darkforest's pentest modules are large by design
  — security test harnesses with intentional density)

---

## Gaps Requiring Upstream Primal Evolution

### P0: biomeOS Neural API — `primal.list` Response Format

darkforest's discovery module queries `primal.list` but the response schema
is not formally specified. We parse both `result` as direct array and
`result.primals` as nested array. biomeOS should publish a canonical schema:

```json
{
  "result": {
    "primals": [
      { "name": "beardog", "port": 9100, "capabilities": ["crypto.sign", ...] }
    ]
  }
}
```

### P0: Capability Method Naming Convention

Different primals return capabilities in different shapes via
`capability.list`. Some return flat string arrays, some return objects
with `method` + `description` + `category`. Standardize the response
format across all 13 primals.

### P1: skunkBat in Tower Atomic

All projectNUCLEUS fragments now include skunkBat in Tower (v3.0.0).
Upstream primalSpring fragments should reflect this if they haven't already.
skunkBat is Tower's defense layer — it should deploy alongside BearDog
and Songbird on every gate, including minimal ones.

### P1: Signal Adoption for NUCLEUS Workloads

projectNUCLEUS workload TOMLs still use method sequences:
```
content.put → dag.event.append → spine.seal → braid.create
```

Per `SIGNAL_ADOPTION_STANDARD.md`, these should migrate to:
```
ctx.dispatch("nest.store", ...)
```

This requires biomeOS signal graph execution to be exercisable from
shell-level deploy tooling (not just Rust `CompositionContext`).

### P2: Songbird Transport for tunnelKeeper

tunnelKeeper v0.3 needs `songbird-quic` and `songbird-tls` as library
dependencies. Current blockers:
- Songbird doesn't yet publish library crates (only binary)
- QUIC transport validation not yet shadow-run against Cloudflare baseline
- NAT traversal parity testing incomplete

### P2: `rustls-rustcrypto` Ecosystem Evolution

The last C/asm dependency in the NUCLEUS deploy stack is `aws-lc-sys` via
`rustls`. When `rustls-rustcrypto` stabilizes beyond 0.0.2-alpha, all
downstream consumers (tunnelKeeper, any primal using reqwest/rustls)
should migrate. This is an ecosystem-wide coordination item.

---

## Findings for Sibling Springs

### Python Benchmarks for barraCuda

| Source | Location | Coverage |
|--------|----------|----------|
| hotSpring | `benchmarks/kokkos-lammps/validate_9cases_python.py` | NumPy Yukawa MD 9 DSF cases |
| wetSpring | `scripts/benchmark_rust_vs_python.py` | 23 algorithmic domains |
| wetSpring | `scripts/benchmark_python_baseline.py` | NumPy/SciPy timing baselines |
| barraCuda | `benches/scipy_parity.rs` | sum/variance/cdist (Bessel/Beta missing) |

**Gap**: No in-tree Python driver that mirrors `cargo bench` for barraCuda.
The Rust benchmarks reference SciPy timings as text comments only.
`scipy_parity.rs` documents Bessel J0/J1 and Beta but doesn't exercise them.

### Industry Benchmark Status

| Framework | Status |
|-----------|--------|
| Kokkos-CUDA | Narrative + shell scripts in hotSpring, no CI-exercised binary |
| LAMMPS | Reference timings in bench comments, not validated |
| cuBLAS/cuDNN | Excluded by ecoBin policy (no CUDA FFI) |
| CUTLASS/Galaxy | Not referenced |

### Paper Queue

Each spring maintains its own queue. No ecosystem-wide bibliography exists.
Key sources: lithoSpore `papers/registry.toml` (16 papers), ludoSpring
`docs/PAPER_QUEUE.md`, airSpring/healthSpring `specs/PAPER_REVIEW_QUEUE.md`.

### Dataset Readiness

**Ready**: LTEE genomes, public 16S BioProjects, ERA5 weather, MNIST, WCM proteomes.
**Needs acquisition**: Large SRA FASTQ (25-55GB+), UniRef90/PDB at scale, MIMIC-III.

---

## Composition Patterns Validated

### Atomic Signal Dispatch

The Neural API flow validated on ironGate:

```
user intent
  → biomeOS receives signal (e.g. "nest.store")
  → biomeOS resolves signal to primal method sequence
  → JSON-RPC dispatch to resolved primals
  → provenance trio stamps the result
  → result returned to caller
```

### Discovery Hierarchy (5-tier, validated by darkforest)

1. Songbird IPC discovery
2. biomeOS Neural API `primal.list`
3. Unix domain socket probing
4. Registry lookup (primalSpring coordination)
5. TCP fallback (port table)

darkforest exercises tiers 2, 3, and 5. Tiers 1 and 4 require Songbird
and primalSpring coordination primal respectively.

### Deploy Graph Execution

`nucleus_complete.toml` defines the canonical 14-node graph:
- biomeOS (order 0, orchestrator, spawn=false)
- Tower: BearDog (1) → Songbird (2)
- Core: ToadStool (3) → barraCuda (4) → coralReef (5)
- Storage: NestGate (6) → Squirrel (7)
- Provenance: rhizoCrypt (8) → loamSpine (9) → sweetGrass (10)
- Defense: skunkBat (11)
- Interface: petalTongue (12)
- Coordination: primalSpring (13, spawn=false)

---

## Action Items for Upstream

| Priority | Owner | Item |
|----------|-------|------|
| P0 | biomeOS | Publish canonical `primal.list` response schema |
| P0 | All primals | Standardize `capability.list` response format |
| P0 | primalSpring | Verify skunkBat in Tower fragments (v3.0.0) |
| P1 | biomeOS | Signal dispatch from shell tooling (`signal_executor.sh`) |
| P1 | Songbird | Library crate for transport (tunnelKeeper v0.3 blocked) |
| P1 | barraCuda | Complete scipy_parity bench (Bessel/Beta) |
| P2 | Ecosystem | Track rustls-rustcrypto stabilization |
| P2 | projectFOUNDATION | BLAKE3 backfill (165 empty fields) |
| P2 | projectFOUNDATION | Thread 1 WCM RPC stack |
