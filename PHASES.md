# Phase Architecture — projectNUCLEUS

Each phase builds on the previous. No phase is thrown away — Phase 1 covalent
bonding remains the foundation even when Phase 4 metallic federation is live.

---

## Phase 1: Covalent LAN HPC

**Status**: Validated (2026-05-04)
**Bonding**: Covalent (shared family seed, full trust)
**Composition**: Node Atomic (BearDog + Songbird + ToadStool + barraCuda + coralReef)

### What Works

- Node Atomic deployed via plasmidBin bootstrap + deploy.sh
- toadStool dispatches workloads from TOML specs (native runtime)
- wetSpring science validated through composition dispatch:
  - 16S Pipeline: 37/37 checks PASS
  - Diversity Metrics: 27/27 checks PASS
  - Gonzales CPU Parity: 43/43 checks PASS
  - Algae 16S: 26/26 checks PASS
  - Python benchmark baseline: SUCCESS (4.15s, all domains)
- JupyterHub running for notebook-style access
- Cat6e ethernet between cluster machines

### What projectNUCLEUS Provides

- `deploy.sh`: Automates seed creation, primal startup, health verification
- Gate manifest: Hardware inventory with atomic assignment
- Workload catalog: Validated TOML specs + templates for new workloads
- Validation logs: Reproducible evidence of composition correctness

### Known Gaps (from COMPOSITION_GAPS.md)

- Sandbox overrides working_dir to /tmp (absolute paths required)
- GPU build breakage (wgpu submit_and_poll → submit_and_map API drift)
- biomeOS not in live composition (single-gate dispatch only)
- No automatic results archival to nestGate
- Python runs as native subprocess (no toadStool introspection)

---

## Phase 2: Ionic Compute Sharing

**Status**: Designed (deploy graphs exist)
**Bonding**: Ionic (metered, scoped access)
**New Primals**: songBird cross-gate routing, BTSP Phase 2 auth

### Architecture

```
[External User] → [Intake Node / Tower Atomic]
                        ↓ LAN
                  [Compute Gate / Node Atomic]
                        ↓ toadStool dispatch
                  [workload execution]
```

- An expendable intake node runs Tower Atomic (tunnel termination, reverse proxy)
- songBird routes workloads from intake to compute gates
- BTSP Phase 2 authenticates external users (identity, not yet encrypted transport)
- JupyterHub proxied through NUC intake
- Ionic bonding: metered access, capability scoping, no family seed sharing

### Relevant Deploy Graphs

From primalSpring/graphs/:
- `bonding/ionic_capability_share.toml` — two-family ionic bridge
- `multi_node/friend_remote_covalent.toml` — friend compute sharing with NAT escalation
- `multi_node/idle_compute_federation.toml` — coordinator discovers idle peers

### What This Unlocks

- External collaborators submit workloads through the intake node
- The "lend a GPU to a friend" pattern — anyone with hardware can participate
- Egress, multi-user, and security patterns validated under real load
- songBird cross-gate dispatch exercised for the first time

---

## Phase 3: Self-Hosted sporePrint

**Status**: Conceptual (petalTongue + songBird required)
**Bonding**: Covalent core + public weak endpoint
**New Primals**: petalTongue (UI), NestGate (content storage), BTSP Phase 3

### Architecture

```
[Browser] → [BTSP tunnel / songBird NAT] → [Intake Node]
                                                ↓
                                     [petalTongue / sporePrint]
                                                ↓
                                     [NestGate / content store]
```

- petalTongue serves sporePrint content (replacing Zola static site generator)
- BTSP Phase 3: ChaCha20-Poly1305 encrypted transport for all connections
- songBird NAT traversal replaces Tailscale and WireGuard tunnels
- DNS: minimal Cloudflare during transition, then sovereign resolution
- Composition: `nest_viz.toml` (biomeOS + Tower + NestGate + petalTongue)

### What This Eliminates

- GitHub Pages dependency (content served from own hardware)
- Cloudflare dependency (BTSP handles TLS termination)
- Tailscale/WireGuard dependency (songBird handles tunneling)

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
