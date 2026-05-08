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
- **Reviewer/observer lockdown**: NoKernelManager blocks all kernel creation, no terminals, filesystem read-only (chmod 550 root-owned)
- **Voila compute contracts**: Reviewers see code + run pipelines via Voila widgets (server-side execution, no kernel). Observers see rendered output + provenance. Code visible for scientific transparency (`strip_sources=False`). Calibration instrument for petalTongue sovereignty replacement
- **Shared notebooks immutable**: compute users can run but not save back (chmod 444, per-user results dirs)
- JupyterHub security headers (X-Frame-Options, X-Content-Type-Options, Referrer-Policy, Server suppressed)
- **MethodGate (JH-0) ADOPTED**: 13/13 primals ship MethodGate pre-dispatch auth gate (primalSpring Phase 60). 9/13 respond to `auth.mode` on TCP. Permissive mode (log + allow) active. Set `NUCLEUS_AUTH_MODE=enforced` to activate scope-based rejection
- **Ionic tokens (JH-1) LIVE**: BearDog `identity.create` → `auth.issue_ionic` → `auth.verify_ionic`. Ed25519-signed scoped tokens with expiry and JTI
- **Resource envelopes (JH-2) RESOLVED**: biomeOS v3.48 enforces `timeout_ms`, ToadStool S232 enforces `mem_mb`, `cpu_cores`, `max_timeout_ms` on all dispatch paths
- **Composition reload (JH-3) RESOLVED**: biomeOS `composition.reload` — hot-swap single primal without full restart
- **Session UX (JH-4) RESOLVED**: BearDog `auth.issue_session` — purpose-based presets (`jupyterhub`, `desktop`, `admin`)
- **Audit log (JH-5) Phase 2 COMPLETE**: skunkBat `security.audit_log` — 1024-event ring buffer, 7 event kinds, cursor-based polling. Cross-primal forwarding deferred
- All PG-55 through PG-62 resolved by primalSpring Phase 60
- **All 14 primal ports bound `127.0.0.1`** (Phase 60 binaries ship PG-55 default). NestGate BTSP method-level auth, skunkBat anomaly detection
- **Automated tier enforcement**: 62 assertions (44 OS-level + 18 JupyterHub API) validate all 4 ABG tiers (`deploy/tier_enforcement_test.sh`, `deploy/jupyterhub_tier_test.py`)
- **Dark Forest hardening**: 5-layer security validation pipeline — **263 PASS, 0 FAIL** (`deploy/security_validation.sh`)
- **Pen test + fuzz coverage**: adversarial pen test (`darkforest_pentest.sh`), protocol fuzzing all 13 primals + JupyterHub (`darkforest_fuzz.py`), timing analysis
- **DNS exfil closed**: iptables DNS rules restricted to local stub resolver (127.0.0.53), external DNS blocked for ABG UIDs
- **Supply chain locked**: shared conda envs root-owned, 755 — compute users cannot plant malicious packages
- **Version disclosure suppressed**: X-JupyterHub-Version and Server headers emptied; /hub/api/ version is JH-10 upstream gap
- **Crontab restricted**: `/etc/crontab` set to 640 — ABG users cannot enumerate scheduled tasks
- **Shared workspace boundaries**: `data/` and `projects/` restricted to `abg-compute` group — reviewer/observer cannot access
- skunkBat surveillance targets identified: JupyterHub auth events, NestGate writes, iptables DROPs, process enumeration

### Sovereignty Evolution

- **40+ dependencies mapped** across 7 clusters including internal primal gaps (`specs/COMPLETE_DEPENDENCY_INVENTORY.md`)
- **Cloudflare baselines capturing** hourly via cron (DNS, TCP, TLS, TTFB, total latency)
- **benchScale framework** operational (`infra/benchScale/`) — 5 scenarios, 3 pentest scripts
- **Forgejo calibration instrument** installed — baseline for RootPulse parity targets
- **RootPulse commit workflow tested** — 5/6 phases pass against live primals, Phase 5 (LoamSpine commit) has param mismatch
- **Voila baselines captured**: ~600ms render latency, 33–51KB output, source stripping active (`validation/baselines/`)
- **4 upstream gap handbacks** delivered: petalTongue (PT-1→PT-5), NestGate (NG-1→NG-4), RootPulse (RP-1→RP-5), JupyterHub patterns (JH-0→JH-5)
- **JH-0 ADOPTED**: MethodGate shipped 13/13 primals. Permissive mode active (log + allow). Enforced mode blocks unauthenticated calls via ionic token scope check. Next: activate enforced mode in staging
- **JH-6**: `KernelSpecManager.allowed_kernelspecs` only filters listing, not creation — bypassed by `NoKernelManager` override
- **JH-7**: Voila executes notebooks as hub user (privilege escalation risk) — mitigated by restricting to curated showcase only
- **JH-8 (New)**: DNS port 53 was open to all external servers — exfiltration channel. **FIXED**: restricted to local resolver only
- **JH-9 (New)**: Shared conda envs were group-writable — supply chain poisoning vector. **FIXED**: root-owned, 755
- **JH-10 (New)**: `/hub/api/` version disclosure (built-in handler, cannot override in config) — document and block at tunnel
- **DF-1 RESOLVED**: Phase 60 binaries (PG-55) default all 13 primals to `127.0.0.1`. All 14 primal ports verified bound to localhost only — no UFW workaround needed

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
