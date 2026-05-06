# Phase Architecture — projectNUCLEUS

Each phase builds on the previous. No phase is thrown away — Phase 1 covalent
bonding remains the foundation even when Phase 4 metallic federation is live.

---

## Phase 1: Covalent LAN HPC

**Status**: Validated (2026-05-06), full provenance pipeline operational
**System**: ironGate (i9-14900K, 96 GB DDR5, RTX 5070)
**Bonding**: Covalent (shared family seed, full trust)
**Composition**: Full NUCLEUS (13 primals, dynamically discovered from plasmidBin)
- Tower: BearDog + SongBird
- Compute: ToadStool + BarraCuda + CoralReef
- Storage: NestGate
- Provenance: rhizoCrypt + loamSpine + sweetGrass
- Coordination: Squirrel
- Defense: skunkBat
- Meta: biomeOS (orchestration) + petalTongue (UI)

ironGate is our local development and validation system. Phase 1 proves
that primalSpring's composition patterns work on real hardware with real
science workloads.

### What Works

- Full NUCLEUS (13 primals) deployed via `deploy.sh --composition full --gate irongate`
- toadStool dispatches workloads from TOML specs (native runtime)
- wetSpring science validated through composition dispatch (235+ checks):
  - 16S Pipeline: 37/37 checks PASS
  - Diversity Metrics: 27/27 checks PASS
  - Gonzales CPU Parity: 43/43 checks PASS
  - Algae 16S: 34/34 checks PASS (real NCBI data — 11.9M reads)
  - R Industry Parity: 53/53 checks PASS
  - Fajgenbaum Pathway: 8/8 checks PASS
  - Cold Seep Pipeline: 8/8 checks PASS
  - Real NCBI Pipeline: 25/25 checks PASS
  - Python 16S baseline: SUCCESS (real NCBI data — 50K reads)
  - Python benchmark baseline: SUCCESS (4.15s, all domains)
- Full provenance pipeline operational (9 phases):
  - BLAKE3 content hashes for all NCBI FASTQs and outputs (5.4 GB)
  - rhizoCrypt DAG session with 26 events
  - Merkle root: `b106aa1d1bb45430d00d605626e10488119f9e4f9f315a738939049a6da9ceec`
  - loamSpine permanent ledger commit (index 32)
  - sweetGrass attribution braid with ed25519 witness (`urn:braid:b106aa1d...`)
  - PROV-O compliant braid with DID attribution
- JupyterHub running for notebook-style access (3 kernels: Python, bioinfo, R)
- ABG validation notebook deployed (`~/notebooks/abg-wetspring-validation.ipynb`)
- Cat6e ethernet between cluster machines

### What projectNUCLEUS Provides

- `deploy.sh`: Automates seed creation, primal startup, health verification
- `provenance_pipeline.sh`: Wraps workload execution with full provenance chain
- Gate manifest: Hardware inventory with atomic assignment
- Workload catalog: Validated TOML specs + templates for new workloads
- Validation logs: Reproducible evidence of composition correctness
- Provenance manifests: BLAKE3 hashes, Merkle roots, braid IDs

### Known Gaps (from COMPOSITION_GAPS.md)

- Sandbox overrides working_dir to /tmp (absolute paths required)
- GPU build breakage (wgpu submit_and_poll → submit_and_map API drift)
- toadStool does not expand environment variables in workload TOMLs
- Python runs as native subprocess (no toadStool introspection)
- `wetspring-exp001-python-baseline` FAIL (missing dependencies — wetSpring issue)

### Security Posture (from `specs/SECURITY_VALIDATION.md`)

Three-layer pen testing baseline captured (2026-05-06):

| Layer | Key Finding | Action |
|-------|-------------|--------|
| Below (OS) | Primals bind 0.0.0.0, UFW inactive | Activate UFW, rebind when NucBox goes live |
| At (APIs) | All primals survive fuzzing, no hidden methods | NestGate storage.list needs BTSP scoping |
| Above (App) | Auth enforced, CSP present | Add missing security headers |

- `deploy/security_validation.sh` — automated three-layer pen testing
- skunkBat observes all scans (training data for baseline learning)
- Run before and after every tunnel evolution step

---

## Phase 2: Ionic Compute Sharing

**Status**: Step 2a validated (2026-05-06) — Cloudflare Tunnel baseline captured
**System**: ironGate + NUC intake
**Bonding**: Ionic (metered, scoped access)
**New Primals**: songBird cross-gate routing, BTSP Phase 3 AEAD (all 13 primals converged)

Deploy a usable system for ABG as validation of primalSpring patterns.
Phase 1 proved the substrate works internally. Phase 2 proves it works
under real external load — ionic bonding, BTSP tunnels, cross-gate
routing, and provenance on collaborator workloads.

### Architecture

```
[ABG / External User]
       ↓ BTSP Phase 3 tunnel
[NUC Intake / Tower Atomic]
       ↓ Cat6e LAN (covalent internal)
[ironGate / JupyterHub + Nest Atomic + provenance]
       ↓ songBird cross-gate routing
[strandGate or southGate / heavy compute]
```

- An expendable NUC intake node runs Tower Atomic (tunnel termination, reverse proxy)
- songBird routes workloads from intake to compute gates over covalent LAN
- BTSP Phase 3 encrypts the full channel (ChaCha20-Poly1305 AEAD)
- JupyterHub proxied through NUC intake, backed by internal compute
- Ionic bonding: capability tokens scope access — no family seed sharing

### Sub-Goal: Sovereign JupyterHub via primals.eco

**Target**: ABG collaborator navigates to `primals.eco/compute`, authenticates
via BTSP, and gets a JupyterHub notebook environment on ironGate:

```
Browser → primals.eco → BTSP tunnel (BearDog + Songbird NAT traversal)
       → petalTongue reverse proxy → JupyterHub on ironGate
       → notebook kernels (Python bioinformatics, R Seurat/DESeq2)
       → toadStool dispatch for heavy workloads → strandGate
       → provenance pipeline (rhizoCrypt → loamSpine → sweetGrass)
```

**Security chain**: The collaborator holds a scoped ionic capability token,
not a family seed. They can run notebooks and submit workloads but cannot
discover internal gates, access raw storage, or see the mesh topology.

**Tunnel evolution** (progressive sovereignty):

| Step | Tunnel | External Dependency | Validation Target |
|------|--------|---------------------|-------------------|
| 2a | Cloudflare Tunnel (`cloudflared`) | Cloudflare edge | Baseline latency, uptime, request metrics |
| 2b | + BTSP auth inside CF tunnel | Cloudflare edge + BearDog | Ionic tokens scope access correctly |
| 2c | WireGuard replaces `cloudflared` | None (manual keys) | NAT punch-through matches CF reliability |
| 2d | Songbird NAT traversal | STUN relay (self-hostable) | Songbird matches WireGuard throughput |
| 2e | Full BTSP tunnel | **Zero** — BearDog keys + Songbird transport | BTSP handshake < 200ms, zero externals |

**Why Cloudflare first**: `cloudflared` gives us HTTPS termination, DDoS
protection, and zero NAT configuration for free. The Cloudflare dashboard
provides baseline metrics that become parity targets for each replacement
step. We don't remove Cloudflare until we prove each primal matches or
exceeds its behavior under real ABG load. See `specs/TUNNEL_EVOLUTION.md`.

**Existing infrastructure** (from `wateringHole/compute-sharing/`):
- `setup-jupyterhub.sh` — portable JupyterHub with Python + R kernels
- `TUNNEL_ACCESS_GUIDE.md` — Tailscale/SSH/WireGuard options
- `SOVEREIGN_COMPUTE_SHARING.md` — full pattern doc (Phase 0-4)
- JupyterHub validated on ironGate (Phase 1)

### Step 2a Validated (2026-05-06)

Cloudflare quick tunnel established and tested end-to-end:
- `cloudflared` 2026.3.0 → QUIC tunnel → Cloudflare ORD edge → public URL
- All JupyterHub endpoints accessible externally (HTTP 200)
- Tunnel latency p50: **270ms** (5-sample median via trycloudflare.com)
- 13/13 primals healthy behind the tunnel
- Provenance pipeline (run 2): 26 events, Merkle root, ed25519 braid
- External validation: **15/15 PASS**
- Local validation: **12/12 PASS**
- `deploy/external_validation.sh` — automated validation for both modes
- See `validation/EXTERNAL_PIPELINE_VALIDATION_MAY06_2026.md`

### ABG Tiered Access Model

JupyterHub supports three access tiers via Linux groups (`deploy/abg_accounts.sh`):

| Tier | Group | Resources | Can Execute | Can Visualize |
|------|-------|-----------|-------------|---------------|
| observer | `abg-observer` | 8 GB / 4 cores | No | Yes (cached results) |
| compute | `abg-compute` | 32 GB / 8 cores | Yes (via ToadStool) | Yes |
| admin | `abg-admin` | 48 GB / 16 cores | Yes | Yes + raw API access |

`pre_spawn_hook` in JupyterHub sets per-user resource limits, `NUCLEUS_TIER`
environment variable, and primal port configuration. Observer-tier users
get `NUCLEUS_READONLY=1` and see pre-computed results only.

### Notebook Elevation (from `specs/NOTEBOOK_ELEVATION.md`)

Validation binaries elevate through four tiers:

```
Tier 0: CLI binary → stdout [OK]/[FAIL]        (current — all springs)
Tier 1: + notebook visualization via matplotlib  (current — wetSpring)
Tier 2: + JSON-RPC primal APIs for live queries  (evolution target)
Tier 3: + petalTongue web dashboards             (long-term)
```

Available notebooks:
- `abg-wetspring-validation.ipynb` — workload submission + provenance inspection
- `wetspring-validation-viz.ipynb` — full visualization dashboard with charts

### Relevant Deploy Graphs

Curated in projectNUCLEUS/graphs/ (canonical source: primalSpring/graphs/):
- `ionic_capability_share.toml` — two-family ionic bridge (Lab Alpha ↔ University Beta)
- `friend_remote_covalent.toml` — friend compute sharing with NAT escalation
- `basement_hpc_covalent.toml` — LAN cluster covalent mesh

### What This Unlocks

- ABG collaborators submit workloads through the ionic tunnel
- The "lend a GPU to a friend" pattern — anyone with hardware can participate
- Egress, multi-user, and security patterns validated under real external load
- songBird cross-gate dispatch exercised for the first time
- Full provenance chain on external workloads (BLAKE3 → DAG → ledger → braid)
- External science outputs become validation targets for Rust reimplementations

---

## Phase 3: Self-Hosted sporePrint

**Status**: Near-term (sporePrint ownership moving to projectNUCLEUS;
NestGate + provenance trio operational from Phase 1)
**Bonding**: Covalent core + public weak endpoint
**New Primals**: petalTongue (UI), BTSP Phase 3

projectNUCLEUS takes short-term ownership of sporePrint. The site
evolves from GitHub Pages → NestGate-backed content → petalTongue
rendering, each step validated against Cloudflare CDN baselines.

### Architecture

```
[Browser] → primals.eco (Cloudflare DNS, then sovereign)
         → [NucBox M6 intake / cloudflared → petalTongue]
         → [ironGate / NestGate content store + JupyterHub]
```

### Convergence Path

```
Now:     GitHub Pages + Cloudflare CDN  →  static Zola site
Phase 2: + Cloudflare Tunnel            →  JupyterHub for ABG (/compute)
Phase 3a: sporePrint content → NestGate  →  remove GitHub Pages dependency
Phase 3b: petalTongue replaces Zola      →  dynamic site + compute dashboard
Phase 3c: BTSP replaces Cloudflare TLS   →  Cloudflare reduced to DNS-only
Phase 3d: Songbird NAT replaces CF tunnel → self-hosted, Cloudflare optional
Phase 4:  sovereign DNS                  →  zero external dependencies
```

### What This Eliminates (progressively)

- GitHub Pages dependency (content served from own hardware)
- Cloudflare TLS dependency (BTSP handles termination)
- Cloudflare Tunnel dependency (Songbird handles NAT traversal)
- Cloudflare DNS dependency (sovereign resolution)

### Validation via benchScale

`infra/benchScale` provides benchmark and penetration testing as each
Cloudflare component is replaced. Validation targets are derived from
Cloudflare dashboard baselines (latency, error rates, throughput).
No component is removed until the primal replacement meets or exceeds
those targets under `benchScale` load.

---

## Phase 4: Full NUCLEUS Desktop Substrate

**Status**: Vision (from BIOMEOS_OS_TRAJECTORY.md)
**Bonding**: All types — covalent core, ionic edges, metallic institutional
**Composition**: Full NUCLEUS + meta tier (biomeOS + Squirrel + petalTongue)

### Architecture

- biomeOS as PID-1-like orchestrator across all gates
- Full NUCLEUS on every capable gate in the cluster
- Metallic bonding for institutional HPC (ICER, university clusters)
- Squirrel coordinates AI workloads across the mesh
- sunCloud metabolic economics: infrastructure compensation via sweetGrass attribution
- All sporeGarden products (helixVision, esotericWebb, RPGPT) running on projectNUCLEUS

### What This Achieves

- Zero external dependencies — fully sovereign infrastructure
- The substrate that gen3 science continues to evolve on
- Desktop experience: petalTongue as the interactive surface
- Institutional federation: universities and labs join as metallic nodes
- Economic sustainability: sunCloud metabolic slices fund infrastructure

---

## Phase Relationship

```
Phase 1 (covalent)     → validates compute dispatch on single gate
Phase 2 (ionic)        → validates cross-gate routing and external access
Phase 3 (self-hosted)  → validates content hosting and BTSP transport
Phase 4 (full NUCLEUS) → validates full mesh orchestration and economics
```

Each phase proves a new bonding model and lights up new primals. The
composition grows monotonically — nothing is removed, only added.
