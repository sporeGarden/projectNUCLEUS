# Gate Portability Protocol

*Version 0.3.0 — May 2026*

## Overview

Gate portability ensures the ABG compute surface (`lab.primals.eco`) can migrate
between physical gates without downtime. The public face (`primals.eco`) lives
permanently on GitHub Pages + Cloudflare CDN — it has no gate dependency.

## Cell Membrane Architecture

The infrastructure follows a cell membrane model: the public internet
(extracellular) is served by external CDN with its own SLA. The tunnel
(membrane) provides gated channels inward to sovereign compute. Inside
(intracellular), the gate has total control.

```
┌─────────────────────────────────────────────────────────┐
│  Extracellular (CDN — no gate dependency)               │
│  └── primals.eco        GitHub Pages + Cloudflare CDN   │
│                         Always on. No tunnel. No gate.  │
├─────────────────────────────────────────────────────────┤
│  Membrane (tunnel channels — gated passage inward)      │
│  ├── lab.primals.eco         Observer + JupyterHub      │
│  ├── git.primals.eco         Forgejo (sovereign git)    │
│  └── (future)                BTSP ion channels          │
├─────────────────────────────────────────────────────────┤
│  Intracellular (sovereign compute — total control)      │
│  ├── JupyterHub              Compute for ABG tiers      │
│  ├── 13 primals              Sovereign services         │
│  ├── Observer surface        Pre-rendered notebooks     │
│  ├── Forgejo                 Sovereign git              │
│  ├── Primal processes        Run on local hardware      │
│  └── Data pipeline           Provenance, validation     │
└─────────────────────────────────────────────────────────┘
```

**Key property:** the extracellular surface has zero structural downtime.
If every gate is offline, `primals.eco` still serves. The membrane channels
(lab, git) are inherently gate-dependent — if no gate is running, there is
no sovereign compute to access.

## Tunnel Replicas (Membrane Resilience)

Multiple gates can run `cloudflared tunnel run nucleus-lab` as replicas.
Cloudflare maintains 4 connections per replica across 2+ data centers.
If a replica goes down, Cloudflare routes to a surviving one — sub-second
failover, invisible to users.

**Per-gate service stack:**
- `cloudflared-replica.service` — tunnel replica, `Restart=always`
- Observer + Forgejo services as appropriate

**Provisioning a new gate:**
```
deploy/gate_provision.sh <target-host>           # replica (observer + git)
deploy/gate_provision.sh <target-host> --full    # primary (all services)
```

**Membrane watchdog:**
`deploy/gate_watchdog.sh` runs as a systemd service, checking membrane
health every 30 seconds. It logs state transitions for skunkBat audit
consumption. It does NOT manage DNS — the public face is always on CDN.

## Static HTML Export

pappusCast renders all public notebooks to static HTML via
`jupyter nbconvert --execute --to html --no-input`. These live in
`.pappusCast/html_export/` with navigation and dark theme.
`observer_server.py` serves this directory on port 8866 — the same port
the Cloudflare tunnel routes `lab.primals.eco` to.

## Gate Switch Protocol (Primary Migration)

Switching the primary gate moves compute services (JupyterHub, observer,
pappusCast). Tunnel replicas on other gates continue serving membrane
channels independently.

### Execution phases

`deploy/gate_switch.sh <target-gate> [--dry-run]`

1. **Pre-flight** — Verify SSH, cloudflared, deploy.sh on target
2. **Static export** — Run pappusCast full sync + HTML export
3. **Stop local compute** — Stop jupyterhub, observer-static, pappusCast
4. **Sync workspace** — rsync ABG_SHARED to target
5. **Deploy remote** — Run deploy.sh on target
6. **Ensure full config** — Sync tunnel credentials and full ingress config
7. **Remote sync** — Trigger pappusCast on target
8. **Verify** — Check lab.primals.eco responds 200

## Routing

```
primals.eco             → GitHub Pages + Cloudflare CDN (extracellular)
lab.primals.eco         → Tunnel → observer-static on :8866
lab.primals.eco/hub/*   → Tunnel → JupyterHub (primary gate)
git.primals.eco         → Tunnel → Forgejo on :3000
```

The tunnel does not carry public traffic. It is purely a membrane —
the boundary between the external internet and sovereign compute.

## Bonding Layer Isolation

External observers never see:
- Which gate hosts the compute (gate names scrubbed from all public content)
- The covalent LAN topology (IP addresses, Cat6e layout)
- Primal ports or discovery hierarchy
- The number or names of other gates

## Evolution Path

```
Membrane (current):
  cloudflared tunnel        → ion channels for authenticated access
  GitHub Pages CDN          → extracellular public face
  gate_watchdog.sh          → membrane health logging

Future (sovereignty horizons):
  BTSP ionic tokens         → ion channel selectivity (who passes through)
  BearDog TLS               → membrane channel encryption (sovereign TLS)
  Songbird NAT              → membrane transport (replace cloudflared)
  NestGate content          → extracellular migration (replace GitHub Pages)
```

The membrane model means sovereignty horizons replace layers independently:
BTSP replaces Cloudflare Access at the membrane. BearDog replaces CF TLS.
Songbird replaces the tunnel transport. Each replacement happens at its
own layer without disturbing the others.

## Failure Modes

| Failure | Impact | Mitigation |
|---|---|---|
| One gate reboots | Membrane channels fail over to other replicas | Automatic (sub-second) |
| All gates down | Membrane down, compute inaccessible | Expected — no gate = no compute |
| GitHub Pages down | primals.eco down | Rare (~99.99% uptime). `sporeprint_dns.sh sovereign` as emergency |
| Cloudflare down | Everything down | Extremely rare. Affects most of the internet. |
| Primary gate switch | Brief membrane gap during transfer | gate_switch.sh handles gracefully |
