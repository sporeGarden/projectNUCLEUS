# projectNUCLEUS

The deployable NUCLEUS infrastructure product. Stand up a sovereign compute
node, host sites, dispatch workloads, and progressively eliminate every
external dependency.

**Organization**: sporeGarden (products built on ecoPrimals)
**Generation**: gen4 — composition and deployment
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What This Is

projectNUCLEUS is the deployable NUCLEUS infrastructure on **ironGate** — our
local development and validation system. It takes primal binaries from
plasmidBin, composition graphs from primalSpring, and standards from
wateringHole, and assembles them into a running gate on real hardware.

```
primalSpring (upstream patterns)
       ↓ deploy graphs, validation, standards
projectNUCLEUS on ironGate (deploys + validates patterns)
       ↓ real workloads, real users
ABG collaborators (ionic compute sharing = pattern validation under load)
```

**The core loop**: primalSpring defines composition patterns upstream.
projectNUCLEUS deploys those patterns on ironGate. ABG workloads validate
them under real external load. Gaps flow back upstream via handoff docs.
Every successful ABG workload is proof that primalSpring's deploy graphs,
BTSP, discovery hierarchy, and provenance pipeline work in production.

## NUCLEUS Atomics

NUCLEUS composes from three atomics, each named for a subatomic particle:

| Atomic | Particle | Primals | Role |
|--------|----------|---------|------|
| **Tower** | Electron | BearDog + Songbird | Trust boundary — crypto, identity, networking, BTSP |
| **Node** | Proton | Tower + ToadStool + barraCuda + coralReef | Compute — workload dispatch, GPU math, shader compilation |
| **Nest** | Neutron | Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass | Storage — content-addressed data, provenance, attribution |

Full NUCLEUS = Tower + Node + Nest + Squirrel (AI) + biomeOS (orchestration).

**fieldMouse** is NOT a primal — it is a deployment class (biomeOS chimeras for edge/IoT).
Do not include fieldMouse in primal rosters.

Every gate runs the atomics that match its hardware. A NUC runs Tower. A
workstation runs Node. A storage server runs Nest. biomeOS coordinates
the mesh.

## Bonding Model

Gates connect to each other through chemical bonding patterns:

| Bond | Trust | Use Case |
|------|-------|----------|
| **Covalent** | Shared family seed, full trust | Basement LAN cluster — your machines |
| **Ionic** | Metered, scoped access | Friend's GPU, ABG compute sharing |
| **Metallic** | Delocalized capabilities | Institutional HPC (ICER), datacenter fleet |
| **Weak** | Pre-trust, external APIs | Dark Forest beacons, initial contact |

## Current State

**Phase 59 absorbed (2026-05-06)** — Full Ecosystem Convergence from primalSpring v0.9.24:
- All **13/13 NUCLEUS primals** deployed and healthy on ironGate: BearDog, Songbird, ToadStool, barraCuda, coralReef, NestGate, rhizoCrypt, loamSpine, sweetGrass, Squirrel, skunkBat, biomeOS, petalTongue
- BTSP Phase 3 AEAD (ChaCha20-Poly1305), Wire Standard L3, discovery escalation — all converged
- **5-tier discovery hierarchy**: Songbird IPC → biomeOS Neural → UDS convention → socket registry → TCP probing
- Deploy graphs carry `tcp_fallback_port`, `bonding_policy`, `security_model` per node — validated by `primalspring_guidestone`
- 235+ wetSpring science checks passing across 10 workloads (8 Rust validators, 2 Python baselines)
- Full provenance chain operational: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid
- **Phase 2a validated**: JupyterHub + Cloudflare Tunnel, 15/15 external checks, 270ms p50 latency
- **ABG tiered access**: observer / compute / admin via PAM groups and `pre_spawn_hook`
- **Security**: Zero open gaps — all PG-55 through PG-59 resolved by primalSpring Phase 59. 13/13 default `127.0.0.1`, NestGate BTSP method-level auth, skunkBat multi-dimensional anomaly detection
- Capability-based discovery (`by_capability`) preferred over identity-based (`name`) in all graphs
- TCP fallback ports aligned to Phase 59 canonical table (skunkBat 9750→9140, port swaps corrected)

## Quick Start

```bash
# Deploy a Node Atomic to the current machine
cd deploy/
bash deploy.sh --composition node --gate mygate

# Execute a workload through toadStool
toadstool execute workloads/wetspring/wetspring-16s-rust-validation.toml
```

See [deploy/](deploy/) for full deployment instructions.

## Phase Roadmap

### Phase 1: Covalent LAN HPC (validated)

13 primals on ironGate (Full NUCLEUS) with provenance pipeline.
235+ wetSpring science checks passing. Full provenance chain operational.
This proves the substrate works on our hardware.

### Phase 2: Ionic Compute Sharing (in progress — Step 2a validated)

Deploy a usable system for ABG as validation of primalSpring patterns.
NUC intake → ironGate JupyterHub → BTSP-secured access via primals.eco.
Step 2a: Cloudflare Tunnel baseline captured (270ms p50, 15/15 external checks).
ABG tiered access live (observer/compute/admin). Notebook elevation operational.

### Phase 3: Self-Hosted sporePrint

petalTongue replaces Zola for primals.eco hosting. BTSP Phase 3 encrypted
transport replaces Tailscale/Cloudflare. songBird NAT traversal replaces
WireGuard tunnels.

### Phase 4: Full NUCLEUS Desktop Substrate

biomeOS as orchestrator across all gates. Metallic bonding for institutional
HPC. All sporeGarden products running on projectNUCLEUS. sunCloud metabolic
economics. Zero external dependencies.

See [PHASES.md](PHASES.md) for detailed phase architecture.

## Repo Structure

```
specs/              Local specs: execution model, composition contract, invisibility, security, tunnel evolution
gates/              Gate inventory and hardware configs
deploy/             Deployment tooling (deploy.sh, provenance_pipeline.sh, security_validation.sh, abg_accounts.sh)
graphs/             Curated deploy graph TOMLs (from primalSpring)
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads (8 Rust + 2 Python + 1 deferred)
  templates/        Templates for new workloads
validation/         Composition validation results, provenance manifests, security handbacks, gap reports
docs/               Architecture primers and external-facing docs
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **primalSpring** | syntheticChemistry | Upstream — defines composition patterns that projectNUCLEUS deploys and validates |
| **plasmidBin** | ecoPrimals/infra | Binary depot — projectNUCLEUS fetches primal binaries from here |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — projectNUCLEUS follows these |
| **sporePrint** | ecoPrimals/infra | The website (primals.eco) — projectNUCLEUS eventually hosts it |
| **foundation** | sporeGarden | External-facing — institutional relationships, metallic federation |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
