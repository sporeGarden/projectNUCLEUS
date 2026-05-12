# Phase Architecture — projectNUCLEUS

Each phase builds on the previous. No phase is thrown away — Phase 1 covalent
bonding remains the foundation even when Phase 4 metallic federation is live.

---

## Phase 1: Covalent LAN HPC

**Status**: Validated (2026-05-06), full provenance pipeline operational
**System**: active gate (i9-14900K, 96 GB DDR5, RTX 4070 / RTX 3090)
**Bonding**: Covalent (shared family seed, full trust)
**Composition**: Full NUCLEUS (13 primals, dynamically discovered from plasmidBin)
- Tower: BearDog + SongBird
- Compute: ToadStool + BarraCuda + CoralReef
- Storage: NestGate
- Provenance: rhizoCrypt + loamSpine + sweetGrass
- Coordination: Squirrel
- Defense: skunkBat
- Meta: biomeOS (orchestration) + petalTongue (UI)

The active gate is our local development and validation system. Phase 1 proves
that primalSpring's composition patterns work on real hardware with real
science workloads.

### What Works

- Full NUCLEUS (13 primals) deployed via `deploy.sh --composition full --gate <active-gate>`
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

### Security Posture (Phase 59 — Zero Open Gaps)

All security gaps from the Phase 2a pen test have been resolved upstream (primalSpring v0.9.24, Phase 59):

| PG | Resolution |
|----|-----------|
| PG-55 | All 13 primals default `127.0.0.1`. `--bind` flags implemented across the board. |
| PG-56 | NestGate BTSP method-level auth gating. 10-method exempt whitelist (health, identity, capabilities). |
| PG-57 | skunkBat multi-dimensional anomaly detection (connection rate + traffic volume + port diversity). |
| PG-58 | Songbird `--bind` for HTTP server, `--listen` for IPC socket (separate concerns). |
| PG-59 | sweetGrass `--http-address` and `--port` both accept `host:port`. |

**Bind policy**: Deploy scripts no longer need explicit `--bind 0.0.0.0` overrides — all primals default localhost. `PrimalDeployProfile.bind_flag` returns `Some(flag)` for all 13 primals.

- `deploy/security_validation.sh` — automated five-layer pen testing (OS, API, application, tier enforcement, dark forest)
- skunkBat multi-dimensional anomaly detection live (12 normal + 7 attack patterns seeded)
- Run before and after every evolution step

---

## Phase 2: Ionic Compute Sharing

**Status**: Step 2b operational (2026-05-10) — cell membrane architecture, static observer, pappusCast, multi-tier testing, tunnelKeeper v0.2.0
**System**: active gate + NUC intake
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
[the active gate / JupyterHub + Nest Atomic + provenance]
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
via BTSP, and gets a JupyterHub notebook environment on the active gate:

```
Browser → primals.eco → BTSP tunnel (BearDog + Songbird NAT traversal)
       → petalTongue reverse proxy → JupyterHub on the active gate
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
- JupyterHub validated on the active gate (Phase 1)

### Step 2a Validated and Hardened (2026-05-06 → 2026-05-07)

Cloudflare tunnel established, hardened, and baselines capturing:

**Tunnel + Lab (2026-05-06)**:
- `cloudflared` 2026.3.0 → QUIC tunnel → Cloudflare ORD edge → `lab.primals.eco`
- Tunnel latency p50: **270ms** (5-sample median via trycloudflare.com)
- External validation: **15/15 PASS**, Local validation: **12/12 PASS**
- See `validation/EXTERNAL_PIPELINE_VALIDATION_MAY06_2026.md`

**System Hardening (2026-05-07)**:
- UFW activated: deny-by-default, allow SSH/LAN/localhost
- JupyterHub security headers added (X-Frame-Options, nosniff, referrer, server suppressed)
- Three-layer pen test: `validation/archive/security-20260507-110312/SECURITY_RESULTS.md`

**Sovereignty Evolution (2026-05-07)**:
- Hourly baseline capture via cron (`validation/baselines/capture_tunnel_metrics.sh`)
- `infra/benchScale/` framework created — 5 parity scenarios, 3 pentest scripts
- `specs/SOVEREIGNTY_VALIDATION_PROTOCOL.md` — master cutover document
- `specs/TUNNEL_EVOLUTION.md` — updated with concrete implementation for Steps 2b→4
- `specs/COMPLETE_DEPENDENCY_INVENTORY.md` — 40+ dependencies across 7 clusters mapped

**Forgejo Calibration Instrument (2026-05-07)**:
- Forgejo v15.0.0 installed as `forgejo.service` on port 3000
- Accessible via `git.primals.eco` (Cloudflare tunnel ingress)
- projectNUCLEUS mirrored, SQLite backend, registration disabled
- Calibration target for RootPulse parity

**RootPulse Commit Workflow (2026-05-07)**:
- Ported `rootpulse_commit.toml` from biomeOS to `graphs/`
- Tested all 6 phases against live primals:
  - Phase 1 (health): PASS — rhizoCrypt + LoamSpine alive
  - Phase 2 (dehydrate): PASS — DAG session created, merkle root returned
  - Phase 3 (sign): PASS — BearDog Ed25519 signature
  - Phase 4 (store): PASS — NestGate KV storage
  - Phase 5 (commit): FAIL — LoamSpine API param mismatch with graph spec
  - Phase 6 (attribute): PASS — sweetGrass braid with W3C PROV witness
- 5 upstream gaps documented in `validation/ROOTPULSE_GAPS_HANDBACK.md`

**Multi-User Pentest and Hardening (2026-05-07)**:
- Ran `nucleus-security-validation.ipynb` from compute-tier user (tamison)
- 13/13 primals reachable on localhost (expected), NestGate write allowed without auth (JH-0)
- Filesystem isolation 10/10 PASS (shadow, homes, tunnel config, sqlite all denied)
- sweetGrass dual-port clarified: 9850=IPC (plaintext by design), 9851=BTSP
- **hidepid=2** on /proc — users can't see other processes (persistent via fstab)
- **iptables/ip6tables** outbound DROP for ABG UIDs 1001-1099 (persistent via systemd)
- **Reviewer lockdown**: terminals disabled, kernels blocked (NoKernelManager), filesystem 550 root-owned
- **Shared notebooks immutable**: chmod 444, compute users run but can't save back
- **Compute/save separation**: shared templates read-only, per-user `~/notebooks/results/` for outputs

**Automated Tier Enforcement (2026-05-08)**:
- `deploy/tier_enforcement_test.sh` — 44 OS-level assertions via `sudo -u` (filesystem, network, process)
- `deploy/jupyterhub_tier_test.py` — 18 JupyterHub API probes (kernels, terminals, file write, Voila)
- Wired into `security_validation.sh` as Layer 4 (total: 62 assertions, 0 FAIL, 0 KNOWN_GAP in enforced mode)
- **JH-6 found**: `KernelSpecManager.allowed_kernelspecs` only filters listing, not creation — bypassed by NoKernelManager
- **JH-7 found**: Voila executes notebooks as hub user — mitigated by restricting to curated showcase only

**Voila Dashboard Service (2026-05-08, replaced 2026-05-10)**:
- Voila 0.5.12 originally served dynamic notebooks on port 8866 (every visit = kernel launch)
- **Replaced by static observer**: pappusCast pre-renders all notebooks to HTML via
  `nbconvert --execute --to html --no-input`; `observer_server.py` serves from disk on :8866
- Zero compute per visit, instant page loads, bot-safe, inter-notebook navigation
- Voila remains available for reviewer/user tiers behind JupyterHub auth
- Switchover: `deploy/switch_to_static_observer.sh` / `switch_to_voila_observer.sh`

**Dark Forest Security Hardening (2026-05-08)**:
- Pure Rust `validation/darkforest/` v0.2.0 — modular pen test + fuzz + crypto validator
- 14 primals + JupyterHub fuzzed, 3 threat actors (external, compute, reviewer/observer)
- `security_validation.sh` invokes Rust darkforest binary directly
- Legacy bash/python scripts archived to `validation/archive/legacy/`
- **DF-1 RESOLVED**: Phase 60 binaries (PG-55) default all 13 primals to `127.0.0.1`. Verified: all 14 primal ports on localhost, no UFW workaround needed. deploy.sh DF-1 comments removed
- **JH-8 FIXED**: DNS port 53 was open to all external servers — exfiltration channel closed, restricted to local stub resolver only
- **JH-9 FIXED**: Shared conda envs group-writable — now root-owned with 755 permissions
- **JH-10 found**: `/hub/api/` version disclosure is a built-in handler that cannot be overridden via config. X-JupyterHub-Version and Server headers suppressed
- `/etc/crontab` restricted to 640 — ABG users cannot enumerate scheduled tasks
- `shared/data/` and `shared/projects/` restricted to `abg-compute` group — reviewer/observer blocked

**Phase 60 Upstream Absorption (2026-05-08)**:
- Pulled primalSpring v0.9.25 (`d56d2ad`), all 15 primals at HEAD, plasmidBin `v2026.05.08`, wateringHole
- **JH-0 ADOPTED**: 13/13 primals ship MethodGate with `auth.check`/`auth.mode`/`auth.peer_info`. 9/13 respond on TCP. Permissive mode active (log + allow)
- **JH-1 RESOLVED**: BearDog `identity.create` → `auth.issue_ionic` → `auth.verify_ionic`. Ed25519-signed ionic tokens with scope, expiry, JTI
- **JH-2 RESOLVED**: biomeOS v3.48 resource envelope enforcement (`timeout_ms`, `cpu`, `mem`). ToadStool S232 enforces all dispatch paths
- **JH-3 RESOLVED**: biomeOS `composition.reload` — hot-swap single primal without full restart
- **JH-4 RESOLVED**: BearDog `auth.issue_session` — purpose-based presets
- **JH-5 Phase 2 COMPLETE**: skunkBat `security.audit_log` — 1024-event ring buffer, 7 event kinds, cursor-based polling
- **GAP-11 CLOSED**: barraCuda 18/18 methods (71 total JSON-RPC methods)
- **Registry**: 389 methods across 82 domains
- **plasmidBin sync gap found**: `git pull` updates checksums but doesn't validate/refresh local binaries. Created `sync.sh` to detect stale binaries via checksum mismatch and re-fetch. Fixed `fetch.sh --force` bug (didn't delete before re-download)

**Phase 60 Revalidation (2026-05-08)**:
- Full 5-layer security validation with Phase 60 binaries: **263 PASS, 0 FAIL, 2 WARN**
- **DF-1 RESOLVED**: All 14 primal ports on `127.0.0.1` — PG-55 default binding confirmed in v2026.05.08 binaries
- **MethodGate (JH-0) confirmed**: All 4 ABG tiers detect permissive mode on `beardog:9100`
- **1 KNOWN_GAP remaining**: `nestgate storage.list` accessible without auth in permissive mode (will auto-resolve when `NUCLEUS_AUTH_MODE=enforced`)
- **4 DARK_FOREST findings**: version disclosure (JH-10), systemd service enumeration, reviewer python3 access (terminals blocked), null byte reflection (CSP mitigates)
- **2 WARN**: sweetgrass secondary port on 0.0.0.0 (ephemeral, not configured port), rustdesk listener
- Removed DF-1 workaround code from `security_validation.sh`, `deploy.sh`
- plasmidBin `sync.sh` verified: 13/13 binaries checksum-matched

**Enforced Mode Activation (2026-05-08)**:
- Switched `NUCLEUS_AUTH_MODE` from `permissive` to `enforced` — default for all deployments
- `deploy.sh` now exports `*_AUTH_MODE=enforced` env vars for all 13 primals
- **JH-0 FULLY RESOLVED**: 10/13 primals confirmed enforced via TCP at this date (now 13/13). All unauthenticated RPC calls return `-32001 PERMISSION_DENIED`
- **Ionic token flow validated**: `identity.create` → `auth.issue_session(purpose="jupyterhub")` → scoped Ed25519 token → `_bearer_token` in RPC params → MethodGate accepts → method dispatches
- **Scope rejection confirmed**: Token with `crypto.*` scope can call `capabilities.list` but token verified on nestgate can't call `storage.list` (wrong scope)
- **Cross-primal gap (JH-11)**: Beardog-issued tokens not verifiable by other primals — each MethodGate validates independently. biomeOS composition forwarding is the intended cross-primal path
- **DF-2 found**: toadstool reads `TOADSTOOL_AUTH_MODE=enforced` env var but reports `permissive` via `auth.mode` — env var mapping or implementation gap
- **3 primals silent on TCP**: songbird, squirrel, petaltongue don't expose `auth.mode` on TCP (petaltongue rejects all unauthenticated TCP via BTSP PT-09 enforcement — stricter than MethodGate)
- Full validation: **265 PASS, 0 FAIL, 0 KNOWN_GAP, 1 WARN, 5 DARK_FOREST**
- Previous KNOWN_GAP (nestgate `storage.list`) is now PASS

**darkforest v2.0 — Modular Rust Security Validator (2026-05-08)**:
- Refactored `validation/darkforest/` into 7 modules: `check.rs`, `net.rs`, `pentest.rs`, `fuzz.rs`, `crypto.rs`, `report.rs`, `main.rs`
- 13 cryptographic strength checks (CRY-01 → CRY-13): cookie entropy/age/perms, shadow hash algo/rounds, ionic token tamper/expiry, BTSP cipher negotiation, file permission sweep
- Structured JSON output (`--output <path>`) for auditable, machine-readable reports
- **175 PASS, 0 FAIL, 6 DARK_FOREST** (authoritative count; supersedes `security_validation.sh` pipeline totals)
- Legacy scripts (`deploy/darkforest_pentest.sh`, `deploy/darkforest_fuzz.py`) archived to `validation/archive/legacy/`

**ABG Workspace Scaffolding (2026-05-08)**:
- Pilot lifecycle: commons → pilot → projects → showcase, with `abg_accounts.sh create-pilot` subcommand
- Per-user private scratch: `~/notebooks/scratch/` (chmod 700)
- Reviewer showcase-only visibility: symlink-level isolation (reviewers see `~/notebooks/showcase/` instead of full `~/notebooks/shared/`)
- Tier-appropriate `Welcome.ipynb` symlinked at login for all 4 tiers
- Validation dashboard (`showcase/validation-dashboard.ipynb`) surfaces darkforest JSON via static HTML
- READMEs seeded in `data/`, `projects/`, `pilot/`, `validation/`

**ABG Compute Usability (2026-05-08)**:
- Per-user Python venvs (`~/.venv/bioinfo/`, `--system-site-packages` on shared bioinfo env)
- Local wheelhouse (`/home/irongate/shared/abg/wheelhouse/`) for offline `%pip install`
- `pre_spawn_hook` PATH priority for user venv binaries
- `Getting-Started.ipynb` onboarding notebook in commons
- `deploy/wheelhouse_sync.sh` admin utility for wheelhouse management

**Upstream Gap Handbacks Delivered**:
- `validation/PETALTONGUE_GAPS_HANDBACK.md` — 5 gaps (PT-1→PT-5)
- `validation/NESTGATE_CONTENT_GAPS_HANDBACK.md` — 4 gaps (NG-1→NG-4)
- `validation/ROOTPULSE_GAPS_HANDBACK.md` — 5 gaps (RP-1→RP-5)
- `validation/JUPYTERHUB_PATTERNS_HANDBACK.md` — 5 gaps (JH-0→JH-5) — all now resolved or adopted upstream
- Dark Forest gaps: JH-8 (DNS exfil — FIXED), JH-9 (supply chain — FIXED), JH-10 (version disclosure — upstream), DF-1 (binding — RESOLVED Phase 60)

### Step 2b: Open Observer + Auto-Propagation (2026-05-09)

**Open Observer Landing**:
- Observer is the default landing page — no credentials, no login
- Static pre-rendered HTML at `lab.primals.eco` (source stripped, inter-notebook nav, index page)
- Originally served by Voila (dynamic); replaced by `observer_server.py` serving pappusCast HTML exports
- All notebooks have `metadata.title` for clean rendering and navigation
- Admin templates and internal paths return 404

**pappusCast Auto-Propagation**:
- Python daemon (`deploy/pappusCast.py`) propagates validated content from workspace to observer
- Named for the dandelion pappus — the parachute that carries seeds to new ground
- Three validation tiers: Light (on-change), Medium (periodic execution), Heavy (~6h regression)
- Adaptive rate limiting: `min(BASE_MINUTES * max(1, active_users), MAX_MINUTES)`
- Snapshot architecture: `public/` holds managed copies, not live symlinks — stable observer surface
- State tracking: `.pappusCast/last_publish.json`, `changelog.jsonl`, `quarantine/`
- systemd service: `pappusCast.service` — persistent, restarts on failure
- Evolution: Python (now) → Rust binary → pappusCast primal

**Multi-Tier Test Suite**:
- `deploy/tier_test_observer.py` — structural checks, execution, HTTP behavior, source stripping
- `deploy/tier_test_reviewer.py` — access control, parse, no-write enforcement
- `deploy/tier_test_compute.py` — venv, packages, kernels, notebook execution
- `deploy/tier_test_all.sh` — unified runner across all tiers + pappusCast health
- Test suite identified and fixed: kernel mismatches on 8 notebooks, missing metadata titles,
  relative path errors after snapshot conversion, dashboard KeyError on status keys,
  package import issues (biopython → Bio), NUCLEUS_TIER env var location

**Cloudflare Access + tunnelKeeper**:
- `deploy/cloudflare/access_setup.sh` — Cloudflare Access policies for reviewer/user gating
- `validation/tunnelKeeper/` — Rust crate for tunnel health, DNS resolution, config parsing
- Integrated into darkforest pen test as A6 (tunnel health verification)

**Deep Debt Sweep (2026-05-09)**:
- `deploy/nucleus_config.sh` centralized as single source of truth — all paths (`$GATE_HOME`), ports, Cloudflare IDs derive from env vars
- `deploy/nucleus_paths.py` provides equivalent Python config module for tier tests and pappusCast
- tunnelKeeper: `Client::new()` returns `Result` (zero `expect()`), tokio slimmed to `rt-multi-thread+macros`, `rand` replaced by `rand_core`, credential paths env-var-driven
- darkforest: PRIMALS array loaded from env with compiled fallback, rhizoCrypt RPC 9602 added to roster, crypto/pentest paths gate-agnostic via `CryptoConfig` struct
- `security_validation.sh` invokes Rust darkforest binary directly (archived bash/python scripts)
- `pappusCast.py`: broad `except Exception` blocks narrowed to `subprocess.SubprocessError`, `json.JSONDecodeError`, `OSError`, `urllib.error.URLError`
- 7 deploy scripts wired to source `nucleus_config.sh` (sporeprint_local, sporeprint_verify, sporeprint_dns, rotate_cookie_secret, gate_switch, tier_enforcement_test, external_validation)
- 96 "ironGate" display references scrubbed across 23 docs → gate-anonymous terms
- Zero TODO/FIXME/HACK remaining, zero clippy warnings

**Cell Membrane Architecture (2026-05-10)**:
- Architectural inversion: `primals.eco` DNS permanently set to GitHub Pages A records (extracellular layer)
- `lab.primals.eco` + `git.primals.eco` routed through Cloudflare tunnel replicas (membrane layer)
- Sovereign compute, primals, and data remain internal (intracellular layer)
- `deploy/gate_provision.sh` provisions remote hosts as membrane replicas (sub-second failover)
- `deploy/gate_watchdog.sh` monitors membrane health, logs state for skunkBat audit (no DNS swapping)
- `tunnelKeeper v0.2.0` reports replica count, unique origins, and edge colos
- `sporeprint-local.service` demoted from production to development preview tool
- Key insight: accept that the extracellular world (CDN, DNS) is uncontrolled; inside the membrane, total sovereign control. The boundary enables future ionic/weak bonding as gated channels.

**Deep Debt Evolution Sweep (2026-05-11)**:
- `pappusCast.py` (953L) smart-refactored into 7 modules: `pappuscast/{config,state,tiers,publisher,export,daemon}.py` + thin CLI entry point (146L)
- Zero hardcoded `/home/irongate` in Rust code: `pentest.rs` (12 hits) and `crypto.rs` (1 hit) → `gate_home()` env-var pattern
- 4 systemd units → `EnvironmentFile=/etc/projectnucleus/gate.env` + `${GATE_HOME}` substitution; `gate_provision.sh` installs env file on replicas
- `tier_test_observer.py` → `validation/archive/legacy/` (superseded by darkforest `--suite observer`)
- `except Exception` blocks narrowed in `tier_test_compute.py` and `jupyterhub_tier_test.py`
- Spec reconciliation: `COMPLETE_DEPENDENCY_INVENTORY.md` (Cluster 7 → ~90%, JH-11 added), `VALIDATION_RESULTS.md` (security table updated), `TUNNEL_EVOLUTION.md` (Step 3a cell membrane context)
- Upstream handback delivered: toadStool/squirrel MethodGate insertion points, barraCuda crypto delegation to bearDog IPC, squirrel `LocalProcessProvider` → toadStool dispatch
- MethodGate was 11/13 at time of sweep (toadStool + squirrel pending) — now **13/13** (upstream resolved)

**NestGate Session 60 + Full Debt Resolution (2026-05-11)**:
- NestGate shipped `content.*` transport parity: all 8 methods (`put`, `get`, `exists`, `list`, `publish`, `resolve`, `promote`, `collections`) on all 4 transports (primary, SemanticRouter, IPC, HTTP)
- H2-05 **DONE**. H2-06 through H2-09 all **UNBLOCKED** (petalTongue `backend=nestgate`, shadow run, cutover)
- All per-primal debt closed: toadStool env expansion (contract), squirrel `RemoteComputeProvider`, barraCuda crypto delegation to bearDog IPC, loamSpine method aliases, skunkBat JH-5 Phase 3 forwarding, petalTongue SPA+CORS
- primalSpring at Wave 7-9: 413 methods, 301 exercised (72%), 22 scenarios, 77 graphs, semantic contract tests
- L1 CLEAN: 13/13 structural + semantic, zero critical gaps. Stadial-ready on Pillar 1.
- **Next priorities**: shadow runs (content parity, BearDog TLS, Songbird NAT), `composition.deploy(graph)`, lithoSpore Tier 1

### ABG Tiered Access Model

Three tiers, simplified from four. Observer is open; reviewer and user are gated
by Cloudflare Access + PAM:

| Tier | Access | Capabilities | Surface |
|------|--------|-------------|---------|
| **observer** | Open, unauthenticated | Read-only rendered notebooks, data, dashboards | Static pre-rendered HTML |
| **reviewer** | Cloudflare Access + PAM | Read + run notebooks (showcase-only view) | JupyterHub |
| **user** | Cloudflare Access + PAM | Read + write + run, shared workspace | JupyterHub |

Admin (gate owner account) owns infrastructure. Users do science. Reviewers validate.
Observers see everything rendered but interact with nothing.

**Reviewer = peer review / PI validation.** Sees code in JupyterLab (read-only),
runs pipelines in showcase notebooks. No arbitrary code execution.

**Observer = public window.** Pre-rendered HTML — no kernel launches, no compute
per visit. View-only rendered output plus provenance chains. The entire project
is functionally exposed but not interactable — science as read-only artifact.

**Admin/user separation.** The system owner has both an admin account (local gate owner,
hardware control) and a user account (ABG member, does science). In later stages,
admin owns hardware, not data — the sovereignty separation.

`pre_spawn_hook` sets resource limits, `NUCLEUS_TIER` environment variable,
primal port configuration, and shared workspace symlinks. Reviewer and observer
tiers enforce no-execution via `NoKernelManager` (blocks `start_kernel()` —
more reliable than `KernelSpecManager.allowed_kernelspecs` which only filters
listing), `--ServerApp.terminals_enabled=False`, and filesystem permissions
(root-owned `chmod 550` notebook directory). Voila serves notebooks with
`strip_sources=False` (code visible for scientific transparency).

**Security note**: The original `NUCLEUS_READONLY=1` env var was a convention
flag with zero enforcement — JupyterLab ignored it entirely (JH-0 pattern).
Replaced with mechanism-level enforcement at application and filesystem layers.

### ABG Shared Workspace

Shared space at `/home/irongate/shared/abg/` (see `specs/SHARED_WORKSPACE.md`):

```
commons/     — group scratch (quick experiments, all members)
pilot/       — structured experiments (hypothesis, criteria, timeline)
projects/    — formal project spaces (abg_accounts.sh create-project)
data/        — shared datasets (NCBI, reference genomes, calibration)
templates/   — starter notebooks, workload TOMLs, tier-appropriate welcome notebooks
showcase/    — polished work for external review (PIs, HPC admins) + dashboards
validation/  — surfaced darkforest JSON reports
```

**Workspace lifecycle**: scratch → commons → pilot → projects → showcase → public

**Per-user landing zone**: `~/notebooks/Welcome.ipynb` (tier-appropriate), `~/notebooks/scratch/`
(chmod 700, private workspace for compute/admin), `~/notebooks/shared/` (symlink to full tree) or
`~/notebooks/showcase/` (reviewer-only, symlink-level isolation).

**Compute usability**: per-user Python venvs (`~/.venv/bioinfo/`, `--system-site-packages` on
shared bioinfo), local wheelhouse at `/home/irongate/shared/abg/wheelhouse/` for offline
`%pip install`, `pre_spawn_hook` PATH priority for user venv binaries, `Getting-Started.ipynb`
onboarding notebook in commons.

Google Doc model: all work visible to all members. Reviewers see showcase/ only.
Work elevates from commons/ → projects/ → showcase/ → primals.eco/lab (public).

### sporePrint Integration (primals.eco/lab) — LIVE

**What's live now** on primals.eco (GitHub Pages + auto-refresh CI):
- 5 public notebooks rendered with embedded charts at `/lab/notebooks/` (16S pipeline, benchmarks, paper reproductions, cross-spring connections, soil Anderson)
- Spring science hubs for wetSpring, hotSpring, airSpring, healthSpring at `/lab/springs/`
- "Reproduce It Yourself" guide, provenance pipeline, compute access pages
- `render_notebooks.sh` executes notebooks via `jupyter nbconvert --execute` and wraps output in Zola front matter
- Auto-refresh CI: `notify-sporeprint.yml` in all 26 repos → `auto-refresh.yml` dispatches metric updates (auto-commit) or content PRs (review)
- `sporeprint/` directories in all 8 springs with validation-summary stubs
- `sources.toml` registry maps 15 primals + 8 springs + 3 products to GitHub repos

**What's still planned** (Phase 3 sovereignty):
- NestGate serves sporePrint content directly (no GitHub Pages)
- petalTongue renders live dashboards from Tier 2 JSON-RPC APIs
- BTSP replaces Cloudflare TLS on all external connections
- sporePrint becomes a NUCLEUS composition running on sovereign hardware

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

**Status**: Upstream gaps delivered (petalTongue PT-1→PT-5, NestGate NG-1→NG-4 all shipped). Cell membrane architecture enables incremental sovereignty — extracellular (CDN) remains as reliable fallback while membrane channels are progressively replaced.
**Bonding**: Covalent core + public weak endpoint
**New Primals**: petalTongue (UI), BTSP Phase 3

The membrane layer (Cloudflare Tunnel) is the sovereignty target. The
extracellular layer (GitHub Pages CDN for primals.eco) remains until
petalTongue can serve sovereign content at parity. Each step validated
against CDN baselines.

### Gap Discovery (2026-05-07)

Live testing of petalTongue and NestGate revealed the gaps that block
Phase 3 execution. These are documented as upstream handbacks:

**petalTongue** (`validation/PETALTONGUE_GAPS_HANDBACK.md`):
- PT-1 (High): `web` mode has 6 hardcoded routes, no catch-all for static files
- PT-2 (High): No NestGate backend integration
- PT-3 (Medium): No web-mode config schema (docroot, backend, cache)
- PT-4 (Medium): Deploy mode mismatch (`server` vs `web`)
- PT-5 (Low): `--workers` flag accepted but unused
- **Shortest fix**: ~10 lines of Rust — `ServeDir` fallback with `--docroot` flag

**NestGate** (`validation/NESTGATE_CONTENT_GAPS_HANDBACK.md`):
- NG-1 (High): No content-addressed storage (BLAKE3 hash as auto-key)
- NG-2 (Medium): No collection/manifest for versioned releases
- NG-3 (Medium): Blob store and KV store are separate namespaces
- NG-4 (Low): No streaming store for large content
- **Workaround**: Client-side BLAKE3 hashing + KV manifest + blob store works today

### Architecture

```
[Browser] → primals.eco (Cloudflare DNS, then sovereign)
         → [NucBox M6 intake / cloudflared → petalTongue]
         → [the active gate / NestGate content store + JupyterHub]
```

### Convergence Path

```
Now:     primals.eco on GitHub Pages CDN (extracellular), lab/git via tunnel (membrane)
Phase 3a: NestGate serves sporePrint content → petalTongue rendering (sovereign extracellular)
Phase 3b: BTSP replaces Cloudflare TLS    →  membrane encryption is sovereign
Phase 3c: Songbird NAT replaces CF tunnel →  membrane channels are sovereign
Phase 4:  sovereign DNS                   →  zero external dependencies
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
