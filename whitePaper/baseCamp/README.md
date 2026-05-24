# projectNUCLEUS BaseCamp — Sovereign Deployment Validation

**Product**: projectNUCLEUS
**Type**: Deployment and validation umbrella (not a domain science spring)
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)
**Updated**: May 19, 2026

## What projectNUCLEUS Validates

projectNUCLEUS is not a Spring — it does not port Python science to Rust.
It is the **deployment product** that validates primalSpring's composition
patterns work in production:

- **13 NUCLEUS primals** deployed and healthy on real hardware
- **Security boundaries** hold under adversarial probing (Dark Forest)
- **Sovereignty infrastructure** replaces every external dependency
- **Composition graphs** execute correctly via biomeOS Neural API
- **Provenance pipeline** stamps validation results with NUCLEUS provenance
- **Shadow parity** proves sovereign components outperform commercial

## Validation Pipeline

```
primalSpring graphs + wateringHole standards
       ↓
projectNUCLEUS deploys on ironGate (local covalent gate)
       ↓
darkforest validates security (pentest, fuzz, crypto, observer)
 → 34 unit tests, 267 security checks, 86 observer checks
       ↓
tunnelKeeper validates transport (health, DNS, config, replicas)
 → 21 unit tests, reqwest + rustls (zero ring)
       ↓
darkforest_membrane audits cellMembrane VPS (SSH, UFW, TURN, listeners)
 → 17/17 PASS (MEM-01 through MEM-13)
       ↓
benchScale validates sovereignty parity (CF baseline → sovereign shadow)
 → 4-track shadow: TLS + NAT + content + auth
       ↓
lithoSpore validates cross-tier science (Python ↔ Rust ↔ Primal IPC)
 → 7/7 modules PASS, 75/75 checks, 117 tests, 7/7 parity MATCH
       ↓
Gaps hand back to primalSpring → primals evolve → cycle continues
```

## Experiments

### E1: Security Boundary Validation (darkforest v0.2.1)

Pure Rust security validator probes all 13 primals via JSON-RPC fuzz,
HTTP pentest, crypto validation, and observer-tier access control.

**34 unit tests**, 267 security checks, 33 structural gate checks.

Key results:
- 13/13 primals respond to `health.liveness`
- BTSP cipher negotiation: 13/13 PASS
- Cookie entropy: above 6.0 bits (threshold: 4.0)
- Discovery cascade: biomeOS → env → compiled defaults
- Wave 20 canonical `primal.list` / `capability.list` parsing

### E2: Transport Sovereignty Validation (tunnelKeeper v0.2.0)

Pure Rust tunnel manager validates Cloudflare transport while preparing
Songbird sovereign replacement.

**21 unit tests**. ChaCha20-Poly1305 credential encryption. reqwest 0.13
with rustls (ring dependency eliminated).

### E3: Sovereignty Parity Validation (benchScale)

Shadow-run protocol: calibrate → shadow → cutover. Four tracks measured
against 7-day rolling baselines. Orchestrated by `shadow_run_orchestrator.sh`.

| Track | Sovereign | Commercial | Measured |
|-------|-----------|------------|----------|
| S1 TLS | BearDog :8443 (rustls) | Cloudflare TLS | **6-12ms** vs 163ms (13-27× faster) |
| S2 NAT | Songbird TURN relay | cloudflared tunnel | **100% reachable**, 3ms UDP |
| S3 Content | petalTongue + NestGate | GitHub Pages | **67ms VPS** vs 111ms GH (40% faster) |
| S4 Auth | BearDog BTSP dual-auth | OAuth2 proxy | Spec shipped, integration pending |

**Cutover gate**: All 4 tracks must demonstrate parity for 7 consecutive days.
Sovereign p95 ≤ 1.5× commercial p95.

### E4: Membrane Security Audit (darkforest_membrane)

Remote audit of cellMembrane VPS: 17 PASS, 0 FAIL, 1 SKIP.
SSH hardening, fail2ban, UFW deny-default, TURN auth, credential perms.

### E5: lithoSpore Cross-Tier Parity

7 LTEE science modules: Python Tier 1 ↔ Rust Tier 2 parity proven.
75/75 checks, 117 tests, 7/7 MATCH. Tier 3 wired for provenance trio.

## Primal Composition

projectNUCLEUS exercises the full NUCLEUS atomic:

| Atomic | Primals | Validation Surface |
|--------|---------|--------------------|
| **Tower** | BearDog + Songbird + skunkBat | BTSP auth, mesh discovery, defense audit |
| **Node** | ToadStool + barraCuda + coralReef | Workload dispatch, GPU math, shader compile |
| **Nest** | NestGate + rhizoCrypt + loamSpine + sweetGrass | Content storage, DAG, ledger, attribution |
| **Meta** | biomeOS + Squirrel + petalTongue | Orchestration, AI inference, visualization |

All 13 primals communicate via JSON-RPC 2.0 over Unix domain sockets.
TCP fallback available per Tier 5 discovery. 458 registered methods (Wave 46, stable).

## Sovereign Infrastructure Status

| Layer | Status | Evidence |
|-------|--------|----------|
| Primal Capabilities | **PASS** | 458 methods (Wave 46), 13/13 primals LIVE |
| Security | **PASS** | BTSP 13/13, MethodGate 13/13, Dark Forest 267 checks |
| Deployment | **PASS** | cellMembrane Nest Atomic (11 services, 7 primals), Channel 3 TLS (ACME) |
| Composition | **ACTIVE** | Forgejo PRIMARY (32 repos, 3 orgs), dual-push mirror |
| Shadow Parity | **PASS** | **6/0/0** — S1 TLS, S2 NAT, S3 content, S4 auth, S5 DNS all LIVE |
| Sovereign DNS | **DEPLOYED** | knot-dns v3.2.6, DNSSEC on VPS. NS cutover (H2-18) pending registrar |

## cellMembrane Interface (VPS 157.230.3.183)

The cellMembrane is the first deployment of NUCLEUS primals on external
substrate — a $12/mo DigitalOcean droplet running **Nest Atomic composition**
(Tower + NestGate + provenance trio) alongside transitional services.

### Services (May 23, 2026)

| Service | Port | Function | Status |
|---------|------|----------|--------|
| BearDog | :9100 | Tower crypto, BTSP auth | RUNNING v0.9.0 |
| BearDog TLS shadow | :8443 | Sovereignty shadow S1 | RUNNING v0.9.0 |
| SkunkBat | :9140 | Audit, threat defense | RUNNING |
| Songbird TURN | UDP :3478 | NAT relay (RFC 5766) | RUNNING v0.2.1 |
| NestGate | :9500 | Content-addressed storage | RUNNING v2.1.0 |
| rhizoCrypt | :9602 | Ephemeral DAG memory | RUNNING v0.14.0 |
| loamSpine | :9700 | Permanence ledger | RUNNING v0.9.16 |
| sweetGrass | :9850 | Attribution braids | RUNNING v0.7.34 |
| Caddy | :80, :443 | TLS termination (transitional) | RUNNING (ACME) |
| petalTongue web | :8080 | Content surface S3 | RUNNING |
| RustDesk hbbs/hbbr | :21115-21119 | Remote desktop relay | RUNNING |

### Phase Evolution

```
Phase 0   (Relay only)      → Songbird TURN + RustDesk           ← DONE
Phase 0.5 (Tower on VPS)    → + BearDog + SkunkBat + Caddy TLS   ← DONE
Phase 1   (Tower compose)   → BearDog + Songbird + SkunkBat + RustDesk ← DONE
Phase 1.5 (Nest Atomic)     → + NestGate + rhizoCrypt + loamSpine + sweetGrass ← CURRENT
Phase 2   (Content)         → petalTongue replaces GitHub Pages   ← SHADOW ACTIVE
Phase 3   (DNS)             → knot-dns replaces Cloudflare DNS    ← DEPLOYED (H2-17)
Phase 4   (Full mesh)       → Multiple VPS, biomeOS orchestrated  ← FUTURE (FlockGate H3-11)
```

### Degradation Behavior

Every service degrades gracefully — science is never gated behind primal
availability:

| Service | If unavailable | Impact |
|---------|----------------|--------|
| Songbird TURN | Direct + STUN fallback | Remote access only (not science) |
| BearDog crypto | Unsigned/unencrypted fallback | Security reduced, not broken |
| SkunkBat audit | Audit logging disabled | No threat detection; no data loss |
| Caddy TLS | HTTP fallback or BearDog direct | Channel 3 down; Channel 1/2 unaffected |
| petalTongue web | GitHub Pages remains primary | Content served from commercial CDN |

### Monitoring

- `darkforest_membrane.sh`: 17 security checks on every VPS change
- `membrane_telemetry.sh`: 15-min cadence → `membrane_7day.toml` rolling baseline
- `membrane_summary.sh`: 7-day aggregate with cutover gate evaluation
- `shadow_run_orchestrator.sh`: weekly 4-track parity measurement

## Progress Toward Sovereignty

### What is sovereign today

- **Code**: Forgejo at git.primals.eco is PRIMARY. 32 repos, 3 orgs. GitHub is
  read-only mirror (dual-push). All code AGPL-3.0-or-later.
- **TLS**: BearDog rustls terminates on :8443. ACME cert via Caddy on :443.
  BearDog is 13-27× faster than Cloudflare tunnel (6-12ms vs 163ms).
- **NAT**: Songbird TURN relay on UDP :3478. 100% reachable. 5-tier
  ConnectionFallbackChain (direct → STUN → lineage → TURN → emergency).
- **Content**: petalTongue serves HTTP on :8080. VPS TTFB 67ms vs GitHub
  Pages 111ms (40% faster). Content hash parity pending (mirror not done).
- **Compute**: 13 primals on ironGate via biomeOS composition.deploy.
  445 registered methods. lithoSpore 7/7 modules validated.
- **Security**: darkforest 267 checks PASS. BTSP 13/13 AEAD. FIDO2/CTAP2
  hardware attestation ready. membrane audit 17/17 PASS.

### What remains external

| Dependency | Sovereign Replacement | Status |
|------------|----------------------|--------|
| Cloudflare DNS | knot-dns | H2-17→20: not deployed |
| Cloudflare tunnel | Songbird relay | LIVE but not cutover |
| GitHub Pages | petalTongue + NestGate | LIVE but content not mirrored |
| GitHub Actions CI | Forgejo Actions | Not started |
| OAuth2 proxy | BearDog BTSP | Spec shipped, integration pending |
| Let's Encrypt | BearDog ACME Phase 3 | Phase 2 shipped, renewal daemon pending |

### Irreducible externals (never sovereign)

- Domain registrar (primals.eco)
- Linux kernel / systemd
- NVIDIA GPU drivers
- Let's Encrypt / ACME (browser trust chain)
- VPS commodity substrate (~$12/mo DigitalOcean)

## The Evolution Pattern

```
Phase 1 (Covalent)     → 13 primals on one gate, provenance pipeline    ← DONE
Phase 2 (Ionic)        → ABG access, tunnel, multi-gate, membrane       ← ACTIVE
Phase 3 (Self-Hosted)  → BTSP replaces TLS, Songbird replaces tunnel    ← SHADOW
Phase 4 (Full NUCLEUS) → biomeOS as OS, zero external dependencies      ← FUTURE
```

Nothing is removed until it is replaced. Nothing is replaced until the
replacement proves parity. The calibrate → shadow → cutover protocol ensures
measured, evidenced transitions. The glacial metaphor is precise: interstadial
(warm period) wires the plumbing; stadial (cold period) tests it under load.

## References

- Composition patterns: `ecoPrimals/springs/primalSpring/graphs/`
- Ecosystem standards: `ecoPrimals/infra/wateringHole/`
- Deployment matrix: `primalSpring/config/deployment_matrix.toml`
- Gap handbacks: `infra/wateringHole/handoffs/`
- Gen4 whitepaper: `ecoPrimals/infra/whitePaper/gen4/`
- Shadow graph: `graphs/sovereignty_shadow.toml`
- Living gaps: `specs/EVOLUTION_GAPS.md`
