# projectNUCLEUS Experiments

Validation experiments for sovereign NUCLEUS deployment.

## Active Experiments

| ID | Name | Crate/Tool | Tests | Status |
|----|------|-----------|-------|--------|
| E1 | Security boundary validation | `validation/darkforest/` | 140 | PASS |
| E2 | Transport sovereignty | `validation/tunnelKeeper/` | 48 | PASS |
| E3 | Sovereignty parity | `infra/benchScale/` | 4 tracks | ACTIVE |
| E4 | Membrane security audit | `validation/darkforest_membrane.sh` | 17 checks (21 PASS) | PASS |
| E5 | lithoSpore integration | `gardens/lithoSpore/` | 117 | PASS |

---

## E1: Dark Forest Security Validation

**Methodology**: Pure Rust security validator probes all 13 primals and
JupyterHub via JSON-RPC fuzz, HTTP pentest, crypto validation, and
observer-tier access control.

**Rust crate**: `validation/darkforest/` (v0.2.1)
**Test coverage**: 140 unit tests (Shannon entropy, hex decode, check builder,
report roundtrip, discovery module, capability probing, HTTP parsing, net graceful failure)

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
**Test coverage**: 48 unit tests (YAML config roundtrip, ChaCha20 encrypt/decrypt,
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
`shadow_run_orchestrator.sh`, measured by `nucleus-deploy telemetry` (was `membrane_telemetry.sh`), gated by
7-day rolling window via `nucleus-deploy summary` (was `membrane_summary.sh`).

**Location**: `infra/benchScale/`
**Deploy graph**: `graphs/sovereignty_shadow.toml`
**Protocol**: `wateringHole/SOVEREIGNTY_STANDARDS.md` §2

### Shadow Matrix (May 23, 2026)

| # | Track | Sovereign | Commercial | Status | Measured |
|---|-------|-----------|------------|--------|----------|
| S1 | TLS termination | BearDog :8443 (rustls) | Cloudflare TLS | **LIVE** | 13ms RPC vs 163ms CF TTFB |
| S2 | NAT traversal | Songbird TURN relay | cloudflared tunnel | **LIVE** | 100% reachable (3ms UDP) |
| S3 | Content hosting | NestGate + petalTongue | GitHub Pages | **LIVE** | 68ms VPS TTFB vs 111ms GH |
| S4 | Auth / JupyterHub | BearDog BTSP dual-auth | OAuth2 proxy | **SHADOW LIVE** | Dual-auth shadow active, events accumulating; full cutover pending |
| S5 | DNS resolution | knot-dns (DNSSEC) | Cloudflare NS | **LIVE** | 45ms authoritative, ECDSAP256SHA256 |

### Orchestrator Progression

```
Run 1-4 (May 19): 4 PASS, 0 FAIL, 1 SKIP (knot-dns not yet deployed)
Run 5   (May 22): 5 PASS, 0 FAIL, 1 SKIP (DNS deployed, probe stabilizing)
Run 6   (May 22): 6 PASS, 0 FAIL, 0 SKIP — FULL PASS (all 5 tracks + DNS)
```

### Parity Reports

| Track | Report | Key metric | Gate met? |
|-------|--------|------------|-----------|
| S1 TLS | `btsp_tls_parity_*.toml` | BearDog RPC 13ms vs CF 163ms | **YES** — 12x faster |
| S2 NAT | `songbird_nat_parity_*.toml` | TURN reachability: 100% | **YES** — dual-path operational |
| S3 Content | `nestgate_content_parity_*.toml` | VPS 68ms vs GH 111ms TTFB | **YES** — parity exceeded |
| S4 Auth | BTSP shadow telemetry | Dual-auth active, events logged | **ACTIVE** — 7-day window running |
| S5 DNS | `dot_parity_*.toml` | knot-dns 45ms, DNSSEC signed | **YES** — H2-17 DEPLOYED |

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
**Result (May 22, 2026)**: **21 PASS, 0 FAIL, 1 SKIP** (SKIP: b3sum not on VPS)

| Check | Description | Result |
|-------|-------------|--------|
| MEM-01 | SSH password auth disabled | PASS |
| MEM-02 | fail2ban active | PASS |
| MEM-03 | UFW deny-default, expected ports only | PASS |
| MEM-04 | TURN unauthenticated allocate rejected | PASS |
| MEM-05 | exim4/droplet-agent/snapd absent | PASS |
| MEM-06 | journald persistent | PASS |
| MEM-07 | Credentials 600/root | PASS |
| MEM-08 | RustDesk hbbs/hbbr active | PASS |
| MEM-09 | BLAKE3 binary integrity | SKIP |
| MEM-10 | No unexpected TCP listeners | PASS |
| MEM-11–13 | Service-specific checks | PASS |
| MEM-14 | NestGate health (:9500 REST) | PASS |
| MEM-15 | rhizoCrypt health (:9602 JSON-RPC) | PASS |
| MEM-16 | loamSpine health (:9700 HTTP) | PASS |
| MEM-17 | sweetGrass health (:9850 TCP) | PASS |

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
darkforest          140 tests  (check, crypto, discovery, fuzz, net, observer, pentest, report)
tunnelKeeper         48 tests  (api, config, crypto, health, transport)
nucleus-deploy       47 tests  (security, provenance, deploy, spore, telemetry, util)
nucleus-primals       7 tests  (shared primal registry)
lithoSpore          117 tests  (7 modules, cross-tier parity)
darkforest_membrane  21 checks (VPS security audit, Nest Atomic)
benchScale            5 tracks (shadow parity, 25+ reports)
5-layer security    267 checks (pentest, fuzz, crypto, observer, gate)
───────────────────────────────────────────────────────────────────
Total               584+ validations, 0 failures
```

All crates: `#![forbid(unsafe_code)]`, zero clippy warnings (pedantic+nursery),
cargo fmt clean, graphs synchronized to primalSpring v0.9.30 (Wave 56), `deny.toml`
on both crates, `secure_by_default` 12/12 deploy graphs. 460 methods registered.
Deploy tooling `--uds-only` VPS standard (Wave 56).

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
| Cron telemetry | E3 | `nucleus-deploy telemetry` every 15min → 7-day rolling baseline (was `membrane_telemetry.sh`) |

### Wave 56 — Niche Climate (NC-1→NC-5, pre-stadial)

| Goal | NC | Gate Criteria | Status |
|------|-----|---------------|--------|
| `biomeos nucleus ingest` on ironGate VPS | NC-1 | hotSpring pseudoSpore v1.6.1 → NUCLEUS column U | **CODE COMPLETE** — biomeOS v3.84 shipped. Live column U gated on VPS deploy |
| southGate 13/13 health | NC-2.1 | Songbird mesh seed fix, bidirectional SONGBIRD_PEERS | 7/13 responding |
| Cross-gate capability call via cellMembrane | NC-2.3 | ironGate ↔ eastGate ↔ southGate mesh | OPEN |
| knot-dns NS cutover to primary | NC-3.3 | Registrar NS delegation → sovereignty DNS (S5) | knot-dns DEPLOYED; NS cutover pending registrar |
| Forgejo binary releases | NC-3.4 / H3-02 | plasmidBin `auto-harvest.yml` publishes to Forgejo | Coordinate with plasmidBin |
| sporePrint via NestGate | NC-3.5 | BearDog `content.*` scope → `publish_sporeprint.sh` | BLOCKED on BearDog scope |

### Medium-term (stadial — measured in months)

| Goal | Experiment | Gate Criteria |
|------|-----------|---------------|
| ~~Sovereign DNS~~ | ~~E3~~ | ~~knot-dns on VPS~~ → **DEPLOYED** (H2-17, May 22). NS cutover H2-18 pending registrar. |
| Cross-gate mesh | E2/E3 | 3+ gates connected via Songbird relay (NC-2.5 bidirectional seeding) |
| Forgejo Actions CI | — | CI pipelines sovereign (Forgejo replaces GitHub Actions). **Wave 59 gap**: git is Forgejo-primary but CI/CD is still GitHub Actions (glacial gate observation, not stadial blocker) |
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

- `nucleus-deploy telemetry` → `nucleus-deploy summary` → `membrane_7day.toml` (15-min cadence; was `membrane_telemetry.sh` → `membrane_summary.sh`)
- `darkforest_membrane.sh` run after every VPS change
- `shadow_run_orchestrator.sh --parity-only` weekly until cutover
- lithoSpore parity maintained as upstream springs evolve

---

## VPS Services (cellMembrane: 157.230.3.183) — Nest Atomic (May 22)

| Service | Port | systemd Unit | Status |
|---------|------|-------------|--------|
| BearDog (Tower crypto) | TCP :9100 | `beardog-membrane.service` | RUNNING (v0.9.0) |
| BearDog TLS shadow | TCP :8443 | `beardog-tls-shadow.service` | RUNNING (v0.9.0) |
| SkunkBat (audit) | TCP :9140 | `skunkbat-membrane.service` | RUNNING |
| Songbird TURN relay | UDP :3478 | `songbird-relay.service` | RUNNING (v0.2.1) |
| NestGate (storage) | TCP :9500 | `nestgate-membrane.service` | RUNNING (v2.1.0) |
| rhizoCrypt (DAG) | TCP :9602 | `rhizocrypt-membrane.service` | RUNNING (v0.14.0) |
| loamSpine (ledger) | TCP :9700 | `loamspine-membrane.service` | RUNNING (v0.9.16) |
| sweetGrass (braid) | TCP :9850 | `sweetgrass-membrane.service` | RUNNING (v0.7.34) |
| Caddy TLS | TCP :80, :443 | `caddy-tls.service` | RUNNING (ACME) |
| petalTongue web | TCP :8080 | `petaltongue-web.service` | RUNNING |
| RustDesk hbbs/hbbr | TCP :21115-21119 | `hbbs/hbbr-membrane.service` | RUNNING |

Resources: 2 GB RAM (~400 MB used), 1.6 GB free. Ubuntu 24.04.
