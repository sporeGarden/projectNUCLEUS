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

**Sovereignty evolution ACTIVE — Forgejo primary (32 repos), VPS Tower LIVE (2GB, 6 services), Channel 3 shadow, content-aware routing (2026-05-15)**

### Infrastructure

- All **13/13 NUCLEUS primals** deployed and healthy — **zero debt** (L1 clean, MethodGate enforced)
- **8/8 springs** at Tier 4 IPC-first — 13,750+ tests, LTEE reproductions active
- **Zero open upstream gaps** — NestGate Session 60, all per-primal debt closed
- BTSP Phase 3 AEAD, Wire Standard L3, 5-tier discovery hierarchy — all converged
- Full provenance chain: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid
- **Cell membrane architecture**: primals.eco on CDN (extracellular), lab/git.primals.eco via tunnel (membrane), cellMembrane fieldMouse on DigitalOcean VPS (external membrane), sovereign compute inside
- **NestGate content pipeline SHIPPED** (Session 60): 8 `content.*` methods on 4 transports. H2-05 **DONE**, H2-06–09 **UNBLOCKED**
- **Static observer surface**: pre-rendered HTML via pappusCast, centralized dark theme, Rust-validated (darkforest `--suite observer`)
- **`composition.deploy(graph)` WIRED**: `deploy_graph.sh` reads graph TOML, starts primals in dependency order
- **cellMembrane LIVE — Tower composition (H2-14)**: fieldMouse deployment on 157.230.3.183 (DigitalOcean nyc1, **$12/mo 2GB RAM**). **6 services active**: Songbird TURN :3478 (Ch2), RustDesk hbbs/hbbr :21115-17 (Ch2b), BearDog crypto :9100 (Tower), SkunkBat audit :9140 (Tower), Caddy TLS :80 shadow (Ch3). Channel 3 health endpoint LIVE. 1.7GB RAM free. Hardened (fail2ban, UFW, tmpfiles.d persistence). DO token encrypted with BearDog AES-256-GCM. Owned by ironGate/projectNUCLEUS. Private ops repo: `sporeGarden/cellMembrane`
- **BearDog TLS shadow LIVE (H2-12)**: BearDog v0.9.0 on :8443 alongside Cloudflare :443 — 2ms RPC latency vs 102ms Cloudflare baseline (51x). `btsp_tls_parity.sh` ready for 7-day comparison
- **BTSP dual-auth shadow ACTIVE**: BTSPAuthenticator plugin live on JupyterHub — PAM + ionic token dual-accept, auth events accumulating
- **Provenance pipeline validated**: Full 9-phase pipeline through trio (rhizoCrypt DAG + loamSpine spine + sweetGrass braid). 6/12 wetspring workloads PASS with BLAKE3-anchored provenance chain. Merkle root + ed25519 witness braid operational
- **DoT baseline CAPTURED**: systemd-resolved DoT ACTIVE via Cloudflare 1.0.0.1, 3-8ms latency, 10/10 success. Sovereign resolver (knot-dns) pending
- **Tunnel baseline CAPTURED**: 9-day quantile summary generated (`validation/baselines/cloudflare_tunnel_7day.toml`)
- **Shadow run orchestrator**: `infra/benchScale/scenarios/shadow_run_orchestrator.sh` ties all 4 parity tests (NestGate, BearDog TLS, Songbird NAT, DoT)
- **NAT shadow run STARTED**: cellMembrane TURN relay 100% reachable (10/10 probes). `songbird_nat_parity.sh` ready for full HTTP parity
- **7-day Cloudflare baseline CAPTURED**: 9 days, 950 samples — TLS p50=73ms p95=101ms, TTFB p50=119ms p95=190ms. BearDog shadow 51x faster at p50
- **darkforest --suite membrane**: 17 PASS, 0 FAIL against live cellMembrane VPS (MEM-01 through MEM-13). Password auth disabled, fail2ban active, credentials 600/root, no unexpected listeners
- **Dark Forest Glacial Gate PASS**: `validation/dark_forest_gate_local.sh` — 33 structural checks across 5 pillars. All deploy graphs carry `secure_by_default = true`
- **Deep debt evolution COMPLETE**: deploy.sh modularized, darkforest pentest/crypto split into submodules, tunnelKeeper clone optimization, all workload TOMLs gate-agnostic (`$SPRINGS_ROOT`), deploy scripts use `$ECOPRIMALS_ROOT`
- **lithoSpore 6/7 modules PASS Tier 2** (51/51 checks): Rust validation for fitness, mutations, alleles, citrate, breseq, anderson. Module 5 (biobricks) awaits upstream B6 data
- **Provenance trio graph capabilities reconciled**: GAP-36 canonical names (`dag.*`, `spine.*`, `braid.*`) aligned across `nucleus_complete.toml`, `rootpulse_commit.toml`, and `provenance_pipeline.sh`
- **BTSP dual-auth plugin BUILT** (H2-01): `deploy/jupyterhub_btsp_auth.py` — BTSPAuthenticator with PAM fallback, auth logging, pre_spawn_hook. `deploy/deploy_btsp_auth_shadow.sh` for shadow run management
- **`biomeos.spring_status` IMPLEMENTED** (v3.54): Binary discovery + workload counts. Registry at **427 methods**
- **API methods RESOLVED**: `nestgate.artifact_query`, `rhizocrypt.dag_summary` covered by existing shipped methods

### Services (all persistent via systemd)

| Service | URL | Port | Layer | Status |
|---------|-----|------|-------|--------|
| primals.eco | `primals.eco` | — | Extracellular | GitHub Pages + Cloudflare CDN (always on, no gate) |
| Observer (static) | `lab.primals.eco` | 8866 | Membrane | Pre-rendered HTML, open/unauthenticated |
| JupyterHub | `lab.primals.eco` (gated) | 8000 | Membrane | PAM auth + Cloudflare Access, reviewer/user tiers |
| Forgejo | `git.primals.eco` | 3000 | Intracellular | **Primary git host** — 32 repos, 3 orgs. GitHub = push mirror |
| pappusCast | — | — | Intracellular | Tiered auto-propagation daemon (workspace → observer) |
| Cloudflare Tunnel | — | outbound | Membrane | Routes lab + git subdomains (membrane channels) |
| cellMembrane | 157.230.3.183 | 3478, 21115-17, 80 | Inner Membrane | **Tower composition** (2GB): Songbird TURN (Ch2) + RustDesk (Ch2b) + BearDog crypto + SkunkBat audit + Caddy TLS shadow (Ch3). DigitalOcean nyc1 |
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
- **External Membrane**: cellMembrane fieldMouse on DigitalOcean VPS — Channel 2 (TURN relay) for NAT traversal across gate boundaries
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
- **MethodGate (JH-0) ENFORCED**: 13/13 primals ship MethodGate. All confirmed enforced via TCP. Unauthenticated calls return `-32001 PERMISSION_DENIED`
- **Ionic tokens (JH-1) LIVE**: BearDog Ed25519-signed scoped tokens with expiry and JTI
- **Resource envelopes (JH-2)**: biomeOS v3.48 + ToadStool S232 enforce limits on all dispatch paths
- **Composition reload (JH-3)**: biomeOS `composition.reload` — hot-swap single primal without full restart
- **Session UX (JH-4)**: `auth.issue_session` — purpose-based presets
- **Audit log (JH-5)**: skunkBat ring buffer, 7 event kinds, cursor-based polling
- **All primal ports bound `127.0.0.1`** — 13 primals + auxiliary ports (Phase 60 PG-55 default)
- **darkforest v0.2.1**: modular Rust security + observer validator — 8 source modules including `observer.rs` (static HTML quality: theme, nav, links, tracebacks, source stripping, headers, directory blocking). Env-var-driven config with compiled fallback. `--suite observer` for static surface validation (86 PASS, 0 FAIL)
- **tunnelKeeper v0.2.0**: Rust crate for tunnel health/management (`validation/tunnelKeeper/`). Replica count, unique origins, edge colo detection. Error-propagating API client, gate-agnostic credential paths
- **Multi-tier test suite**: observer + reviewer + compute + hub + pappusCast health (`deploy/tier_test_all.sh`)
- **DNS exfil closed**, **supply chain locked**, **crontab restricted**, **version disclosure suppressed**

### Sovereignty Evolution

- **Three-membrane architecture**: outer (GitHub mirror) / inner (VPS touchpoint) / intracellular (gate source of truth)
- **VPS as touchpoint, gate as source**: VPS terminates TLS, relays traffic, caches content. Gate hardware runs full NUCLEUS
- **Forgejo PRIMARY**: 32 repos mirrored across 3 orgs. `forgejo_mirror.sh` dual-push. GitHub is outer membrane
- **Content-aware routing**: `routing_config.toml` — static→VPS cache, auth/API/git→gate, large→Songbird P2P
- **Trust model**: covalent/ionic/metallic/weak bonding maps to content access scopes in routing
- **VPS Tower LIVE**: BearDog crypto + SkunkBat audit + Songbird relay + RustDesk + Caddy TLS shadow
- **Channel 3 shadow**: HTTP health on :80, TLS blocks ready for DNS grey-cloud
- **DO credentials encrypted**: BearDog AES-256-GCM with Argon2id on VPS
- **40+ dependencies mapped** across 7 clusters (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines captured**: 9-day summary (950 samples) — TTFB p50=119ms p95=190ms, TLS p50=73ms p95=101ms
- **benchScale framework** operational — 5 scenarios, 3 pentest scripts
- **6 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub (JH-0→JH-11), primal deep debt

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
  nucleus_config.sh Gate-agnostic config (all paths, ports, env vars, routing, membrane — single source of truth)
  forgejo_mirror.sh Forgejo org/repo creation + dual-push for all repos
  vps_resize.sh     doctl VPS resize automation
  routing_config.toml Content-aware routing rules (trust model, backends, cache policy)
  nucleus_paths.py  Python config module (imports GATE_HOME, ABG_SHARED, etc. from env)
  observer_server.py Static HTTP server for pre-rendered observer HTML (port 8866)
  pappusCast.py     Tiered auto-propagation daemon (workspace → observer surface)
  gate_provision.sh Provision new membrane replicas (tunnel-only gates)
  gate_watchdog.sh  Membrane health monitor (lab/git endpoints, logs for skunkBat)
  gate_switch.sh    Migrate compute services between gates
  tier_test_all.sh  Unified test runner across all tiers + pappusCast health
  cloudflare/       Cloudflare Access setup and tunnel configuration
  cloudflared/      Tunnel config templates (config-full.yml, config-static.yml)
  nucleus_config.sh includes cellMembrane VPS config (MEMBRANE_VPS_IP, TURN credentials)
graphs/             Deploy graph TOMLs — curated from primalSpring + RootPulse workflows
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads (8 Rust + 2 Python + 1 deferred)
  templates/        Templates for new workloads
validation/         Composition validation, security pen tests, upstream gap handbacks
  dark_forest_gate_local.sh  Dark Forest Glacial Gate 5-pillar structural validation (33 checks)
  darkforest_membrane.sh     cellMembrane VPS remote audit (MEM-01 through MEM-13)
  darkforest/       Pure Rust security validator (v0.2.1 — pen test + fuzz + crypto, modular submodules)
  tunnelKeeper/     Rust crate for Cloudflare tunnel health/management
  baselines/        Hourly Cloudflare tunnel metrics (cron-captured CSVs + 7-day summary TOML)
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
| **cellMembrane** | sporeGarden | **Private** ops repo — VPS state, runbooks, credential procedures for the cellMembrane fieldMouse deployment |
| **foundation** | sporeGarden | The soil — validated scientific lineage, gap handbacks, bonding models, domain threads |
| **helixVision** | sporeGarden | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | sporeGarden | Creative product — runs on projectNUCLEUS |
