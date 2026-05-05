# projectNUCLEUS

The deployable NUCLEUS infrastructure product. Stand up a sovereign compute
node, host sites, dispatch workloads, and progressively eliminate every
external dependency.

**Organization**: sporeGarden (products built on ecoPrimals)
**Generation**: gen4 — composition and deployment
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What This Is

projectNUCLEUS packages the proven ecoPrimals substrate into a deployable
system. It is the infrastructure product that every other sporeGarden product
(helixVision, esotericWebb, RPGPT) eventually runs on.

```
ecoPrimals (organisms) → syntheticChemistry (springs/validation) → sporeGarden (products)
                                    ↓                                       ↓
                            primalSpring                          projectNUCLEUS
                         (validates compositions)          (deploys compositions)
```

It takes primal binaries from plasmidBin, composition graphs from
primalSpring, and standards from wateringHole, and assembles them into a
running gate on your hardware.

## NUCLEUS Atomics

NUCLEUS composes from three atomics, each named for a subatomic particle:

| Atomic | Particle | Primals | Role |
|--------|----------|---------|------|
| **Tower** | Electron | BearDog + Songbird | Trust boundary — crypto, identity, networking, BTSP |
| **Node** | Proton | Tower + ToadStool + barraCuda + coralReef | Compute — workload dispatch, GPU math, shader compilation |
| **Nest** | Neutron | Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass | Storage — content-addressed data, provenance, attribution |

Full NUCLEUS = Tower + Node + Nest + Squirrel (AI) + biomeOS (orchestration).

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

**Phase 1 validated (2026-05-04, elevated 2026-05-04)**:
- Nest Atomic + ToadStool (9 primals) running: BearDog, SongBird, ToadStool, BarraCuda, CoralReef, NestGate, rhizoCrypt, loamSpine, sweetGrass
- 235+ wetSpring science checks passing across 10 workloads (8 Rust validators, 2 Python baselines)
- Real NCBI data processed: 11.9M paired-end reads from PRJNA488170
- Full provenance chain operational: BLAKE3 content hashes → rhizoCrypt DAG (24 events) → loamSpine permanent ledger → sweetGrass ed25519-witnessed braid
- Python + Rust workloads dispatched via TOML workload specs through toadStool
- JupyterHub available for notebook-style access

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

### Phase 1: Covalent LAN HPC (current)

Single-gate Nest Atomic + ToadStool (9 primals) with provenance pipeline.
Manual bootstrap, covalent bonding only. Workloads run locally with full
provenance: BLAKE3 content hashing, DAG sessions, permanent ledger, attribution braids.

### Phase 2: Ionic Compute Sharing

Tower Atomic on NUC as intake node. songBird cross-gate routing. BTSP Phase 2
authentication for external users. The "lend a GPU to a friend" pattern.

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
gates/              Gate inventory and hardware configs
deploy/             Deployment tooling (deploy.sh, provenance_pipeline.sh, gate manifest)
graphs/             Curated deploy graph TOMLs (from primalSpring)
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads
  templates/        Templates for new workloads
validation/         Composition validation results, provenance manifests, gap reports
docs/               Architecture primers and external-facing docs
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **plasmidBin** | ecoPrimals/infra | Binary depot — projectNUCLEUS fetches primals from here |
| **primalSpring** | syntheticChemistry | Composition validation — projectNUCLEUS references deploy graphs |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — projectNUCLEUS follows these |
| **sporePrint** | ecoPrimals/infra | The website — projectNUCLEUS eventually hosts it |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
