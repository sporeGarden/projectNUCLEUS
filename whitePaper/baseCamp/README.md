# projectNUCLEUS BaseCamp — Sovereign Deployment Validation

**Product**: projectNUCLEUS
**Type**: Deployment and validation umbrella (not a domain science spring)
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What projectNUCLEUS Validates

projectNUCLEUS is not a Spring — it does not port Python science to Rust.
It is the **deployment product** that validates primalSpring's composition
patterns work in production:

- **13 NUCLEUS primals** deployed and healthy on real hardware
- **Security boundaries** hold under adversarial probing (Dark Forest)
- **Sovereignty infrastructure** replaces every external dependency
- **Composition graphs** execute correctly via biomeOS Neural API
- **Provenance pipeline** stamps validation results with NUCLEUS provenance

## Validation Pipeline

```
primalSpring graphs + standards
       ↓
projectNUCLEUS deploys on ironGate (local covalent gate)
       ↓
darkforest validates security (pentest, fuzz, crypto, observer)
       ↓
tunnelKeeper validates transport (health, DNS, config, replicas)
       ↓
benchScale validates parity (CF baseline → BearDog TLS → Songbird NAT)
       ↓
Gaps hand back to primalSpring → primals evolve → cycle continues
```

## Experiments

### E1: Security Boundary Validation (darkforest)

**Methodology**: Pure Rust security validator probes all 13 primals and
JupyterHub via JSON-RPC fuzz, HTTP pentest, crypto validation, and
observer-tier access control.

**Rust crate**: `validation/darkforest/` (v0.2.1)
**Test coverage**: 34 unit tests (Shannon entropy, hex decode, check builder,
report roundtrip, discovery module, capability probing)

**Key results**:
- 13/13 primals respond to JSON-RPC `health.liveness`
- Zero crashes under 20+ malformed payload types per primal
- BTSP cipher negotiation: 13/13 PASS
- Cookie entropy: above 6.0 bits (threshold: 4.0)
- Shadow hash: SHA-512 rounds ≥5000
- Discovery cascade: biomeOS → env → compiled defaults

**Evolution**: Discovery module added (Wave 18) replaces hardcoded port table
with capability-based runtime resolution via `primal.list`, `health.liveness`,
and `capability.list` JSON-RPC calls.

### E2: Transport Sovereignty Validation (tunnelKeeper)

**Methodology**: Pure Rust tunnel manager validates Cloudflare transport layer
while preparing Songbird sovereign replacement.

**Rust crate**: `validation/tunnelKeeper/` (v0.2.0)
**Test coverage**: 21 unit tests (YAML config roundtrip, ChaCha20 encrypt/decrypt,
health evaluation, JSON serialization)

**Key results**:
- Health probes: process, connectivity, DNS, config, replicas
- Credential encryption: ChaCha20-Poly1305 at rest (BearDog pattern)
- reqwest upgraded 0.12→0.13 (ring eliminated, aws-lc-rs provider)
- Zero clippy warnings (pedantic + nursery)

### E3: Sovereignty Parity Validation (benchScale)

**Methodology**: Shadow-run protocol compares Cloudflare tunnel baseline
against sovereign replacements (BearDog TLS, Songbird NAT, sovereign DNS).

**Location**: `infra/benchScale/`
**Key results**:
- HTTP parity PASS: VPS 68ms vs GitHub Pages 89ms
- BearDog TLS shadow: 11ms (Channel 3 ACME cert live)
- Songbird NAT relay: PASS on direct, variable on TTFB

## Primal Composition

projectNUCLEUS exercises the full NUCLEUS atomic:

| Atomic | Primals | Validation Surface |
|--------|---------|--------------------|
| **Tower** | BearDog + Songbird + skunkBat | BTSP, discovery, defense audit |
| **Node** | ToadStool + barraCuda + coralReef | Workload dispatch, GPU math |
| **Nest** | NestGate + rhizoCrypt + loamSpine + sweetGrass | Content storage, provenance pipeline |
| **Meta** | biomeOS + Squirrel + petalTongue | Orchestration, AI, visualization |

All 13 primals communicate via JSON-RPC 2.0 over Unix domain sockets.
TCP fallback available per Tier 5 discovery.

## Sovereign Infrastructure Status

| Layer | Status | Evidence |
|-------|--------|----------|
| Primal Capabilities | **PASS** | 451 methods, 13/13 primals LIVE |
| Security | **PASS** | BTSP 13/13, MethodGate 13/13, Dark Forest PASS |
| Deployment | **PASS** | VPS membrane live, Channel 3 TLS, dual-push mirror |
| Composition | **ACTIVE** | Forgejo PRIMARY (32 repos), sovereign DNS pending |

## References

- Composition patterns: `ecoPrimals/springs/primalSpring/graphs/`
- Ecosystem standards: `ecoPrimals/infra/wateringHole/`
- Deployment matrix: `primalSpring/config/deployment_matrix.toml`
- Gap handbacks: `infra/wateringHole/handoffs/`
