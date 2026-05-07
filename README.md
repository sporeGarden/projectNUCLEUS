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

**Phase 59 absorbed (2026-05-06), sovereignty evolution in progress (2026-05-07)**

### Infrastructure

- All **13/13 NUCLEUS primals** deployed and healthy on ironGate
- BTSP Phase 3 AEAD (ChaCha20-Poly1305), Wire Standard L3, discovery escalation — all converged
- **5-tier discovery hierarchy**: Songbird IPC → biomeOS Neural → UDS convention → socket registry → TCP probing
- 235+ wetSpring science checks passing across 11 workloads (8 Rust PASS, 2 Python RUN, 1 deferred FAIL)
- Full provenance chain operational: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid

### Services (all persistent via systemd)

| Service | URL | Port | Status |
|---------|-----|------|--------|
| JupyterHub | `lab.primals.eco` | 8000 | Live, PAM auth, tiered ABG access |
| Forgejo | `git.primals.eco` | 3000 | Live, projectNUCLEUS mirrored |
| Cloudflare Tunnel | — | outbound | Routes lab + git subdomains |
| 13 NUCLEUS primals | localhost | 9100–9900 | All healthy, user services |

### Security

- **UFW active**: deny-by-default, allow SSH/LAN/localhost
- **hidepid=2**: process isolation — ABG users cannot see primal PIDs or other users' processes
- **Outbound network blocked**: iptables/ip6tables owner match DROPs all internet for ABG UIDs (localhost + LAN preserved)
- **Reviewer/observer lockdown**: no kernels, no terminals, filesystem read-only (chmod 550 root-owned)
- **Shared notebooks immutable**: compute users can run but not save back (chmod 444, per-user results dirs)
- JupyterHub security headers (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Server suppressed)
- All PG-55 through PG-59 resolved by primalSpring Phase 59
- 13/13 primals default `127.0.0.1`, NestGate BTSP method-level auth, skunkBat anomaly detection
- Two-round pen test complete: infrastructure layer + multi-user layer (`validation/SECURITY_HANDBACK_MAY06_2026.md`)
- skunkBat surveillance targets identified: JupyterHub auth events, NestGate writes, iptables DROPs, process enumeration

### Sovereignty Evolution

- **40+ dependencies mapped** across 7 clusters including internal primal gaps (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines capturing** hourly via cron (DNS, TCP, TLS, TTFB, total latency)
- **benchScale framework** operational (`infra/benchScale/`) — 5 scenarios, 3 pentest scripts
- **Forgejo calibration instrument** installed — baseline for RootPulse parity targets
- **RootPulse commit workflow tested** — 5/6 phases pass against live primals, Phase 5 (LoamSpine commit) has param mismatch
- **4 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub patterns (JH-0→JH-5)
- **JH-0 (Critical)**: RPC dispatchers accept unauthenticated calls from any localhost user — upstream gap for all primal teams
- **40+ dependencies mapped** across 7 clusters including internal primal gaps (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)

### sporePrint

- 5 public notebooks on [primals.eco/lab/notebooks](https://primals.eco/lab/notebooks/)
- Auto-refresh CI across 26 repos; `sporeprint/` directories in all 8 springs
- Live Science API spec: 6 JSON-RPC methods (`specs/LIVE_SCIENCE_API.md`)

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
specs/              Local specs: execution model, composition, security, tunnel evolution, dependency inventory
gates/              Gate inventory and hardware configs
deploy/             Deployment tooling (deploy.sh, provenance_pipeline.sh, security_validation.sh)
graphs/             Deploy graph TOMLs — curated from primalSpring + RootPulse workflows
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads (8 Rust + 2 Python + 1 deferred)
  templates/        Templates for new workloads
validation/         Composition validation, security pen tests, upstream gap handbacks
  baselines/        Hourly Cloudflare tunnel metrics (cron-captured CSVs)
  archive/          Timestamped provenance runs and prior security scans
infra/              Infrastructure tooling
  benchScale/       Load generation and pen testing framework for sovereignty validation
docs/               Architecture primers and external-facing docs
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **primalSpring** | syntheticChemistry | Upstream — defines composition patterns that projectNUCLEUS deploys and validates |
| **plasmidBin** | ecoPrimals/infra | Binary depot — projectNUCLEUS fetches primal binaries from here |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — projectNUCLEUS follows these |
| **sporePrint** | ecoPrimals/infra | The website ([primals.eco](https://primals.eco)) — live with notebooks, spring science hubs, auto-refresh CI; Phase 3 target: self-hosted on NUCLEUS |
| **foundation** | sporeGarden | External-facing — institutional relationships, metallic federation |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
