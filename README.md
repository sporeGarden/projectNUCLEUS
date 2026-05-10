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

**Post-interstadial, cell membrane architecture, zero open upstream gaps (2026-05-10)**

### Infrastructure

- All **13/13 NUCLEUS primals** deployed and healthy on the active gate — zero debt across all primals
- **8/8 springs** completed primordial extinction — eukaryotic UniBin, 9,317+ tests across the delta
- **Zero open upstream gaps** — all 11 gaps resolved by primal teams (JH-11, GAP-03/06/09/12, U1-U3, DF-2/3, U5)
- BTSP Phase 3 AEAD (ChaCha20-Poly1305), Wire Standard L3, discovery escalation — all converged
- **5-tier discovery hierarchy**: Songbird IPC → biomeOS Neural → UDS convention → socket registry → TCP probing
- Full provenance chain operational: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid
- **Cell membrane architecture**: primals.eco on GitHub Pages CDN (extracellular), lab/git.primals.eco via tunnel (membrane), sovereign compute inside
- **Static observer surface**: pre-rendered HTML via pappusCast, centralized dark theme, Rust-validated (darkforest `--suite observer`)
- **Deep debt swept**: gate-agnostic config (`$GATE_HOME`), env-var-driven port/path resolution, zero TODO/FIXME/HACK
- **Sovereignty unblocked upstream**: bearDog TLS + rate limiting (H2-10/11), songbird full NAT chain (H2-13-16), JH-11 token federation, skunkBat Phase 2 audit logging — all shipped

### Services (all persistent via systemd)

| Service | URL | Port | Layer | Status |
|---------|-----|------|-------|--------|
| primals.eco | `primals.eco` | — | Extracellular | GitHub Pages + Cloudflare CDN (always on, no gate) |
| Observer (static) | `lab.primals.eco` | 8866 | Membrane | Pre-rendered HTML, open/unauthenticated |
| JupyterHub | `lab.primals.eco` (gated) | 8000 | Membrane | PAM auth + Cloudflare Access, reviewer/user tiers |
| Forgejo | `git.primals.eco` | 3000 | Membrane | Sovereign git mirror |
| pappusCast | — | — | Intracellular | Tiered auto-propagation daemon (workspace → observer) |
| Cloudflare Tunnel | — | outbound | Membrane | Routes lab + git subdomains (membrane channels) |
| 13 NUCLEUS primals | localhost | 9100–9900 | Intracellular | All healthy, user services |

### Access Model

Three-tier model simplified from four. Observer is the default, open landing page.
Reviewer and user tiers are gated by Cloudflare Access + PAM.

| Tier | Access | Capabilities | Surface |
|------|--------|-------------|---------|
| **Observer** | Open — no login | Read-only rendered notebooks, data, dashboards | Static HTML at `lab.primals.eco` |
| **Reviewer** | Cloudflare Access + PAM | Read + run notebooks (showcase) | JupyterHub (showcase-only view) |
| **User** | Cloudflare Access + PAM | Read + write + run, shared workspace | JupyterHub (full workspace) |

### Auto-Propagation (pappusCast)

`pappusCast` daemon auto-propagates validated content from the shared workspace
to the public observer surface on an adaptive schedule:

- **Light** (on-change): JSON valid, kernel available, title present
- **Medium** (periodic): Light + execute as voila user, check for cell errors
- **Heavy** (~6 hours): Medium + diff, changelog, full regression
- **Adaptive rate limiting**: publish interval scales with active JupyterHub users
- **Snapshot architecture**: public/ holds managed copies, not live symlinks
- **Evolution path**: Python (now) → Rust binary → pappusCast primal. Static observer = primary surface since 2026-05-10
- **Static HTML observer**: Medium + Heavy tiers render all public notebooks to `.pappusCast/html_export/` — served directly as the observer surface (replaces dynamic Voila). `pappusCast.py export` for manual regeneration

### Gate Portability (Cell Membrane)

Infrastructure follows a cell membrane model. See `specs/GATE_PORTABILITY.md`.

- **Extracellular**: `primals.eco` on GitHub Pages CDN — always on, zero gate dependency
- **Membrane**: `lab/git.primals.eco` via Cloudflare tunnel with multi-gate replicas (sub-second failover)
- **Intracellular**: sovereign compute, primals, data — total control inside the gate
- **Gate-portable**: `deploy/gate_switch.sh <target>` migrates compute; replicas stay in membrane pool
- **Provisioning**: `deploy/gate_provision.sh <host>` adds a new membrane replica (friend's house, etc.)
- **Membrane watchdog**: `deploy/gate_watchdog.sh` logs membrane health for skunkBat audit

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
- **darkforest v0.2.1**: modular Rust security + observer validator — 8 source modules including `observer.rs` (static HTML quality: theme, nav, links, tracebacks, source stripping, headers, directory blocking). Env-var-driven config with compiled fallback. `--suite observer` for static surface validation (86 PASS, 0 FAIL)
- **tunnelKeeper v0.2.0**: Rust crate for tunnel health/management (`validation/tunnelKeeper/`). Replica count, unique origins, edge colo detection. Error-propagating API client, gate-agnostic credential paths
- **Multi-tier test suite**: observer + reviewer + compute + hub + pappusCast health (`deploy/tier_test_all.sh`)
- **DNS exfil closed**, **supply chain locked**, **crontab restricted**, **version disclosure suppressed**

### Sovereignty Evolution

- **Cell membrane architecture**: extracellular (CDN) / membrane (tunnel) / intracellular (sovereign) separation
- **40+ dependencies mapped** across 7 clusters (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines capturing** hourly via cron (DNS, TCP, TLS, TTFB, total latency)
- **Cloudflare Access**: Reviewer/user tiers gated via Zero Trust policies
- **tunnelKeeper v0.2.0**: Rust crate for tunnel health, replica count, membrane monitoring
- **benchScale framework** operational — 5 scenarios, 3 pentest scripts
- **Forgejo calibration instrument** installed — baseline for RootPulse parity targets
- **6 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub patterns (JH-0→JH-11), primal deep debt, consolidated upstream gaps

### sporePrint (Extracellular)

- `primals.eco` served permanently by GitHub Pages + Cloudflare CDN (extracellular surface)
- 15+ notebooks across commons/, showcase/, data/, pilot/, validation/
- Auto-refresh CI across 26 repos; `sporeprint/` directories in all 8 springs
- Local preview via `deploy/sporeprint_local.sh` (dev tool, not production path)

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
Step 2b: Open observer landing (static HTML, no credentials). Reviewer/user gated via
Cloudflare Access + PAM. Cell membrane architecture live: `primals.eco` on GitHub
Pages CDN (extracellular), `lab/git.primals.eco` via tunnel replicas (membrane),
sovereign compute inside (intracellular). pappusCast auto-propagation, multi-tier
test suite, tunnelKeeper v0.2.0 replica monitoring.

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
deploy/             Deployment tooling, test suites, pappusCast daemon, membrane infrastructure
  nucleus_config.sh Gate-agnostic config (all paths, ports, env vars — single source of truth)
  nucleus_paths.py  Python config module (imports GATE_HOME, ABG_SHARED, etc. from env)
  observer_server.py Static HTTP server for pre-rendered observer HTML (port 8866)
  pappusCast.py     Tiered auto-propagation daemon (workspace → observer surface)
  gate_provision.sh Provision new membrane replicas (tunnel-only gates)
  gate_watchdog.sh  Membrane health monitor (lab/git endpoints, logs for skunkBat)
  gate_switch.sh    Migrate compute services between gates
  tier_test_all.sh  Unified test runner across all tiers + pappusCast health
  cloudflare/       Cloudflare Access setup and tunnel configuration
  cloudflared/      Tunnel config templates (config-full.yml, config-static.yml)
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
| **sporePrint** | ecoPrimals/infra | The website ([primals.eco](https://primals.eco)) — extracellular layer on GitHub Pages CDN; Phase 3 target: petalTongue self-hosted rendering |
| **foundation** | sporeGarden | The soil — validated scientific lineage, gap handbacks, bonding models, domain threads |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
