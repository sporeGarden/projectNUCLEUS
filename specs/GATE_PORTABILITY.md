# Gate Portability Protocol

*Version 0.1.0 — May 2026*

## Overview

Gate portability ensures the ABG compute surface (`lab.primals.eco`) can migrate between physical gates without downtime for the static observer layer. Compute follows the active gate; rendered content persists independently.

## Architecture: Three Availability Tiers

```
┌─────────────────────────────────────────────────────────┐
│  Always On (no gate dependency)                         │
│  ├── primals.eco          GitHub Pages (sporePrint)      │
│  └── .pappusCast/html_export/   Static HTML renders     │
├─────────────────────────────────────────────────────────┤
│  Gate-Portable (follows active gate)                    │
│  ├── lab.primals.eco/voila/*   Rendered notebooks       │
│  ├── lab.primals.eco/hub/*     JupyterHub               │
│  └── pappusCast daemon         Validation + propagation │
├─────────────────────────────────────────────────────────┤
│  Gate-Specific (stays with hardware)                    │
│  ├── Local disk state          /home/<user>/shared/abg  │
│  ├── GPU/HPC workloads         Bound to hardware        │
│  └── Primal processes          Run on local hardware    │
└─────────────────────────────────────────────────────────┘
```

## Static HTML Export

pappusCast's Heavy validation tier (every ~6 hours) produces static HTML renders of all public notebooks via `jupyter nbconvert --to html --no-input`. These live in `.pappusCast/html_export/` and mirror the directory structure of `public/`.

These renders serve as an always-on fallback: if the active gate goes down, the last exported HTML can be served from any static host (NestGate, GitHub Pages, S3).

### Export structure

```
.pappusCast/html_export/
├── commons/
│   ├── Getting-Started.html
│   └── wetspring-public/notebooks/
│       ├── 01-16s-pipeline-validation.html
│       ├── 02-benchmark-python-vs-rust.html
│       └── 03-paper-reproductions.html
├── showcase/
│   ├── validation-dashboard.html
│   └── wetspring-validation-viz.html
├── data/
│   └── explorer.html
└── validation/
    └── darkforest-viewer.html
```

## Gate Switch Protocol

### Prerequisites

| Requirement | Description |
|---|---|
| SSH key access | Passwordless SSH to target gate |
| cloudflared | Installed and accessible on target |
| ABG_SHARED | Directory available on target (rsync or shared mount) |
| deploy.sh | Present on target for automated service startup |
| Tunnel credentials | `.cloudflared/` directory with tunnel config and JSON key |

### Execution phases

`deploy/gate_switch.sh <target-gate> [--dry-run]`

1. **Pre-flight** — Verify SSH, cloudflared, deploy.sh availability on target
2. **Static export** — Run pappusCast full sync + HTML export (preserves always-on layer)
3. **Stop local** — Gracefully stop jupyterhub, voila-public, pappusCast, voila-redirect
4. **Sync workspace** — rsync ABG_SHARED to target, excluding ephemeral state
5. **Deploy remote** — Run deploy.sh on target to start all services
6. **Transfer tunnel** — Sync cloudflared credentials, start tunnel on target, stop locally
7. **Remote sync** — Trigger pappusCast full sync on target to populate public/
8. **Verify** — Check lab.primals.eco responds 200

### What transfers

- ABG shared workspace (notebooks, data, configs)
- Cloudflare tunnel credentials
- Service composition (via deploy.sh)

### What does NOT transfer

- pappusCast state (`.pappusCast/`) — rebuilt on first run
- User environments (`envs/`, `wheelhouse/`) — rebuilt per gate
- Primal processes — restarted by deploy.sh on target hardware

## Cloudflare Routing

The Cloudflare tunnel and Access policies are gate-independent. They route by domain, not by backend IP.

```
primals.eco             → GitHub Pages (always on, no gate)
lab.primals.eco         → Cloudflare Tunnel → active gate
lab.primals.eco/hub/*   → Cloudflare Access → JupyterHub (authenticated)
lab.primals.eco/voila/* → Voila on active gate (public)
```

Switching gates means:
1. Stop cloudflared on old gate
2. Start cloudflared on new gate with the same tunnel ID
3. Cloudflare routes to the new backend within seconds

No DNS changes, no Access policy changes, no certificate changes.

## Bonding Layer Isolation

External observers never see:
- Which gate hosts the compute (gate names scrubbed from all public content)
- The covalent LAN topology (IP addresses, Cat6e layout)
- Primal ports or discovery hierarchy
- The number or names of other gates

This is enforced by:
- Voila source stripping (no Python code visible)
- Topology audit (gate names, absolute paths removed from markdown/outputs)
- Cloudflare tunnel (backend IP hidden)
- `hidepid=2` (primal processes hidden from users)

## Evolution Path

```
bash gate_switch.sh        (current)
  → tunnelKeeper gate-switch <target>   (Rust, absorbs gate_switch.sh)
    → primal capability: gate.switch    (neuralAPI routable)
```

## Failure Modes

| Failure | Impact | Mitigation |
|---|---|---|
| Target gate unreachable | Switch aborted at phase 1 | Static HTML still available |
| rsync interrupted | Partial workspace on target | Re-run gate_switch.sh |
| Tunnel fails to start on target | lab.primals.eco down | Static HTML fallback; restart tunnel manually |
| pappusCast sync fails on target | Stale public/ content | pappusCast daemon auto-syncs on next cycle |
| Both gates down | No live compute | Static HTML + sporePrint still serve content |
