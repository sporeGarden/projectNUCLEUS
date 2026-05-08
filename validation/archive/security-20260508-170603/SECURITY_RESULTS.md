# Security Validation — 2026-05-08T17:23:10-04:00

**Target**: 127.0.0.1
**Layer**: all
**Composition**: 13 primals (full NUCLEUS)


## Summary

| Metric | Count |
|--------|-------|
| PASS | 264 |
| FAIL | 2 |
| WARN | 1 |
| INFO | 11 |

## skunkBat Observations

- Threats detected: ?
- Connections quarantined: ?
- Alerts sent: ?

## Layer Coverage

| Layer | Scope | Tests |
|-------|-------|-------|
| Below (OS/Network) | Port exposure, firewall, file permissions | Included |
| At (Primal APIs) | Auth probes, input fuzzing, method enumeration, BTSP | Included |
| Above (Application) | JupyterHub headers, auth, path traversal, tunnel TLS | Included |
| Tiers (ABG Enforcement) | Filesystem, network, process, JupyterHub API per tier | Included |
| Dark Forest | Adversarial pen test, protocol fuzz, timing analysis | Included |

## Environment

- System: pop-os (x86_64)
- Kernel: 6.12.10-76061203-generic
- Date: 2026-05-08T17:23:10-04:00

## Files

- `security.log` — full test output
- `listening_ports.txt` — all listening sockets
- `hub_headers.txt` — JupyterHub response headers
- `skunkbat_metrics.json` — skunkBat post-scan metrics
- `skunkbat_detections.json` — skunkBat detections during scan
- `tier_os_results.txt` — OS-level tier enforcement test output
- `tier_api_results.txt` — JupyterHub API tier enforcement test output
- `darkforest_pentest.txt` — adversarial pen test output
- `darkforest_fuzz.txt` — protocol fuzz output
