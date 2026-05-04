# Deploy Graphs

Curated subset of primalSpring/graphs/ for projectNUCLEUS deployment.
The canonical source for all 73+ graphs remains in primalSpring.

## Fragments (building blocks)

| File | Particle | Primals |
|------|----------|---------|
| `tower_atomic.toml` | Electron | BearDog + Songbird |
| `node_atomic.toml` | Proton | Tower + ToadStool + barraCuda + coralReef |
| `nest_atomic.toml` | Neutron | Tower + NestGate + provenance trio |
| `nucleus.toml` | Full atom | Tower + Node + Nest (9 primals) |

## Deployment Graphs

| File | Composition | Phase |
|------|-------------|-------|
| `node_atomic_compute.toml` | biomeOS + Node Atomic | Phase 1 |
| `nucleus_complete.toml` | Full NUCLEUS with bonding policy | Phase 4 |

## Bonding Patterns

| File | Bond Type | Use Case |
|------|-----------|----------|
| `basement_hpc_covalent.toml` | Covalent | LAN cluster — your machines |
| `friend_remote_covalent.toml` | Covalent + NAT | Friend's machine with tunnel |
| `ionic_capability_share.toml` | Ionic | Two-family metered sharing |

## Source

All graphs originate from:
`ecoPrimals/springs/primalSpring/graphs/`
