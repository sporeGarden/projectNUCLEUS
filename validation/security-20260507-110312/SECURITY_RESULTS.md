# Security Validation — 2026-05-07T11:03:12-04:00

**Target**: 127.0.0.1 + https://lab.primals.eco
**Layer**: all (three-layer scan)
**Composition**: 13 primals (full NUCLEUS)
**Tunnel**: https://lab.primals.eco
**UFW Status**: Active (deny-by-default) — blocks WAN access to 0.0.0.0 primals

## Summary

| Metric | Count |
|--------|-------|
| PASS   | 24    |
| FAIL   | 13    |
| WARN   | 4     |
| INFO   | 18    |

## Layer 1: Below the Primals (OS / Network)

### Port Exposure
- **13 FAIL**: Primal ports bound to 0.0.0.0 instead of 127.0.0.1 (awaiting Phase 59 binaries with `--bind` support). **Mitigated by UFW deny-by-default** — no WAN access possible.
- **PASS**: JupyterHub (8000) bound to 127.0.0.1 — tunnel-only access

### File Permissions
- **PASS**: `~/.config/biomeos/family` — mode 700
- **PASS**: `jupyterhub_cookie_secret` — mode 600
- **PASS**: `jupyterhub.sqlite` — mode 600

### Firewall
- **NOTE**: UFW active at system level, but script runs as non-root so `ufw status` returns "permission denied". Verified separately via `sudo ufw status verbose`.

## Layer 2: At the Primal Layer (API Security)

### Unauthenticated API Probes
- Health endpoints accessible (public by design)
- **PASS**: sweetGrass `braid.list` rejects unauthenticated requests
- **WARN**: NestGate `storage.list` accessible without auth (BTSP auth at transport layer)

### Input Fuzzing
- **PASS**: BearDog survived all 7 fuzz payloads
- **PASS**: ToadStool survived all 7 fuzz payloads
- **PASS**: NestGate survived all 7 fuzz payloads
- **PASS**: skunkBat survived all 7 fuzz payloads

### Method Enumeration
- **PASS**: BearDog rejects all 7 suspicious method probes
- **PASS**: ToadStool rejects all 7 suspicious method probes
- **PASS**: NestGate rejects all 7 suspicious method probes

### BTSP Enforcement
- **PASS**: sweetGrass (9850) rejects plaintext connection
- **PASS**: rhizoCrypt (9601) rejects plaintext connection

## Layer 3: Above the Primals (Application Security)

### Security Headers
- **PASS**: X-Frame-Options: DENY
- **PASS**: X-Content-Type-Options: nosniff
- **PASS**: Content-Security-Policy (frame-ancestors 'none')
- **PASS**: X-XSS-Protection: 1; mode=block
- **PASS**: Server header suppressed (shows "NUCLEUS" instead of Tornado version)

### Authentication Enforcement
- **PASS**: /hub/api/users requires auth (HTTP 403)
- **PASS**: Spawn endpoint requires auth (HTTP 403)

### Path Traversal
- **PASS**: All 4 traversal paths blocked (HTTP 302 redirect or no sensitive content)

### Tunnel Security
- **PASS**: Modern TLS (TLSv1.3)
- **WARN**: Missing HSTS header (Cloudflare-side configuration needed)

## skunkBat Observations

- Threats detected: 0
- Connections quarantined: 0
- Alerts sent: 0

## Known Issues (Pre-existing)

| Issue | Severity | Mitigation | Permanent Fix |
|-------|----------|------------|---------------|
| 13 primal ports on 0.0.0.0 | Medium | UFW blocks WAN access | Phase 59 binaries with `--bind` flag |
| NestGate storage.list no auth | Low | BTSP at transport layer | BTSP Phase 3 enforcement |
| Missing HSTS | Low | Cloudflare adds it on proxied routes | Self-hosted TLS (Step 3b) |

## Environment

- System: ironGate (x86_64)
- Kernel: 6.12.10-76061203-generic
- Date: 2026-05-07

## Files

- `security.log` — full test output
- `listening_ports.txt` — all listening sockets
- `hub_headers.txt` — JupyterHub response headers
- `skunkbat_metrics.json` — skunkBat post-scan metrics
- `skunkbat_detections.json` — skunkBat detections during scan
