# Deploy Graphs

Curated subset of primalSpring/graphs/ for projectNUCLEUS deployment.
The canonical source for all 77+ graphs remains in primalSpring.

**Dark Forest compliance (2026-07-15):**
- All 19/19 graphs carry `secure_by_default = true` (fragments + deploy + workflow + compositions) (DF-4 requirement)
- No non-Songbird nodes advertise `http` or `tls` capabilities (DF-3)
- All nodes declare `security_model = "btsp"` or `"tower_delegated"` (DF-4)
- UDS-only default transport, TCP fallback opt-in via `PRIMALSPRING_TCP_TIER5` (DF-2)

**Phase 59 updates (2026-05-06):**
- All deploy graph nodes carry `tcp_fallback_port` for Tier 5 discovery
- Graph-level `bonding_policy = "btsp_required"` validates against node `security_model`
- All 14 primal endpoints (13 binaries; rhizoCrypt serves :9601 + :9602) converged to BTSP Phase 3 AEAD — `security_model = "btsp"` is universal
- Wire Standard L3: every primal's `capabilities.list` returns `protocol` + `transport`

## Bond Type Taxonomy

Every graph carries a `bond_type` field classifying its dominant bonding pattern:

| Bond Type | Model | When Used |
|-----------|-------|-----------|
| `covalent` | Shared family seed, direct trust | Tower internals, same-family cross-gate, workflows |
| `metallic` | Electron sea — tower mediates | Compositions where tower brokers compute/storage |
| `ionic` | Metered capability lease | Cross-family sharing with contracts |
| `hydrogen` | Directional cross-gate (WG mesh) | Same-family remote gates (future) |
| `van_der_waals` | Weak transient | Discovery, observe-only (future) |

## Fragments (building blocks)

| File | Particle | Bond | Primals |
|------|----------|------|---------|
| `tower_atomic.toml` | Electron | covalent | BearDog + Songbird + skunkBat |
| `node_atomic.toml` | Proton | metallic | Tower + ToadStool + barraCuda + coralReef |
| `nest_atomic.toml` | Neutron | metallic | Tower + NestGate + provenance trio |
| `nucleus.toml` | Full atom | metallic | Tower + Node + Nest (10 domain primals; see `nucleus_complete.toml` for full 13) |

## Deployment Graphs

| File | Bond | Composition | Phase |
|------|------|-------------|-------|
| `node_atomic_compute.toml` | metallic | biomeOS + Node Atomic (v2.0.0) | Phase 1 |
| `nucleus_complete.toml` | metallic | Full NUCLEUS with bonding policy (v2.0.0) | Phase 4 |
| `graphene_portable.toml` | covalent | Tower Atomic for `portable_anchor` gates (v0.1.0) | Wave 69 |
| `strand_heavy_compute.toml` | metallic | Tower + Compute + Nest + Provenance — dual EPYC (v1.0.0) | Wave 73 |
| `west_cold_storage.toml` | metallic | Nest Atomic — 76 TB ZFS cold archive (v1.0.0) | Wave 73 |

## Application Compositions (protists)

| File | Bond | Composition | Phase |
|------|------|-------------|-------|
| `sporeprint_composition.toml` | metallic | Nest Atomic + petalTongue content renderer (primals.eco) | Phase 3 |
| `footprint_composition.toml` | metallic | Nest Atomic + petalTongue + songBird drawbridge (GIS planner) | Wave 136b |

Application compositions serve a product. The primals are the backend; the
product frontend runs in the browser. `application = true` in the fragment
metadata distinguishes these from infrastructure compositions.

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
| `nest_ingest_spore.toml` (biomeOS canonical) | `nucleus.ingest_spore` | validate → store → DAG → ledger → braid → sign | NC-1 CODE COMPLETE — biomeOS v4.31 `biomeos-pseudospore` shipped |

The `nest_ingest_spore` signal graph originates from `primals/biomeOS/graphs/signals/` (also copied in primalSpring).
It composes existing primal capabilities — no new capabilities required. Six sequential nodes:
NestGate `storage.exists` + `content.put` → rhizoCrypt `dag.session.create` → loamSpine `entry.append` →
sweetGrass `braid.create` → BearDog `crypto.sign`. Output: `receipts/nucleus_ingest.toml`.

projectNUCLEUS ironGate is a deployment target — when biomeOS v4.31+ gateway completes (NC-1),
run `biomeos nucleus ingest <pseudoSpore-dir>` on ironGate VPS to close NUCLEUS_VALIDATION_MATRIX column U.

## Source

All graphs originate from:
`ecoPrimals/springs/primalSpring/graphs/`
