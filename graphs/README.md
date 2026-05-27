# Deploy Graphs

Curated subset of primalSpring/graphs/ for projectNUCLEUS deployment.
The canonical source for all 77+ graphs remains in primalSpring.

**Dark Forest compliance (2026-05-27):**
- All 12/12 graphs carry `secure_by_default = true` (fragments + deploy + workflow) (DF-4 requirement)
- No non-Songbird nodes advertise `http` or `tls` capabilities (DF-3)
- All nodes declare `security_model = "btsp"` or `"tower_delegated"` (DF-4)
- UDS-only default transport, TCP fallback opt-in via `PRIMALSPRING_TCP_TIER5` (DF-2)

**Phase 59 updates (2026-05-06):**
- All deploy graph nodes carry `tcp_fallback_port` for Tier 5 discovery
- Graph-level `bonding_policy = "btsp_required"` validates against node `security_model`
- All 13 primals converged to BTSP Phase 3 AEAD — `security_model = "btsp"` is universal
- Wire Standard L3: every primal's `capabilities.list` returns `protocol` + `transport`

## Fragments (building blocks)

| File | Particle | Primals |
|------|----------|---------|
| `tower_atomic.toml` | Electron | BearDog + Songbird + skunkBat |
| `node_atomic.toml` | Proton | Tower + ToadStool + barraCuda + coralReef |
| `nest_atomic.toml` | Neutron | Tower + NestGate + provenance trio |
| `nucleus.toml` | Full atom | Tower + Node + Nest (10 domain primals; see `nucleus_complete.toml` for full 13) |

## Deployment Graphs

| File | Composition | Phase |
|------|-------------|-------|
| `node_atomic_compute.toml` | biomeOS + Node Atomic (v2.0.0) | Phase 1 |
| `nucleus_complete.toml` | Full NUCLEUS with bonding policy (v2.0.0) | Phase 4 |

## Bonding Patterns

| File | Bond Type | Use Case |
|------|-----------|----------|
| `basement_hpc_covalent.toml` | Covalent | LAN cluster — your machines |
| `friend_remote_covalent.toml` | Covalent + NAT | Friend's machine with tunnel |
| `ionic_capability_share.toml` | Ionic | Two-family metered sharing |
| `sovereignty_shadow.toml` | Shadow | 5-track sovereignty parity proofs (TLS/NAT/content/auth/DNS) — **6/0/0 FULL PASS** |

## TCP Fallback Ports (Tier 5 Discovery)

| Primal | Port | Primal | Port |
|--------|------|--------|------|
| BearDog | 9100 | loamSpine | 9700 |
| Songbird | 9200 | coralReef | 9730 |
| Squirrel | 9300 | barraCuda | 9740 |
| toadStool | 9400 | skunkBat | 9140 |
| NestGate | 9500 | biomeOS | 9800 |
| rhizoCrypt | 9601 | sweetGrass | 9850 |
| petalTongue | 9900 | | |

## Signal Graphs (Wave 55 — Niche Climate)

| File | Signal | Steps | Status |
|------|--------|-------|--------|
| `nest_ingest_spore.toml` (biomeOS canonical) | `nucleus.ingest_spore` | validate → store → DAG → ledger → braid → sign | NC-1; BLOCKED on biomeOS gateway |

The `nest_ingest_spore` signal graph originates from `primals/biomeOS/graphs/signals/` (also copied in primalSpring).
It composes existing primal capabilities — no new capabilities required. Six sequential nodes:
NestGate `storage.exists` + `content.put` → rhizoCrypt `dag.session.create` → loamSpine `entry.append` →
sweetGrass `braid.create` → BearDog `crypto.sign`. Output: `receipts/nucleus_ingest.toml`.

projectNUCLEUS ironGate is a deployment target — when biomeOS v3.77+ gateway completes (NC-1),
run `biomeos nucleus ingest <pseudoSpore-dir>` on ironGate VPS to close NUCLEUS_VALIDATION_MATRIX column U.

## Source

All graphs originate from:
`ecoPrimals/springs/primalSpring/graphs/`
