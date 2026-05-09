# projectNUCLEUS

The deployable NUCLEUS infrastructure product. Stand up a sovereign compute
node, host sites, dispatch workloads, and progressively eliminate every
external dependency.

**Organization**: sporeGarden (products built on ecoPrimals)
**Generation**: gen4 — composition and deployment
**License**: AGPL-3.0-or-later (code), ORC (system mechanics), CC-BY-SA 4.0 (docs)

## What This Is

projectNUCLEUS is the deployable NUCLEUS infrastructure — the compute layer
that takes primal binaries from plasmidBin, composition graphs from
primalSpring, and standards from wateringHole, and assembles them into a
running gate on real hardware.

```
primalSpring (upstream patterns)
       ↓ deploy graphs, validation, standards
projectNUCLEUS on active gate (deploys + validates patterns)
       ↓ real workloads, real users
ABG collaborators (ionic compute sharing = pattern validation under load)
       ↓ geological record
foundation (the soil: validated lineage, gap handbacks, bonding models)
```

**The core loop**: primalSpring defines composition patterns upstream.
projectNUCLEUS deploys those patterns on the active gate. ABG workloads
validate them under real external load. Gaps flow back upstream via handoff
docs. Validated patterns and geological records settle into foundation.
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

**Phase 60 absorbed, MethodGate enforced, open observer live (2026-05-09)**

### Infrastructure

- All **13/13 NUCLEUS primals** deployed and healthy on the active gate
- BTSP Phase 3 AEAD (ChaCha20-Poly1305), Wire Standard L3, discovery escalation — all converged
- **5-tier discovery hierarchy**: Songbird IPC → biomeOS Neural → UDS convention → socket registry → TCP probing
- 235+ wetSpring science checks passing across 11 workloads (8 Rust PASS, 2 Python RUN, 1 deferred FAIL)
- Full provenance chain operational: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid

### Services (all persistent via systemd)

| Service | URL | Port | Status |
|---------|-----|------|--------|
| Voila (observer) | `lab.primals.eco` | 8866 | Live, open/unauthenticated, public science surface |
| JupyterHub | `lab.primals.eco` (gated) | 8000 | Live, PAM auth + Cloudflare Access, reviewer/user tiers |
| Forgejo | `git.primals.eco` | 3000 | Live, projectNUCLEUS mirrored |
| pappusCast | — | — | Tiered auto-propagation daemon (workspace → observer) |
| Cloudflare Tunnel | — | outbound | Routes lab + git subdomains; Access gates reviewer/user |
| 13 NUCLEUS primals | localhost | 9100–9900 | All healthy, user services |

### Access Model

Three-tier model simplified from four. Observer is the default, open landing page.
Reviewer and user tiers are gated by Cloudflare Access + PAM.

| Tier | Access | Capabilities | Surface |
|------|--------|-------------|---------|
| **Observer** | Open — no login | Read-only rendered notebooks, data, dashboards | Voila at `lab.primals.eco` |
| **Reviewer** | Cloudflare Access + PAM | Read + run Voila contracts | JupyterHub (showcase-only view) |
| **User** | Cloudflare Access + PAM | Read + write + run, shared workspace | JupyterHub (full workspace) |

### Auto-Propagation (pappusCast)

`pappusCast` daemon auto-propagates validated content from the shared workspace
to the public observer surface on an adaptive schedule:

- **Light** (on-change): JSON valid, kernel available, title present
- **Medium** (periodic): Light + execute as voila user, check for cell errors
- **Heavy** (~6 hours): Medium + diff, changelog, full regression
- **Adaptive rate limiting**: publish interval scales with active JupyterHub users
- **Snapshot architecture**: public/ holds managed copies, not live symlinks
- **Evolution path**: Python (now) → Rust binary → pappusCast primal
- **Static HTML export**: Heavy tier renders all public notebooks to `.pappusCast/html_export/` as always-on fallback

### Gate Portability

The compute surface is gate-portable — it can migrate between physical gates
without downtime for the static observer layer. See `specs/GATE_PORTABILITY.md`.

- **Always-on**: Static HTML exports + sporePrint (GitHub Pages) persist regardless of gate
- **Gate-portable**: `deploy/gate_switch.sh <target>` transfers compute services to any gate
- **Bonding isolation**: Observer surface is gate-anonymous (no gate names, no internal topology)

### Security

- **UFW active**: deny-by-default, allow SSH/LAN/localhost
- **hidepid=2**: process isolation — ABG users cannot see primal PIDs or other users' processes
- **Outbound network blocked**: iptables/ip6tables owner match DROPs all internet for ABG UIDs (localhost + LAN preserved)
- **Observer surface hardened**: source stripped, internal directories blocked, page titles on all notebooks, admin templates disabled, root redirects to Welcome.ipynb
- **Reviewer/user lockdown**: NoKernelManager blocks kernel creation for reviewers, no terminals, filesystem read-only (chmod 550 root-owned)
- **Shared notebooks immutable**: compute users can run but not save back (chmod 444, per-user results dirs)
- JupyterHub security headers (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Server suppressed)
- **MethodGate (JH-0) ENFORCED**: 13/13 primals ship MethodGate. 10/13 confirmed enforced via TCP. All unauthenticated calls return `-32001 PERMISSION_DENIED`
- **Ionic tokens (JH-1) LIVE**: BearDog Ed25519-signed scoped tokens with expiry and JTI
- **Resource envelopes (JH-2)**: biomeOS v3.48 + ToadStool S232 enforce limits on all dispatch paths
- **Composition reload (JH-3)**: biomeOS `composition.reload` — hot-swap single primal without full restart
- **Session UX (JH-4)**: `auth.issue_session` — purpose-based presets
- **Audit log (JH-5)**: skunkBat ring buffer, 7 event kinds, cursor-based polling
- **All 14 primal ports bound `127.0.0.1`** (Phase 60 PG-55 default)
- **darkforest v0.2.0**: modular Rust security validator — **175 PASS, 0 FAIL, 6 DARK_FOREST** (`validation/darkforest/`)
- **tunnelKeeper**: Rust crate for Cloudflare tunnel health/management (`validation/tunnelKeeper/`)
- **Multi-tier test suite**: observer + reviewer + compute + hub + pappusCast health (`deploy/tier_test_all.sh`)
- **DNS exfil closed**, **supply chain locked**, **crontab restricted**, **version disclosure suppressed**

### Sovereignty Evolution

- **40+ dependencies mapped** across 7 clusters (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines capturing** hourly via cron (DNS, TCP, TLS, TTFB, total latency)
- **Cloudflare Access**: Reviewer/user tiers gated via Zero Trust policies (`deploy/cloudflare/access_setup.sh`)
- **tunnelKeeper** (`validation/tunnelKeeper/`): Rust crate for programmatic tunnel health checks, DNS resolution, config validation — first step toward Rust-native Cloudflare interaction
- **benchScale framework** operational — 5 scenarios, 3 pentest scripts
- **Forgejo calibration instrument** installed — baseline for RootPulse parity targets
- **6 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub patterns (JH-0→JH-11), primal deep debt, consolidated upstream gaps

### sporePrint

- Public observer surface live at `lab.primals.eco` (Voila — open, no login)
- 15+ notebooks across commons/, showcase/, data/, pilot/, validation/
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

13 primals on the active gate (Full NUCLEUS) with provenance pipeline.
235+ wetSpring science checks passing. Full provenance chain operational.
This proves the substrate works on our hardware.

### Phase 2: Ionic Compute Sharing (in progress — Step 2a/2b operational)

Deploy a usable system for ABG as validation of primalSpring patterns.
Step 2a: Cloudflare Tunnel baseline captured (270ms p50, 15/15 external checks).
Step 2b: Open observer landing (Voila, no credentials). Reviewer/user gated via
Cloudflare Access + PAM. pappusCast auto-propagation daemon live (tiered validation,
adaptive rate limiting, snapshot architecture). Multi-tier test suite validates all
access levels. tunnelKeeper Rust crate for Cloudflare interaction evolution.

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
deploy/             Deployment tooling, test suites, pappusCast daemon
  pappusCast.py     Tiered auto-propagation daemon (workspace → observer surface)
  tier_test_*.py    Multi-tier test suite (observer, reviewer, compute)
  tier_test_all.sh  Unified test runner across all tiers + pappusCast health
  cloudflare/       Cloudflare Access setup and tunnel configuration
graphs/             Deploy graph TOMLs — curated from primalSpring + RootPulse workflows
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads (8 Rust + 2 Python + 1 deferred)
  templates/        Templates for new workloads
validation/         Composition validation, security pen tests, upstream gap handbacks
  darkforest/       Pure Rust security validator (v0.2.0 — pen test + fuzz + crypto)
  tunnelKeeper/     Rust crate for Cloudflare tunnel health/management
  baselines/        Hourly Cloudflare tunnel metrics (cron-captured CSVs)
  archive/          Timestamped provenance runs, prior security scans, legacy scripts
infra/              Infrastructure tooling
  benchScale/       Load generation and pen testing framework for sovereignty validation
docs/               Architecture primers and external-facing docs
```

ABG shared workspace (`$ABG_SHARED`):

```
commons/            Group scratch — quick experiments, onboarding notebooks
pilot/              Structured experiments (hypothesis, decision criteria, timeline)
projects/           Formal project spaces (notebooks, data, results)
data/               Shared datasets (NCBI, reference genomes, calibration)
templates/          Starter notebooks, workload TOMLs, welcome notebooks
showcase/           Polished work + Voila dashboards
validation/         Surfaced darkforest JSON reports
public/             Managed snapshot copies for observer surface (pappusCast-managed)
  .pappusCast/      Daemon state, changelog, quarantine
```

## Relationship to Other Repos

| Repo | Org | Relationship |
|------|-----|-------------|
| **primalSpring** | syntheticChemistry | Upstream — defines composition patterns that projectNUCLEUS deploys and validates |
| **plasmidBin** | ecoPrimals/infra | Binary depot — projectNUCLEUS fetches primal binaries from here |
| **wateringHole** | ecoPrimals/infra | Standards and guidance — projectNUCLEUS follows these |
| **sporePrint** | ecoPrimals/infra | The website ([primals.eco](https://primals.eco)) — live with notebooks, spring science hubs, auto-refresh CI; Phase 3 target: self-hosted on NUCLEUS |
| **foundation** | sporeGarden | The soil — validated scientific lineage, gap handbacks, bonding models, domain threads |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
