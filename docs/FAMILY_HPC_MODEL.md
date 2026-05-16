# Family HPC Model — Personal Computers as Covalent Compute Mesh

**Date**: May 15, 2026
**Status**: Operational (single-gate), design (multi-gate mesh)
**Relates to**: `docs/BONDING_MODELS.md`, `gates/*.toml`,
`gen4/architecture/SOVEREIGN_HPC_EVOLUTION.md`

---

## Principle

Every gate is someone's **personal computer first**, HPC node second.

A family member's gaming PC (northGate: RTX 5090, 9950X3D) is theirs. They
game on it, browse on it, do their work on it. NUCLEUS primals run in the
background at low priority — invisible unless you look for them. When the
owner is idle, toadStool dispatches science workloads. When the owner returns
to high-demand tasks, toadStool yields.

This is not time-sharing. This is not a job queue. This is a living organism
where each cell has its own life but contributes to the collective when
capacity exists.

---

## How It Works

### Enrollment

A new gate joins the family mesh in three steps:

```bash
# 1. Bootstrap NUCLEUS on the machine
./bootstrap.sh --fresh

# 2. Join the family (derives gate seed from family root)
./bootstrap.sh --join-family --family-seed ~/.config/biomeos/family/.beacon.seed

# 3. Songbird discovers the gate automatically
#    (BirdSong UDP multicast 239.255.0.1:4200 on LAN)
#    (TCP fallback for WAN gates like flockGate)
```

After enrollment:
- BearDog authenticates the gate using its lineage seed
- Songbird announces the gate's capabilities to the mesh
- toadStool registers the gate as a dispatch target
- The gate's `[science]` section declares what workloads it accepts

### Resource Sharing

Each gate publishes its **available compute** to the mesh via toadStool:

```toml
# Dynamic state (not in static TOML — computed at runtime)
[compute_state]
cpu_available_percent = 85    # owner using 15% CPU
gpu_available_percent = 100   # GPU idle
ram_available_gb = 72         # 24 GB in use by owner
owner_active = false          # no mouse/keyboard in 10 min
dispatch_accepting = true     # gate is willing to accept work
```

**Dispatch rules:**
- Owner active + GPU load >50% → toadStool will NOT dispatch GPU workloads
- Owner active + CPU load >70% → toadStool will NOT dispatch CPU workloads
- Owner inactive (10 min idle) → full dispatch capacity available
- `max_guest_load` configurable per gate (flockGate default: 50%)
- Owner can set `dispatch_accepting = false` at any time (instant opt-out)

### Priority Hierarchy

```
1. Owner foreground tasks (gaming, development, browsing)    — ALWAYS wins
2. Owner background tasks (compilation, downloads)           — yields to owner fg
3. Family covalent dispatch (toadStool science workloads)    — yields to all owner
4. Ionic dispatch (ABG workloads via tunnel)                 — lowest priority
```

toadStool uses Linux cgroups (`cpu.weight`, `memory.max`) and GPU compute
priority (Vulkan priority hints) to enforce this hierarchy without manual
intervention.

---

## Gate Roles in Family Context

| Gate | Owner | Primary Use | HPC Contribution |
|------|-------|-------------|------------------|
| **northGate** | Family member A | Gaming (RTX 5090) | GPU compute when idle (nights, work hours) |
| **ironGate** | Developer | Agentic development | Composition validation, ABG-facing gateway |
| **southGate** | Family member B | Gaming + general | 128 GB RAM for fuzz farm, float 3090 workloads |
| **strandGate** | Dedicated HPC | Always compute | 100% science (no personal owner) |
| **biomeGate** | Dedicated HPC | Always compute | 100% science (HBM2 bench) |
| **westGate** | Dedicated storage | Always-on NAS | 76 TB ZFS (low CPU, always serving) |
| **flockGate** | Family member C | Personal (remote household) | Overnight batches, latency-tolerant |
| **eastGate** | Family member D | General use | Neuromorphic (Akida) when idle |
| **kinGate** | Staging | Dev/test | Staging primals before production |

**Key insight**: strandGate, biomeGate, and westGate have no "personal owner" —
they are dedicated compute/storage nodes. They accept workloads 24/7. The
family-owned gates (northGate, southGate, flockGate, eastGate) contribute
spare cycles. The aggregate is larger than any one machine alone.

---

## Multi-Household (flockGate Pattern)

flockGate demonstrates that covalent bonding works across households:

```
┌─────────────────────┐          WAN           ┌─────────────────────┐
│  Primary Household   │ ←─── TCP fallback ───→ │  Remote Household   │
│  (LAN: 10G backbone) │     (Songbird BirdSong) │  (LAN: 1G)          │
│                       │                         │                      │
│  ironGate             │                         │  flockGate           │
│  northGate            │                         │  (i9-13900K,         │
│  strandGate           │                         │   RTX 3070 Ti)       │
│  biomeGate            │                         │                      │
│  westGate             │                         └──────────────────────┘
│  southGate            │
│  eastGate             │
└───────────────────────┘
```

**Same trust, different physics:**
- Same family seed → full covalent trust
- Songbird BirdSong over TCP (not UDP multicast — WAN doesn't support it)
- Higher latency (~20-40ms WAN vs <1ms LAN)
- Only latency-tolerant workloads dispatched (overnight batches, parameter sweeps)
- Owner sets `max_guest_load_default = 50` (half capacity reserved for personal use)

**Scaling the pattern**: Any family member with a PC can enroll. A cousin's
gaming laptop, a sibling's workstation, a parent's media PC — if they share
the family seed and run NUCLEUS, they're part of the mesh. The organism grows
by adding cells.

---

## Privacy Model

Covalent bonding means full trust, but trust is not surveillance:

| What's visible | To whom | Why |
|---------------|---------|-----|
| Gate capabilities (CPU, GPU, RAM) | All covalent peers | toadStool needs this for dispatch |
| Available compute (% free) | All covalent peers | Dispatch targeting |
| Workload TOML spec (domain, requirements) | Dispatching gate | Decides WHERE to send |
| Workload binary/shader | Executing gate | Runs the actual computation |
| Owner's local files | Nobody else | Never shared, never accessed |
| Owner's browsing/activity | Nobody else | `owner_active` is a boolean, not keylogging |
| Results of dispatched workloads | Dispatching gate (via braid) | Provenance chain |

**What's encrypted in transit** (BTSP everywhere):
- Workload payloads between gates
- Results streaming back from execution
- Control messages (dispatch, status, cancel)

**What's NOT encrypted** (LAN covalent, full trust):
- Songbird BirdSong discovery announcements (needed for mesh formation)
- Gate capability advertisements (needed for dispatch)
- Health/liveness probes

---

## The Economics

A family of 4 with gaming PCs already owns ~$8,000 in compute hardware:

| Component | Traditional Use | HPC Contribution |
|-----------|----------------|------------------|
| 4x GPUs (RTX 3070-5090) | Gaming 4h/day | 20h/day idle → science dispatch |
| ~300 GB RAM aggregate | Web browsers | 200+ GB available during off-hours |
| ~60 CPU cores | General compute | 40+ cores available overnight |

**Vs institutional HPC**: A university allocation of 10,000 GPU-hours/year
costs $50,000+. A family mesh with 4 GPUs contributing 20h/day provides
29,200 GPU-hours/year at zero marginal compute cost (electricity is heating
— see `GPU_HEAT_RECOVERY_OPERATIONS.md`).

The family mesh is not replacing Frontier. It's replacing the grant
application process for small-to-medium science.

---

## Operational Requirements

### For 10G LAN Gates

- MTU 9000 (jumbo frames) on dedicated covalent VLAN
- `nucleus_config.sh` defines inter-gate IPs (static, .toml-referenced)
- Songbird auto-discovers 10G peers → adjusts payload sizing
- NestGate replication uses full 10G bandwidth for cross-gate blob sync
- toadStool workload data transfer negligible for <1 GB payloads at 10G

### For WAN Gates (flockGate)

- Songbird TCP fallback (BirdSong over encrypted WAN connection)
- Only workloads tagged `latency_tolerant = true` dispatched to WAN gates
- Data transfer budgeted (don't saturate residential upload with GB payloads)
- Owner opt-in/opt-out via simple CLI: `nucleus dispatch --pause` / `--resume`

### For toadStool Dispatch

- `compute.capabilities` returns gate hardware + `[science]` from TOML
- `compute.dispatch.submit` checks target gate `available_compute` before sending
- Dispatch respects `owner_priority` field (per-gate configurable)
- Failed dispatch (gate went busy) → automatic re-route to next-best gate
- Multi-gate pipeline DAGs coordinated by dispatching gate (ironGate typically)

---

## The Living Pattern

A family HPC is not a cluster you build once and operate. It's an organism
that grows as family members upgrade their PCs, add new machines, or bring
new households into the covalent mesh.

northGate was an AI/LLM development machine. It evolved into a gaming PC.
Its RTX 5090 contributes to lattice QCD when the owner isn't gaming. That's
not a compromise — it's ecology. The hardware serves two purposes, and both
are valuable.

The 10G backbone ($50 in cables) transforms isolated personal computers into
a unified supercomputer during off-hours. Nobody gives up their PC. Everyone
gains access to the collective.

---

*11 gates, 4 households reachable, 1 family seed. Each member owns their
machine. The collective owns the compute.*
