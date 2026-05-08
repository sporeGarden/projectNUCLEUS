# Phase Architecture — projectNUCLEUS

Each phase builds on the previous. No phase is thrown away — Phase 1 covalent
bonding remains the foundation even when Phase 4 metallic federation is live.

---

## Phase 1: Covalent LAN HPC

**Status**: Validated (2026-05-06), full provenance pipeline operational
**System**: ironGate (i9-14900K, 96 GB DDR5, RTX 4070 / RTX 3090)
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

- `deploy/security_validation.sh` — automated three-layer pen testing
- skunkBat multi-dimensional anomaly detection live (12 normal + 7 attack patterns seeded)
- Run before and after every tunnel evolution step

---

## Phase 2: Ionic Compute Sharing

**Status**: Step 2a hardened (2026-05-07) — UFW active, baselines capturing, Forgejo live
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
- `specs/COMPLETE_DEPENDENCY_INVENTORY.md` — 35 dependencies across 6 clusters mapped

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
- Wired into `security_validation.sh` as Layer 4 (total: 62 assertions, 0 FAIL, 4 KNOWN_GAP)
- **JH-6 found**: `KernelSpecManager.allowed_kernelspecs` only filters listing, not creation — bypassed by NoKernelManager
- **JH-7 found**: Voila executes notebooks as hub user — mitigated by restricting to curated showcase only

**Voila Dashboard Service (2026-05-08)**:
- Voila 0.5.12 installed as JupyterHub managed service on port 8866
- Serves curated showcase notebooks at `/services/voila/` with dark theme, source stripping
- All ABG users can access dashboards via JupyterHub OAuth (no code exposure)
- Baseline captured: ~600ms render, 33–51KB output, accessible via tunnel
- Calibration instrument for petalTongue sovereignty replacement

**Dark Forest Security Hardening (2026-05-08)**:
- `deploy/darkforest_pentest.sh` — comprehensive adversarial pen test across 3 threat actors (external, compute, reviewer/observer)
- `deploy/darkforest_fuzz.py` — protocol-level fuzzing for all 13 primals + JupyterHub (malformed JSON-RPC, binary probes, timing analysis, auth bypass)
- Wired into `security_validation.sh` as Layer 5 (total: 5 layers, 100+ assertions)
- **DF-1 FIXED**: 5 primals on `0.0.0.0` — deploy.sh now uses `$NUCLEUS_BIND_ADDRESS` (127.0.0.1) for all primals
- **JH-8 FIXED**: DNS port 53 was open to all external servers — exfiltration channel closed, restricted to local stub resolver only
- **JH-9 FIXED**: Shared conda envs group-writable — now root-owned with 755 permissions
- **JH-10 found**: `/hub/api/` version disclosure is a built-in handler that cannot be overridden via config. X-JupyterHub-Version and Server headers suppressed
- Version suppression: `Server` header emptied, `X-JupyterHub-Version` header emptied

**Upstream Gap Handbacks Delivered**:
- `validation/PETALTONGUE_GAPS_HANDBACK.md` — 5 gaps (PT-1→PT-5)
- `validation/NESTGATE_CONTENT_GAPS_HANDBACK.md` — 4 gaps (NG-1→NG-4)
- `validation/ROOTPULSE_GAPS_HANDBACK.md` — 5 gaps (RP-1→RP-5)
- `validation/JUPYTERHUB_PATTERNS_HANDBACK.md` — 5 gaps (JH-0→JH-5), 1 critical
- Dark Forest gaps: JH-8 (DNS exfil), JH-9 (supply chain), JH-10 (version disclosure), DF-1 (binding)

### ABG Tiered Access Model

Four tiers, modeled on scientific peer review (`deploy/abg_accounts.sh`):

| Tier | Group | Resources | Kernel | Sees Code | Runs Pipelines | Saves |
|------|-------|-----------|--------|-----------|----------------|-------|
| admin | `abg-admin` | 48 GB / 16 cores | Yes | Yes | Yes (arbitrary) | Yes |
| user | `abg-compute` | 32 GB / 8 cores | Yes | Yes | Yes (ToadStool) | Yes |
| reviewer | `abg-reviewer` | 8 GB / 4 cores | **No** (NoKernelManager) | Yes | Contracts (Voila widgets) | No (550 fs) |
| observer | `abg-observer` | 4 GB / 2 cores | **No** (NoKernelManager) | Yes (rendered) | No | No (550 fs) |

**Reviewer = peer review / PI validation.** The reviewer sees code in JupyterLab
(read-only filesystem) and can run pipelines via Voila compute contracts — the
notebook defines the contract, Voila executes server-side, the reviewer interacts
through widgets only (e.g., upload own data, select parameters). No arbitrary
code execution, no kernel under reviewer control. Like submitting to a journal:
the reviewer reads your code and results, and if they want to test a data point,
the system runs the fixed pipeline on their input.

**Observer = public window.** View-only rendered output plus provenance chains.
No execution whatsoever.

**Admin/user separation.** The system owner has both an admin account (irongate,
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

All-visible shared space at `/home/irongate/shared/abg/` (see `specs/SHARED_WORKSPACE.md`):

```
commons/     — scratch notebooks, experiments (all members)
projects/    — per-project spaces (abg_accounts.sh create-project)
data/        — shared datasets (NCBI, reference genomes)
templates/   — starter notebooks and workload TOMLs
showcase/    — polished work for external review (PIs, HPC admins)
```

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

**Status**: Blocked on upstream gaps (petalTongue PT-1, NestGate NG-1) — gap handbacks delivered
**Bonding**: Covalent core + public weak endpoint
**New Primals**: petalTongue (UI), BTSP Phase 3

projectNUCLEUS takes short-term ownership of sporePrint. The site
evolves from GitHub Pages → NestGate-backed content → petalTongue
rendering, each step validated against Cloudflare CDN baselines.

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
