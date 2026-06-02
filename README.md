# projectNUCLEUS

The deployable NUCLEUS infrastructure product. Stand up a sovereign compute
node, host sites, dispatch workloads, and progressively eliminate every
external dependency.

**Organization**: gardens (products built on ecoPrimals)
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
| **Tower** | Electron | BearDog + Songbird + skunkBat | Trust boundary — crypto, identity, networking, defense |
| **Node** | Proton | Tower + ToadStool + barraCuda + coralReef | Compute — workload dispatch, GPU math, shader compilation |
| **Nest** | Neutron | Tower + NestGate + rhizoCrypt + loamSpine + sweetGrass | Storage — content-addressed data, provenance, attribution |

Full NUCLEUS = Tower + Node + Nest + Squirrel (AI) + biomeOS (orchestration) + petalTongue (viz).

| Composition | Particle | Primals | Role |
|-------------|----------|---------|------|
| **Agent** | Photon | Tower + biomeOS + Squirrel | Autonomous agent — AI planning via `signal_plan`, graph execution via `signal.dispatch` |

The Agent composition is the smallest unit that can reason + act + audit autonomously.
Squirrel decomposes intent into atomic signals, biomeOS dispatches them through tower
primals. `signal_executor.sh` bridges the two via JSON-RPC.

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

**Wave 69 (2026-06-02)** — Forgejo CI promoted to primary (Rust toolchain pinned, `rust-toolchain.toml`). grapheneGate deploy graph added (`portable_anchor` gate class). Gate manifest extended to all 8 gates. 7 deprecated bash deploy scripts archived to `deploy/legacy/`. Hardcoded `127.0.0.1` evolved to `${NUCLEUS_BIND_ADDRESS}`. Doc test counts synced to actual **234 Rust tests** (darkforest 140, tunnelKeeper 48, nucleus-deploy 46). Wave 68 deep debt: temporal.rs split, divergence policy explicit, 204 GB build debris cleaned. `unsafe_code = "forbid"` across all crates. NC-1 **CODE COMPLETE**.

### Infrastructure

- All **13/13 NUCLEUS primals** deployed and healthy — **zero debt** (L1 clean, MethodGate enforced)
- **8/8 springs** at Tier 4 IPC-first — 13,750+ tests, LTEE reproductions active
- **Zero open upstream gaps** — NestGate Session 60, all per-primal debt closed
- BTSP Phase 3 AEAD, Wire Standard L3, 5-tier discovery hierarchy — all converged
- Full provenance chain: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid
- **Cell membrane architecture**: primals.eco on CDN (extracellular), lab/git.primals.eco via tunnel (membrane), cellMembrane fieldMouse on DigitalOcean VPS (external membrane), sovereign compute inside
- **NestGate content pipeline SHIPPED** (Session 60): 8 `content.*` methods on 4 transports. H2-05 **DONE**, H2-06–09 **UNBLOCKED**
- **Static observer surface**: pre-rendered HTML via pappusCast, centralized dark theme, Rust-validated (darkforest `--suite observer`)
- **`composition.deploy(graph)` WIRED**: `deploy_graph.sh` reads graph TOML, starts primals in dependency order. **Wave 56 `--uds-only`**: deploy scripts suppress all TCP port arguments for VPS standard (zero-port mode)
- **Agent composition WIRED**: `tower_agent.toml` graph + `signal_executor.sh` bridge — Squirrel `signal_plan` → biomeOS `signal.dispatch` agent loop. 5 compositions: tower, agent, node, nest, full
- **cellMembrane LIVE — Nest Atomic composition**: fieldMouse deployment on 157.230.3.183 (DigitalOcean nyc1, **$12/mo 2GB RAM**). **11 services, 7 primals**: Tower (BearDog :9100, SkunkBat :9140, Songbird :3478) + Nest (NestGate :9500, rhizoCrypt :9602, loamSpine :9700, sweetGrass :9850) + RustDesk :21115-17 + Caddy TLS :80/:443 + petalTongue :8080 + BearDog TLS shadow :8443. **Channel 3 TLS LIVE**: `membrane.primals.eco` ACME cert. 1.6GB RAM free. Hardened (fail2ban, UFW, tmpfiles.d). DO token encrypted (BearDog AES-256-GCM). Private ops repo: `gardens/cellMembrane`
- **BearDog TLS shadow LIVE (H2-12)**: BearDog v0.9.0 on :8443 alongside Cloudflare :443 — **3ms RPC latency** vs 102ms Cloudflare baseline (34x). Telemetry probe fixed: `/dev/tcp` + `read -t 1` replaces `nc` (which inflated to 3s). `btsp_tls_parity.sh` ready for 7-day comparison
- **BTSP dual-auth shadow ACTIVE**: BTSPAuthenticator plugin live on JupyterHub — PAM + ionic token dual-accept, auth events accumulating
- **Provenance pipeline validated**: Full 9-phase pipeline through trio (rhizoCrypt DAG + loamSpine spine + sweetGrass braid). 6/12 wetspring workloads PASS with BLAKE3-anchored provenance chain. Merkle root + ed25519 witness braid operational
- **Sovereign DNS LIVE (H2-17)**: knot-dns v3.2.6 authoritative on VPS, DNSSEC ECDSAP256SHA256. DoT baseline via Cloudflare 1.0.0.1. NS cutover (H2-18) pending registrar action
- **Tunnel baseline CAPTURED**: 9-day quantile summary at `validation/baselines/cloudflare_tunnel_7day.toml` (subsumed by unified `membrane_7day.toml`)
- **Shadow run orchestrator**: `infra/benchScale/scenarios/shadow_run_orchestrator.sh` ties all 5 shadow tracks + DNS (NestGate content, BearDog TLS, Songbird NAT, Auth, DoT). Reads unified `membrane_7day.toml` baselines
- **Continuous membrane telemetry**: `deploy/membrane_telemetry.sh` probes both membranes (VPS + gate) every 15 min via cron. `deploy/membrane_summary.sh` produces rolling 7-day `validation/baselines/membrane_7day.toml` with parity checks and cutover gates. Shadow data is **permanent** — collection continues beyond cutover
- **NAT shadow run + HTTP parity PASS**: cellMembrane TURN relay 100% reachable (10/10 probes). HTTP parity: VPS 68ms TTFB vs GitHub Pages 89ms (**PASS**, 10 samples). TLS parity via `membrane.primals.eco`: 130ms vs 96ms, 100% uptime both channels
- **7-day Cloudflare baseline CAPTURED**: 9 days, 950 samples — TLS p50=73ms p95=101ms, TTFB p50=119ms p95=190ms. BearDog shadow 51x faster at p50
- **darkforest --suite membrane**: 17 PASS, 0 FAIL against live cellMembrane VPS (MEM-01 through MEM-13). Password auth disabled, fail2ban active, credentials 600/root, no unexpected listeners
- **Dark Forest Glacial Gate PASS**: `validation/dark_forest_gate_local.sh` — 33 structural checks across 5 pillars. All deploy graphs carry `secure_by_default = true`
- **Deep debt evolution COMPLETE**: deploy.sh modularized, darkforest pentest/crypto split into submodules, tunnelKeeper clone optimization, all workload TOMLs gate-agnostic (`$SPRINGS_ROOT`), deploy scripts use `$ECOPRIMALS_ROOT`
- **lithoSpore 7/7 modules PASS Tier 2** (75/75 checks, 117 tests): Rust validation for fitness, mutations, alleles, citrate, biobricks, breseq, anderson. Cross-tier parity 7/7 MATCH (Python ↔ Rust). Tier 3 wired (trio JSON-RPC, graceful degradation)
- **Provenance trio graph capabilities reconciled**: GAP-36 canonical names (`dag.*`, `spine.*`, `braid.*`) aligned across `nucleus_complete.toml`, `rootpulse_commit.toml`, and `provenance_pipeline.sh`
- **BTSP dual-auth plugin BUILT** (H2-01): `deploy/jupyterhub_btsp_auth.py` — BTSPAuthenticator with PAM fallback, auth logging, pre_spawn_hook. `deploy/deploy_btsp_auth_shadow.sh` for shadow run management
- **`biomeos.spring_status` IMPLEMENTED** (v3.84): Binary discovery + workload counts. Registry at **460 methods** (Wave 56 — `nucleus.ingest_spore` + `nucleus.emit_spore` added; typed errors, env_keys centralized; cell graph `vps_standard` tagging; 12 primordial scripts archived to fossilRecord). NC-1 **CODE COMPLETE** — `biomeos-pseudospore` + emit materialization shipped. Live column U gated on VPS deploy
- **API methods RESOLVED**: `nestgate.artifact_query`, `rhizocrypt.dag_summary` covered by existing shipped methods
- **Wave 56 deployment standard**: `deploy.sh --uds-only` suppresses TCP ports across all 13 primals (VPS standard). `deploy_graph.sh` + `deploy_primal_start.sh` + `deploy_health_check.sh` all UDS-aware. Socket-based health checks in UDS-only mode. `primalspring checksums` + `primalspring registry` replace shell validation scripts
- **Wave 64 Rust evolution**: All deploy scripts evolved to idiomatic Rust — `nucleus-deploy` binary with 9 subcommands: `security`, `provenance`, `deploy`, `spore`, `telemetry`, `summary`, `verify`, `provision`, `dns`. `clap` CLI, `tokio` async, zero `unwrap()` in production, `unsafe_code = "forbid"` in all Cargo.toml. Security module split into `security/` directory (6 submodules). Shared `util.rs` deduplicates `blake3_hash`, `value_to_hex`, `hex_to_bytes`, timestamped logging. VPS IP/user centralized in `NucleusConfig` (was hardcoded across 3 modules). Bash originals deprecated in place. Cloudflare artifacts fossilized to `deploy/legacy/`. `SongbirdTransport` v0.2 in tunnelKeeper (sovereign TCP probe alongside cloudflared). **234 Rust tests PASS** (darkforest 140, tunnelKeeper 48, nucleus-deploy 46)
- **Wave 58 deep debt**: Blocking I/O evolved to `tokio::task::spawn_blocking` (health.rs). Discovery transport evolved from HTTP POST to newline-delimited JSON-RPC (matching primal wire format). Silent JSON serialization bugs fixed (`unwrap_or_default` → error propagation). `chrono` dependency removed (pure `std::time`). Deploy script hardcoded ports wired to `nucleus_config.sh` variables. Transport module ungated (+9 tests), pentest coverage expanded (+12 tests), health.rs async load fix. 14 deploy scripts evolved from hardcoded IPs/hostnames to `nucleus_config.sh` variables
- **Wave 55 deep debt**: `yaml_serde` (libyaml C) → `serde-saphyr` (pure Rust, panic-free). `net.rs` refactored with shared HTTP helpers. Zero clippy pedantic+nursery warnings. `deny.toml` bans `unsafe-libyaml`. darkforest zero C deps
- **Wave 46→56 upstream zero gate debt**: primalSpring v0.9.30 — 56 scenarios, 44-cell deployment matrix, 813 tests. All 14 primals pulled to latest. Deploy graphs 12/12 `secure_by_default`. `deny.toml` enforced. FlockGate gap tracked (H3-11). `--uds-only` VPS standard shipped Wave 56
- **Wave 38 sovereignty shadow FULL PASS**: `graphs/sovereignty_shadow.toml` — 5-track parity proof + DNS. Orchestrator: **6 PASS, 0 FAIL, 0 SKIP**. S1 TLS **LIVE** (13ms vs 163ms CF), S2 NAT **LIVE** (100% reachable), S3 content **LIVE** (TTFB 68ms vs 111ms GH), S4 auth **SHADOW LIVE** (cutover pending), S5 DNS **DEPLOYED** (NS cutover pending) (knot-dns DNSSEC). **Nest Atomic** deployed: NestGate v2.1.0, rhizoCrypt v0.14.0, loamSpine v0.9.16, sweetGrass v0.7.34 — provenance trio 10/10 PASS. 11 services, 7 primals on VPS

### Services (all persistent via systemd)

| Service | URL | Port | Layer | Status |
|---------|-----|------|-------|--------|
| primals.eco | `primals.eco` | — | Extracellular | GitHub Pages + Cloudflare CDN (always on, no gate) |
| Observer (static) | `lab.primals.eco` | 8866 | Membrane | Pre-rendered HTML, open/unauthenticated |
| JupyterHub | `lab.primals.eco` (gated) | 8000 | Membrane | PAM auth + Cloudflare Access, reviewer/user tiers |
| Forgejo | `git.primals.eco` | 3000 | Intracellular | **Primary git host** — 39 repos, 3 orgs. K-Derm diderm relay → GitHub |
| pappusCast | — | — | Intracellular | Tiered auto-propagation daemon (workspace → observer) |
| K-Derm Relay | — | outbound | Membrane | Diderm relay: gate → golgiBody-inner → peptidoglycan → golgiBody-ext → GitHub |
| cellMembrane VPS | `MEMBRANE_VPS_IP` | 3478, 9100–9850, 80, 443 | Inner Membrane | **Nest Atomic** (2GB): Tower + Nest (7 primals) + RustDesk + Caddy TLS + petalTongue. `membrane.primals.eco` ACME cert |
| 13 NUCLEUS primals | localhost | 9100–9900 | Intracellular | All healthy, user services |

### Access Model

Three-tier model. Observer is the default, open landing page.
Reviewer and user tiers are gated by BTSP dual-auth (PAM + ionic token).

| Tier | Access | Capabilities | Surface |
|------|--------|-------------|---------|
| **Observer** | Open — no login | Read-only rendered notebooks, data, dashboards | Static HTML at `lab.primals.eco` |
| **Reviewer** | BTSP + PAM | Read + run notebooks (showcase) | JupyterHub (showcase-only view) |
| **User** | BTSP + PAM | Read + write + run, shared workspace | JupyterHub (full workspace) |

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
- **Provisioning**: `nucleus-deploy provision --target <host>` provisions sovereign gates via SSH
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
- **darkforest v0.2.1**: modular Rust security + observer validator — 8 source modules including `observer.rs` (static HTML quality: theme, nav, links, tracebacks, source stripping, headers, directory blocking). Env-var-driven config with compiled fallback. `--suite observer` for static surface validation (86 PASS, 0 FAIL). **140 unit tests** covering check, crypto, discovery, fuzz, net, observer, pentest, report modules. Newline-delimited JSON-RPC discovery (primal wire-native). 1.1 MB release binary, zero C deps
- **tunnelKeeper v0.2.0**: Rust crate for tunnel health/management (`validation/tunnelKeeper/`). Dual-transport: `CloudflareTunnelTransport` (v0.1) + `SongbirdTransport` (v0.2 sovereign TCP probe). Error-propagating API client, gate-agnostic credential paths. Async-correct: blocking health checks on `spawn_blocking`, `tokio::fs` for credential reads. **48 unit tests**, `unsafe_code = "forbid"`. 6.5 MB release binary
- **Multi-tier test suite**: observer + reviewer + compute + hub + pappusCast health (`deploy/tier_test_all.sh`)
- **DNS exfil closed**, **supply chain locked**, **crontab restricted**, **version disclosure suppressed**

### Sovereignty Evolution

- **K-Derm diderm architecture**: outer (GitHub extracellular ledger) / inner (VPS golgiBody) / intracellular (gate source of truth)
- **VPS as touchpoint, gate as source**: VPS terminates TLS, relays traffic, caches content. Gate hardware runs full NUCLEUS
- **Forgejo PRIMARY**: 39 repos across 3 orgs. K-Derm diderm relay (push forgejo only → relay → GitHub via golgiBody-ext). No dual-push
- **Content-aware routing**: `routing_config.toml` — static→VPS cache, auth/API/git→gate, large→Songbird P2P
- **Trust model**: covalent/ionic/metallic/weak bonding maps to content access scopes in routing
- **VPS Tower LIVE**: BearDog crypto + SkunkBat audit + Songbird relay + RustDesk + Caddy TLS shadow
- **Channel 3 TLS LIVE**: `membrane.primals.eco` → VPS (DNS-only A record), ACME cert auto-obtained (Let's Encrypt E8). Caddy serves sporePrint + health/status endpoints on :443. HTTP parity PASS (68ms vs 89ms GitHub Pages)
- **DO credentials encrypted**: BearDog AES-256-GCM with Argon2id on VPS
- **40+ dependencies mapped** across 7 clusters (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines captured**: 9-day summary (950 samples) — TTFB p50=119ms p95=190ms, TLS p50=73ms p95=101ms
- **benchScale framework** operational — 5 scenarios, 3 pentest scripts
- **L3+L4 membrane bridge**: Layer 3 (external membrane) and Layer 4 (internal membrane) connected via unified telemetry pipeline. `routing_config.toml [telemetry]` formalizes `shadow_mode = "permanent"` with SkunkBat audit correlation
- **6 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub (JH-0→JH-11), primal deep debt

### sporePrint (Extracellular)

- `primals.eco` served permanently by GitHub Pages + Cloudflare CDN (extracellular surface)
- 15+ notebooks across commons/, showcase/, data/, pilot/, validation/
- Auto-refresh CI across 26 repos; `sporeprint/` directories in all 8 springs
- Local preview via `deploy/sporeprint_local.sh` (dev tool, not production path)

## Quick Start

```bash
# Deploy a Node Atomic via Rust CLI
nucleus-deploy deploy --composition node --gate mygate

# Deploy VPS standard (Wave 56): zero TCP ports, UDS-only
nucleus-deploy deploy --composition nest --uds-only

# Run five-layer security validation
nucleus-deploy security --layer all

# Collect membrane telemetry
nucleus-deploy telemetry --mode all

# Emit pseudoSpore from workload
nucleus-deploy spore --workload workloads/wetspring/wetspring-16s-rust-validation.toml
```

### Agent Loop (signal_plan → signal.dispatch)

With the agent composition running, use `signal_executor.sh` to close the agent
loop — Squirrel plans, biomeOS executes:

```bash
# Plan + dispatch: natural language → atomic signals
bash deploy/signal_executor.sh "check the health of all tower primals"

# Plan only (inspect the signal plan without dispatching)
bash deploy/signal_executor.sh --plan-only "deploy a nest composition"

# Dispatch a single signal directly (bypass planning)
bash deploy/signal_executor.sh --signal tower.health

# Dry run (show what would dispatch without executing)
bash deploy/signal_executor.sh --dry-run "store this data securely"
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
HPC. All gardens products running on projectNUCLEUS. sunCloud metabolic
economics. Zero external dependencies.

See [PHASES.md](PHASES.md) for detailed phase architecture.

## Repo Structure

```
specs/              Local specs: execution model, composition, security, tunnel evolution, dependency inventory
gates/              Gate inventory and hardware configs
deploy/             Deployment tooling, test suites, pappusCast daemon
  nucleus-deploy/   Rust binary: 9 subcommands (security, provenance, deploy, spore, telemetry, summary, verify, provision, dns)
  nucleus_config.sh Gate-agnostic config (all paths, ports, env vars — single source of truth)
  forgejo_mirror.sh Forgejo org/repo creation + relay push for all repos
  gate_watchdog.sh  Membrane health monitor (lab/git endpoints, logs for skunkBat)
  gate_switch.sh    Migrate compute services between gates
  pappusCast.py     Tiered auto-propagation daemon (workspace → observer surface)
  observer_server.py Static HTTP server for pre-rendered observer HTML (port 8866)
  legacy/           Fossilized scripts: cloudflare configs, cloudflared provisioning, songbird relay
graphs/             Deploy graph TOMLs — curated from primalSpring + RootPulse workflows
  tower_agent.toml  Agent composition: Tower + biomeOS neural-api + Squirrel (agentic AI)
workloads/          Workload catalog (TOML specs for toadStool)
  wetspring/        Validated wetSpring science workloads (8 Rust + 2 Python + 1 deferred)
  templates/        Templates for new workloads
validation/         Composition validation, security pen tests, upstream gap handbacks
  dark_forest_gate_local.sh  Dark Forest Glacial Gate 5-pillar structural validation (33 checks)
  darkforest_membrane.sh     cellMembrane VPS remote audit (MEM-01 through MEM-13)
  darkforest/       Pure Rust security validator (v0.2.1 — pen test + fuzz + crypto, modular submodules)
  tunnelKeeper/     Rust crate for tunnel health (Cloudflare v0.1 + Songbird v0.2)
  baselines/        Tunnel metrics + unified membrane telemetry (cron CSVs + membrane_7day.toml)
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
| **cellMembrane** | gardens | **Private** ops repo — VPS state, runbooks, credential procedures for the cellMembrane fieldMouse deployment |
| **projectFOUNDATION** | gardens | The soil — validated scientific lineage, gap handbacks, bonding models, domain threads |
| **helixVision** | gardens | Genomics product — runs on projectNUCLEUS |
| **esotericWebb** | gardens | Creative product — runs on projectNUCLEUS |
