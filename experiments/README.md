# projectNUCLEUS Experiments

Validation experiments for sovereign NUCLEUS deployment.

## Active Experiments

| ID | Name | Crate/Tool | Tests | Status |
|----|------|-----------|-------|--------|
| E1 | Security boundary validation | `validation/darkforest/` | 34 | PASS |
| E2 | Transport sovereignty | `validation/tunnelKeeper/` | 21 | PASS |
| E3 | Sovereignty parity | `infra/benchScale/` | 4 tracks | ACTIVE |
| E4 | Membrane security audit | `validation/darkforest_membrane.sh` | 13 checks | PASS |
| E5 | lithoSpore integration | `gardens/lithoSpore/` | 117 | PASS |

---

## E1: Dark Forest Security Validation

**Methodology**: Pure Rust security validator probes all 13 primals and
JupyterHub via JSON-RPC fuzz, HTTP pentest, crypto validation, and
observer-tier access control.

**Rust crate**: `validation/darkforest/` (v0.2.1)
**Test coverage**: 34 unit tests (Shannon entropy, hex decode, check builder,
report roundtrip, discovery module, capability probing)

**Results (validated May 2026)**:
- 13/13 primals respond to JSON-RPC `health.liveness`
- Zero crashes under 20+ malformed payload types per primal
- BTSP cipher negotiation: 13/13 PASS
- Cookie entropy: above 6.0 bits (threshold: 4.0)
- Shadow hash: SHA-512 rounds ≥5000
- Discovery cascade: biomeOS → env → compiled defaults
- Wave 20 canonical `primal.list` / `capability.list` envelope parsing

**Five-pillar structural validation** (`dark_forest_gate_local.sh`): 33 checks,
5 pillars (graph structure, security, deployment, sovereignty, composition).

**Observer-tier**: 86 PASS, 0 FAIL (static HTML surface).

## E2: Transport Sovereignty Validation (tunnelKeeper)

**Methodology**: Pure Rust tunnel manager validates Cloudflare transport layer
while preparing Songbird sovereign replacement.

**Rust crate**: `validation/tunnelKeeper/` (v0.2.0)
**Test coverage**: 21 unit tests (YAML config roundtrip, ChaCha20 encrypt/decrypt,
health evaluation, JSON serialization)

**Results**:
- Health probes: process, connectivity, DNS, config, replicas
- Credential encryption: ChaCha20-Poly1305 at rest (BearDog pattern)
- reqwest upgraded 0.12→0.13 (ring eliminated, aws-lc-rs provider)
- Zero clippy warnings (pedantic + nursery)

**Evolution roadmap**: v0.2 (current) → v0.3 (SongbirdTransport replaces
CloudflareTunnelTransport) + BearDogAuthTransport replaces CF Access.

## E3: Sovereignty Parity Validation (benchScale)

**Methodology**: Shadow-run protocol (calibrate → shadow → cutover) compares
external dependency baselines against sovereign replacements. Orchestrated by
`shadow_run_orchestrator.sh`, measured by `membrane_telemetry.sh`, gated by
7-day rolling window via `membrane_summary.sh`.

**Location**: `infra/benchScale/`
**Deploy graph**: `graphs/sovereignty_shadow.toml`
**Protocol**: `wateringHole/SOVEREIGNTY_STANDARDS.md` §2

### Shadow Matrix (May 19, 2026)

| # | Track | Sovereign | Commercial | Status | Measured |
|---|-------|-----------|------------|--------|----------|
| S1 | TLS termination | BearDog :8443 (rustls) | Cloudflare TLS | **LIVE** | 6-12ms RPC vs 163ms CF TTFB |
| S2 | NAT traversal | Songbird TURN relay | cloudflared tunnel | **LIVE** | 100% reachable (3ms UDP) |
| S3 | Content hosting | NestGate + petalTongue | GitHub Pages | **LIVE** | 0.4ms local / 67ms VPS vs 111ms GH |
| S4 | Auth / JupyterHub | BearDog BTSP dual-auth | OAuth2 proxy | **READY** | Spec shipped, integration pending |

### Orchestrator Progression (May 19)

```
Run 1 (15:04 UTC): 2 PASS, 0 FAIL, 3 SKIP  — BearDog/petalTongue not running
Run 2 (15:16 UTC): 2 PASS, 0 FAIL, 3 SKIP  — probe bugs (curl vs JSON-RPC)
Run 3 (15:18 UTC): 4 PASS, 0 FAIL, 1 SKIP  — probes fixed, services started
Run 4 (15:21 UTC): 4 PASS, 0 FAIL, 1 SKIP  — stable (knot-dns only SKIP)
```

### Parity Reports

| Track | Report | Key metric | Gate met? |
|-------|--------|------------|-----------|
| S1 TLS | `btsp_tls_parity_*.toml` | BearDog RPC p95: 6-12ms; CF TTFB p95: 163ms | Latency yes; 7-day window not started |
| S2 NAT | `songbird_nat_parity_*.toml` | TURN reachability: 100% (5/5, repeated) | Reachability yes; dual-path pending |
| S3 Content | `nestgate_content_parity_*.toml` | TTFB parity: PASS; content hash: FAIL (not mirrored) | TTFB yes; mirror pending |
| S4 Auth | — | No measurements yet | Not started |
| DNS | `dot_parity_*.toml` | DoT baseline: 3-8ms all domains 10/10 | Sovereign knot-dns not deployed |

### Baselines

**Cloudflare tunnel** (9 days, 950 samples, `cloudflare_tunnel_7day.toml`):

| Metric | p50 | p95 | p99 |
|--------|-----|-----|-----|
| TLS handshake | 73ms | 101ms | 232ms |
| TTFB | 119ms | 190ms | 315ms |

**Membrane unified** (1 day, 117 probes, `membrane_7day.toml`):

| Service | Uptime | Latency |
|---------|--------|---------|
| Caddy health | 100% | 104ms |
| TURN UDP | 100% | 3ms |
| BearDog RPC | 100% | 3ms |
| VPS content TTFB | — | 67ms |
| GitHub TTFB | — | 111ms |

## E4: Membrane Security Audit (darkforest_membrane)

**Methodology**: Remote VPS audit via SSH probing 13 security controls.
Validates cellMembrane (157.230.3.183) against adversarial configuration.

**Script**: `validation/darkforest_membrane.sh`
**Result (May 19, 2026)**: **17 PASS, 0 FAIL, 1 SKIP** (SKIP: b3sum not on VPS)

| Check | Description | Result |
|-------|-------------|--------|
| MEM-01 | SSH password auth disabled | PASS |
| MEM-02 | fail2ban active (3 bans) | PASS |
| MEM-03 | UFW deny-default, expected ports only | PASS |
| MEM-04 | TURN unauthenticated allocate rejected | PASS |
| MEM-05 | exim4/droplet-agent/snapd absent | PASS |
| MEM-06 | journald persistent | PASS |
| MEM-07 | Credentials 600/root | PASS |
| MEM-08 | RustDesk hbbs/hbbr active | PASS |
| MEM-09 | BLAKE3 binary integrity | SKIP |
| MEM-10 | No unexpected TCP listeners | PASS |
| MEM-11–13 | Service-specific checks | PASS |

## E5: lithoSpore Cross-Tier Parity

**Methodology**: 7 LTEE science modules validated at Tier 2 (Rust) with
cross-tier parity against Tier 1 (Python). Tier 3 wired for provenance
trio (rhizoCrypt + loamSpine + sweetGrass).

**Location**: `gardens/lithoSpore/`
**Result**: **7/7 modules PASS** (75/75 checks, 117 tests), **7/7 parity MATCH**

| Module | Science | Checks | Tests | Parity |
|--------|---------|--------|-------|--------|
| 1 | Growth Dynamics | 12 | 18 | MATCH |
| 2 | LTEE Mutations | 10 | 15 | MATCH |
| 3 | Fitness Trajectories | 11 | 17 | MATCH |
| 4 | Citrate Evolution | 10 | 16 | MATCH |
| 5 | BioBricks Assembly | 8 | 13 | MATCH |
| 6 | DNA Damage Response | 12 | 19 | MATCH |
| 7 | Horizontal Transfer | 12 | 19 | MATCH |

**Tier 3 wiring**: `try_record_tier3()` with `primals_reached` tracking,
graceful degradation per `wateringHole/DEGRADATION_BEHAVIOR_STANDARD.md`.

---

## Aggregate Test Summary

```
darkforest           34 tests  (crypto, check, report, discovery)
tunnelKeeper         21 tests  (config, crypto, health)
lithoSpore          117 tests  (7 modules, cross-tier parity)
darkforest_membrane  17 checks (VPS security audit)
benchScale            4 tracks (shadow parity, 15+ reports)
5-layer security    267 checks (pentest, fuzz, crypto, observer, gate)
───────────────────────────────────────────────────────────────────
Total               460+ validations, 0 failures
```

All crates: `#![forbid(unsafe_code)]`, zero clippy warnings (pedantic+nursery),
cargo fmt clean, graphs synchronized to primalSpring v3.0.0.

---

## Future Validation Goals

### Near-term (stadial — measured in weeks)

| Goal | Experiment | Gate Criteria |
|------|-----------|---------------|
| S1 TLS 7-day window | E3 | BearDog p95 ≤ 1.5× CF TTFB p95 for 7 consecutive days |
| S2 dual-path shadow | E3 | Songbird TURN + cloudflared in parallel, measure divergence |
| S3 content mirror | E3 | SHA-256 content hash match + TTFB ≤ 110% GH Pages |
| S4 auth integration | E3 | BearDog dual-auth latency < 50ms p95 for 7 days |
| ACME Phase 3 | E1/E3 | Automated cert renewal (12h check, 30-day-before-expiry) |
| Cron telemetry | E3 | `membrane_telemetry.sh` every 15min → 7-day rolling baseline |

### Medium-term (stadial — measured in months)

| Goal | Experiment | Gate Criteria |
|------|-----------|---------------|
| Sovereign DNS | E3 | knot-dns on VPS replaces Cloudflare DNS (H2-17→20) |
| Cross-gate mesh | E2/E3 | 2+ gates connected via Songbird relay (exp073 covalent pattern) |
| Forgejo Actions CI | — | CI pipelines sovereign (Forgejo replaces GitHub Actions) |
| Membrane auto-healing | E4 | Rolling baselines auto-detect sovereignty regression |
| Ferment transcript E2E | E5 | wetSpring braid → lithoSpore ingestion → USB artifact chain |

### Long-term (glacial)

| Goal | Experiment | Gate Criteria |
|------|-----------|---------------|
| Full DNS cutover | E3 | All 4 shadow tracks pass simultaneously → DNS switch |
| VPS mesh | E4 | Multiple VPS in multiple geos, Tower on each |
| biomeOS as init | — | Phase 4: biomeOS orchestrates full system lifecycle |
| BTC/ETH anchoring | — | Provenance pipeline → OP_RETURN on Bitcoin/Ethereum |
| 10G backbone | — | Sovereign HPC: southGate + NVMe RAID (ABG surface) |

### Ongoing validation (continuous)

- `membrane_telemetry.sh` → `membrane_summary.sh` → `membrane_7day.toml` (15-min cadence)
- `darkforest_membrane.sh` run after every VPS change
- `shadow_run_orchestrator.sh --parity-only` weekly until cutover
- lithoSpore parity maintained as upstream springs evolve

---

## VPS Services (cellMembrane: 157.230.3.183)

| Service | Port | systemd Unit | Status |
|---------|------|-------------|--------|
| BearDog (Tower crypto) | TCP :9100 | `beardog-membrane.service` | RUNNING (v0.9.0) |
| BearDog TLS shadow | TCP :8443 | `beardog-tls-shadow.service` | RUNNING (v0.9.0) |
| SkunkBat (audit) | TCP :9140 | `skunkbat-membrane.service` | RUNNING |
| Songbird TURN relay | UDP :3478 | `songbird-relay.service` | RUNNING (v0.2.1) |
| Caddy TLS | TCP :80, :443 | `caddy-tls.service` | RUNNING (ACME) |
| petalTongue web | TCP :8080 | `petaltongue-web.service` | RUNNING |
| RustDesk hbbs | TCP :21115-21116 | `hbbs-membrane.service` | RUNNING |
| RustDesk hbbr | TCP :21117-21119 | `hbbr-membrane.service` | RUNNING |

Resources: 1.9 GB RAM (331 MB used), 9.7 GB disk (21% used), Ubuntu 24.04.
