# Security Validation — 2026-05-07T21:35:30-04:00

**Target**: 127.0.0.1
**Layer**: tiers
**Composition**: 13 primals (full NUCLEUS)


## Summary

| Metric | Count |
|--------|-------|
| PASS | 44 |
| FAIL | 0 |
| WARN | 0 |
| INFO | 4 |

## skunkBat Observations

- Threats detected: 0
- Connections quarantined: 0
- Alerts sent: 0

## Layer Coverage

| Layer | Scope | Tests |
|-------|-------|-------|
| Below (OS/Network) | Port exposure, firewall, file permissions | Skipped |
| At (Primal APIs) | Auth probes, input fuzzing, method enumeration, BTSP | Skipped |
| Above (Application) | JupyterHub headers, auth, path traversal, tunnel TLS | Skipped |
| Tiers (ABG Enforcement) | Filesystem, network, process, JupyterHub API per tier | Included |

## Environment

- System: pop-os (x86_64)
- Kernel: 6.12.10-76061203-generic
- Date: 2026-05-07T21:35:30-04:00

## Files

- `security.log` — full test output
- `listening_ports.txt` — all listening sockets
- `hub_headers.txt` — JupyterHub response headers
- `skunkbat_metrics.json` — skunkBat post-scan metrics
- `skunkbat_detections.json` — skunkBat detections during scan
- `tier_os_results.txt` — OS-level tier enforcement test output
- `tier_api_results.txt` — JupyterHub API tier enforcement test output
