# Security Validation — Five-Layer Penetration Testing

How projectNUCLEUS validates security posture below, at, and above the
primal layer. Every tunnel evolution step (from `TUNNEL_EVOLUTION.md`)
is tested here before and after replacement.

## Current State (2026-05-08)

**265 PASS, 0 FAIL, 0 KNOWN_GAP** — `deploy/security_validation.sh`

- **Five layers**: OS/network, primal APIs, application, ABG tier enforcement, dark forest (pentest + fuzz)
- **MethodGate enforced**: 10/13 primals confirmed via TCP. All unauthenticated RPC calls return `-32001`
- **All 14 primal ports on 127.0.0.1** (Phase 60 PG-55 default)
- **Ionic tokens live**: Ed25519-signed, scope-checked, expiry-verified
- **UFW active**, hidepid=2, iptables outbound DROP for ABG UIDs, DNS exfil closed
- See `validation/REVALIDATION_PHASE60_MAY08_2026.md` for full results

> The May 6 baselines below are preserved as fossil record — they document
> the initial security posture and the gap discovery process.

---

## Principle: Continuous Security Regression

Security is not a one-time audit. Every change to the composition,
tunnel, or access model triggers a security validation run. Results
are provenance-tracked (BLAKE3 → rhizoCrypt → loamSpine → sweetGrass)
just like science workloads.

skunkBat observes every security test run. Its metrics (`security.scan`,
`security.detect`, `security.metrics`) capture what the primals see
from the inside during external probing.

---

## Layer Model (originally 3, now 5)

> **Update**: Layers 4 (ABG tier enforcement) and 5 (dark forest pentest + protocol fuzz) 
> were added 2026-05-08. See `deploy/security_validation.sh` for the live implementation.

### Historical Three-Layer Model (May 6 baseline)

### Layer 1: Below the Primals (OS / Network)

What the operating system and network expose, independent of primal code.

| Test | What it checks | Severity |
|------|---------------|----------|
| Port exposure | All primal ports bound to 0.0.0.0 vs 127.0.0.1 | HIGH |
| JupyterHub binding | Port 8000 should be localhost-only | HIGH |
| Firewall status | UFW/iptables active with deny-by-default | MEDIUM |
| Listening services | No unnecessary daemons (mysql, apache, etc.) | MEDIUM |
| File permissions | Family seed dir (700), cookie secret (600), DB | MEDIUM |
| SSH configuration | Key-only auth, no root login | MEDIUM |

**Current findings (2026-05-06 baseline):**
- Primals bind to `0.0.0.0` — this is how the Rust TCP listeners
  default. On a single-gate system with no external exposure (firewall
  + tunnel-only), this is acceptable. When the NucBox intake goes live,
  primals on the active gate must rebind to `127.0.0.1` or the USB-C subnet.
- JupyterHub correctly binds to `127.0.0.1:8000`
- Family seed directory has mode `700`
- JupyterHub cookie secret has mode `600`
- JupyterHub SQLite DB is `644` — should be `600`
- UFW needs activation

**Action items:**
1. Activate UFW with deny-by-default, allow SSH and tunnel only
2. Fix `jupyterhub.sqlite` permissions to `600`
3. When NucBox goes live, bind primals to `127.0.0.1` or `10.99.0.0/30`

### Layer 2: At the Primal Layer (API Security)

Security of the primal JSON-RPC interfaces themselves.

| Test | What it checks | Severity |
|------|---------------|----------|
| Unauthenticated access | Sensitive methods reject without BTSP | HIGH |
| Input fuzzing | Malformed JSON-RPC doesn't crash primals | HIGH |
| Method enumeration | No hidden admin/debug/shell methods | HIGH |
| BTSP enforcement | Plaintext connections rejected on BTSP ports | HIGH |
| Path traversal | No file system access via method names | MEDIUM |
| Payload size | Large payloads don't cause OOM | MEDIUM |

**Current findings (2026-05-06 baseline):**
- All 4 fuzzed primals (beardog, toadstool, nestgate, skunkbat) survived
  all 7 malformed payloads without crash. Rust's type system and
  serde_json deserialization provide strong input validation by default.
- All 3 probed primals reject all 7 suspicious method names
  (admin.shutdown, system.exec, debug.dump, etc.)
- sweetGrass rejects plaintext connections (BTSP enforced)
- rhizoCrypt rejects plaintext on tarpc port (9601)
- NestGate's `storage.list` is accessible without auth — this is
  expected for content-addressed storage (keys are opaque hashes),
  but should gain BTSP scoping in Phase 2b
- sweetGrass `braid.list` correctly rejects unauthenticated requests

**What this means for skunkBat:**
The fuzz payloads and enumeration probes are exactly the kind of
traffic skunkBat should learn to baseline and detect. Future runs
should see skunkBat's `threats_detected` counter increment when
it observes the enumeration patterns. This is training data.

### Layer 3: Above the Primals (Application Security)

JupyterHub, Cloudflare Tunnel, and the web application layer.

| Test | What it checks | Severity |
|------|---------------|----------|
| Security headers | X-Frame-Options, CSP, HSTS, X-Content-Type | MEDIUM |
| Auth enforcement | API endpoints reject unauthenticated requests | HIGH |
| Path traversal | ../../../etc/passwd blocked | HIGH |
| Open redirect | next= parameter doesn't redirect externally | MEDIUM |
| TLS configuration | Modern TLS (1.2+), valid certificate | HIGH |
| Server disclosure | Version header not exposed | LOW |

**Current findings (2026-05-06 baseline):**
- JupyterHub returns `403` for unauthenticated API access — correct
- Path traversal attempts return `302` (redirect to login) — correct
- JupyterHub sends `content-security-policy: frame-ancestors 'none'` — good
- Missing: `X-Frame-Options`, `X-Content-Type-Options`, `X-XSS-Protection`
- Server discloses `TornadoServer/6.5.5` — minor information leak
- Cloudflare tunnel uses TLS 1.3 with valid certificate
- Open redirect via `?next=//evil.com` returns 200 — needs investigation

**Action items:**
1. Add security headers to JupyterHub (Tornado config or reverse proxy)
2. Investigate open redirect on login `next` parameter
3. Suppress server version header if possible

---

## skunkBat Integration

skunkBat has four security methods relevant to penetration testing:

| Method | Role in pen testing |
|--------|-------------------|
| `security.scan` | Baseline topology snapshot before testing |
| `security.detect` | Real-time detection of anomalous patterns |
| `security.respond` | Automated defensive response to threats |
| `security.metrics` | Post-test summary (threats, quarantines, alerts) |

### Current State (v0.2.0-dev)

skunkBat is running and responsive. During the initial baseline scan:
- Threats detected: 0
- Connections quarantined: 0
- Alerts sent: 0
- Scans performed: 0

This is expected — skunkBat needs training data to establish baselines.
The security validation pipeline provides exactly this. Future runs
should show increasing detection capability as skunkBat learns the
difference between normal primal traffic and probe/fuzz traffic.

### Evolution Path

```
Now:      security_validation.sh probes → skunkBat observes passively
Phase 2b: skunkBat detects fuzz patterns → alerts to BearDog
Phase 3:  skunkBat auto-quarantines → defensive response tested
Phase 4:  skunkBat feeds into sweetGrass → security events provenance-tracked
```

### What skunkBat Needs (upstream feedback)

1. **Baseline learning** — run security_validation.sh repeatedly so
   skunkBat can distinguish normal health checks from probe patterns
2. **BearDog integration** — BTSP rejection events should flow to skunkBat
3. **Network awareness** — skunkBat should detect port scans and
   enumeration attempts against the mesh
4. **Provenance** — security events should be DAG-tracked in rhizoCrypt

---

## Validation Schedule

| Trigger | Layer | Scope |
|---------|-------|-------|
| Every tunnel evolution step | All | Full three-layer scan |
| New primal added to composition | At | API fuzzing + enumeration |
| New ABG user added | Above | Auth + tier enforcement |
| After deploy.sh --composition | Below + At | Port binding + health |
| Weekly (automated, future) | All | Regression baseline |

---

## Running the Pipeline

```bash
# Full three-layer scan (localhost)
bash deploy/security_validation.sh --layer all

# Test only primal APIs
bash deploy/security_validation.sh --layer at

# Include tunnel TLS validation
bash deploy/security_validation.sh --layer all --tunnel-url https://your-tunnel.trycloudflare.com

# From another machine on the LAN (test external exposure)
bash deploy/security_validation.sh --layer below --target 192.168.1.238
```

Results are written to `validation/security-YYYYMMDD-HHMMSS/` with:
- `security.log` — full test output
- `listening_ports.txt` — socket inventory
- `hub_headers.txt` — HTTP response headers
- `skunkbat_metrics.json` — post-scan defense metrics
- `skunkbat_detections.json` — detected anomalies
- `SECURITY_RESULTS.md` — summary report

---

## Relationship to benchScale

`infra/benchScale/` provides:
- 5 parity scenarios for sovereignty validation
- 3 pentest scripts for security testing
- Baseline comparison framework for external dependency replacement

benchScale topologies model **multi-node** security scenarios (untrusted
external node probing a defended mesh). `security_validation.sh` tests
**single-gate** security posture. Both are needed:

```
security_validation.sh  →  single gate, production posture
benchScale topologies   →  multi-node, simulated adversary
skunkBat showcase/      →  violation detection scenarios
```

---

## Baseline Results (2026-05-06)

### Before Fix

| Layer | PASS | FAIL | WARN | Key Finding |
|-------|------|------|------|-------------|
| Below | 4 | 13 | 2 | All 13 primals bind 0.0.0.0 |
| At | 11 | 0 | 1 | NestGate storage.list unauthenticated |
| Above | 2 | 0 | 5 | Missing security headers, server disclosure |
| **Total** | **17** | **13** | **8** | Port binding is the primary issue |

### After Fix (same day)

`deploy.sh` updated with `BIND_ADDRESS` (default `127.0.0.1`):

| Category | Before | After |
|----------|--------|-------|
| Ports on 0.0.0.0 | 13 | 6 |
| Ports on 127.0.0.1 | 2 | 9 |
| LAN-reachable primal ports | 13 | 6 |
| `jupyterhub.sqlite` perms | 644 | 600 |

7 primals fixed at composition level (had `--bind`/`--listen`/`--host` flags).
6 primals still on 0.0.0.0 — need upstream `--bind` flag (see
`validation/SECURITY_HANDBACK_MAY06_2026.md`).

Remaining exposed: songbird, toadstool, skunkbat, biomeos, sweetgrass, petaltongue.
