# projectNUCLEUS Wave 55 Deep Debt + Niche Climate Handoff — May 27, 2026

**From**: projectNUCLEUS (ironGate)
**To**: primalSpring, biomeOS, Songbird, cellMembrane, lithoSpore, hotSpring teams
**Scope**: Deep debt resolution, niche climate readiness, composition patterns, upstream evolution notes

---

## Current State

| Metric | Value |
|--------|-------|
| Registry | 460 methods (Wave 56, primalSpring v0.9.30) |
| Rust unit tests | **65** (darkforest 44, tunnelKeeper 21) — zero failures |
| Bash validation | 267 security checks + 33 gate checks + 21 membrane checks |
| Shadow orchestrator | **6 PASS / 0 FAIL / 0 SKIP** (S1–S5 + DNS) |
| Provenance trio | 10/10 PASS on VPS |
| VPS composition | Nest Atomic — 11 services, 7 primals |
| Clippy | pedantic+nursery: **zero warnings** across both crates |
| C dependencies | darkforest: **zero**. tunnelKeeper: ring + aws-lc-sys (rustls TLS only) |

---

## Deep Debt Resolution (this session)

### 1. YAML: libyaml eliminated → serde-saphyr

`yaml_serde 0.10` (which pulled `libyaml-rs` — a C binding) replaced with `serde-saphyr v0.0.26`:
- Pure Rust, panic-free, no `unsafe`, YAML 1.2 compliant
- 934K downloads in 90 days, actively maintained
- Import: `serde_saphyr::from_str` / `serde_saphyr::to_string`
- `deny.toml` updated to ban `unsafe-libyaml`

**Upstream relevance**: Any primal using `serde_yaml` or `yaml_serde` for YAML parsing can migrate
to `serde-saphyr` for pure Rust. Drop-in except: serialization errors are `serde_saphyr::ser::Error`
(separate type from deserialization `serde_saphyr::Error`).

### 2. net.rs refactored — shared HTTP parsing helpers

Extracted `parse_status_code()` and `split_http_response()` from duplicated inline logic across
`http_get`, `http_post`, `http_method`, `send_jsonrpc`. Added 10 unit tests for HTTP parsing and
graceful failure on unreachable hosts.

### 3. Full codebase audit — clean bill

| Check | Result |
|-------|--------|
| Files >800 lines | **Zero** |
| `unsafe` blocks | **Zero** (`#![forbid(unsafe_code)]` in both crates) |
| `TODO`/`FIXME`/`HACK` | **Zero** in source code |
| Production mocks | **Zero** (adversarial fuzz probes are security testing, not mocks) |
| `.unwrap()`/`.expect()` in production | **Zero** (all confined to `#[cfg(test)]`) |
| Production panics | **Zero** |
| Dead code / `#[allow(dead_code)]` | **Zero** |

### 4. Hardcoded values — already agnostic

All hardcoded values follow env-var fallback cascades:
- Primal ports: `BEARDOG_PORT=9100`, etc. (env override → compiled default)
- Discovery: biomeOS `primal.list` → env vars → compiled table
- Gate home: `GATE_HOME` → `HOME` → `/home/nobody` sentinel
- VPS config: centralized in `deploy/nucleus_config.sh` with `${VAR:-default}`

No changes needed — patterns are correct.

---

## Niche Climate Readiness (NC-1→NC-5)

### NC-1: postPrimordial Spore Gateway (biomeOS + lithoSpore)

**Status**: BLOCKED — biomeOS v3.77 scaffolded `nucleus ingest/emit`, v3.78 cleaned up.
NC-1.3 (pseudospore-core in lithoSpore) COMPLETE. NC-1.4 (biomeOS uses pseudospore-core) OPEN.

**projectNUCLEUS readiness**: ironGate VPS has live Nest Atomic. When biomeOS ships:
1. Deploy biomeOS v3.77+ to ironGate
2. Run `biomeos nucleus ingest <pseudoSpore-dir>` against hotSpring v1.6.1
3. Verify `receipts/nucleus_ingest.toml`
4. Column U closes

**Signal graph**: `nest_ingest_spore.toml` documented in `graphs/README.md`.
Six sequential nodes: NestGate `storage.exists` + `content.put` → rhizoCrypt `dag.session.create` →
loamSpine `entry.append` → sweetGrass `braid.create` → BearDog `crypto.sign`.

**Note for biomeOS**: primalSpring flagged NC-1.4 — `nucleus_ingest.rs` uses inline envelope
validation instead of lithoSpore's `pseudospore-core` crate. One canonical implementation recommended.

### NC-2: Multi-Gate Mesh (Songbird + wetSpring)

**Status**: BLOCKED — southGate 7/13 health-responding.

**Wave 54 redeploy fixes ready** (plasmidBin bf5c96b→b310310):
- petalTongue: `--socket`/`--port` global CLI flags
- barraCuda: `--no-gpu-probe` / `BARRACUDA_NO_GPU_PROBE`
- ToadStool: early health responder on pre-bound socket
- Squirrel: SQ-01 socket path fix
- Launcher: skunkBat in STARTUP_ORDER

**Note for Songbird**: `discovery.peers` empty after `mesh.init` (v0.2.1). Bidirectional
`SONGBIRD_PEERS` seeding needed for cross-gate mesh.

### NC-3: cellMembrane Sovereignty (cellMembrane)

**Status**: IN PROGRESS — NC-3.1/3.2 DONE (VPS_STATE synced, membrane.toml = nest, signal enabled).

**Remaining**:
- NC-3.3: knot-dns NS cutover (registrar action)
- NC-3.4: Forgejo releases (coordinate with plasmidBin `auto-harvest.yml`)
- NC-3.5: sporePrint living content (blocked on BearDog `content.*` scope)

---

## Composition Patterns for NUCLEUS Deployment

### Atomic compositions (validated)

| Composition | Primals | Deploy Graph | VPS Status |
|-------------|---------|-------------|------------|
| Tower | BearDog + Songbird + SkunkBat | `tower_atomic.toml` | LIVE |
| Node | Tower + ToadStool + barraCuda + coralReef | `node_atomic.toml` | Gate-local |
| Nest | Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass | `nest_atomic.toml` | LIVE on VPS |
| NUCLEUS | Tower + Node + Nest (10+ primals) | `nucleus.toml` | ironGate |

### Neural API ingest pattern (NC-1 — when biomeOS ships)

```
biomeos nucleus ingest <pseudoSpore-dir>
  → nest_ingest_spore signal graph
    → NestGate storage.exists + content.put
    → rhizoCrypt dag.session.create
    → loamSpine entry.append
    → sweetGrass braid.create
    → BearDog crypto.sign
  → receipts/nucleus_ingest.toml
```

### Discovery cascade (darkforest pattern — reusable)

```
1. biomeOS primal.list (runtime topology) — canonical Wave 20+
2. Environment variables (BEARDOG_PORT, NESTGATE_PORT, etc.)
3. Compiled defaults (last resort)

by_capability("crypto") → filters primals advertising that capability
```

---

## Upstream Evolution Notes

### For primalSpring
- **DOWNSTREAM_PATTERN_GUIDE.md** still says 445 methods in 4 places — should be 460
- toadStool git remote expects `master` but only `main` exists — pull fails
- exp115 `nest_ingest_pseudospore` is structural; live phases activate when NC-1 ships
- Our 65 Rust tests + 267 bash checks + 33 gate + 21 membrane = 386+ validations

### For biomeOS
- NC-1.4: swap inline envelope validation for `pseudospore-core` crate
- Signal graph divergence: biomeOS copy omits `storage.store` step and bonding policy present in primalSpring canonical copy — reconcile
- Neural API routes `nucleus.ingest_spore` and `nucleus.ingest` both exist — clarify which is canonical

### For Songbird
- southGate 7/13 health: `discovery.peers` empty after mesh.init
- Bidirectional SONGBIRD_PEERS seeding needed for NC-2.5
- TCP fallback mesh seed bug identified — fixed in v0.2.1 but may need redeploy

### For cellMembrane
- NC-3.1/3.2 DONE — your ops docs are synced, membrane.toml says nest
- NC-3.3 registrar NS cutover is the next step — blocks sovereignty layer S2
- NC-3.5 needs BearDog `auth.issue_session` scope for `content.*`

### For lithoSpore
- NC-1.3 COMPLETE — `pseudospore-core` wired
- CHASSIS.md and PSEUDOSPORE_STANDARD.md specs pulled (Wave 55)
- projectFOUNDATION BLAKE3 backfill on threads 4, 5, 1 is their next action

### For hotSpring
- First NUCLEUS ingest target: pseudoSpore v1.6.1 artifact
- biomeGate elevation (NC-2.6) needs coordination when HBM2 hardware permits
- Local repo diverged from remote main — needs rebase

---

## Stadial Entry Requirements

| Requirement | Current | Target | Blocker |
|-------------|---------|--------|---------|
| NC-1: Column U for 2+ springs | 0 | 2+ | biomeOS gateway |
| NC-2: 3+ gates meshed | 1 (ironGate) | 3+ | southGate 7/13 |
| NC-4: All 4 gates healthy | 1/4 | 4/4 | southGate, biomeGate |

projectNUCLEUS is **ready to execute** when external blockers clear.
