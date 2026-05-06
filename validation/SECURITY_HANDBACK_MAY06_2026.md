# Security Handback — Penetration Testing Results

**Date**: 2026-05-06
**From**: projectNUCLEUS (ironGate)
**For**: Primal teams, primalSpring, wateringHole
**Context**: Three-layer pen test on live 13-primal full NUCLEUS composition

---

## Summary

Ran automated penetration testing (below/at/above the primal layer) on the
live composition. Fixed what we could at the composition level. Remaining
issues require primal-level changes.

**Before fixes**: 13 primal ports on 0.0.0.0, reachable from LAN
**After fixes**: 7 ports rebound to 127.0.0.1, 6 ports still need upstream `--bind` flag

---

## Fixed at Composition Level (deploy.sh)

These primals accept a bind address flag. `deploy.sh` now passes
`BIND_ADDRESS` (default `127.0.0.1`, overridable via `NUCLEUS_BIND_ADDRESS`):

| Primal | Port | Flag Used | Result |
|--------|------|-----------|--------|
| BearDog | 9100 | `--listen $BIND:$PORT` | 127.0.0.1 |
| Squirrel | 9300 | `--bind $BIND` | 127.0.0.1 |
| NestGate | 9500 | `--bind $BIND` | 127.0.0.1 |
| rhizoCrypt | 9601/9602 | `--host $BIND` | 127.0.0.1 |
| loamSpine | 9700 | `--bind-address $BIND` | 127.0.0.1 |
| CoralReef | 9730 | `--rpc-bind $BIND:$PORT` | 127.0.0.1 (was already) |
| BarraCuda | 9740 | `--bind $BIND:$PORT` | 127.0.0.1 (env default) |
| sweetGrass | 9851 | `--http-address $BIND:$PORT` | 127.0.0.1 (HTTP endpoint) |

Also fixed: `jupyterhub.sqlite` permissions (644 → 600).

---

## Needs Primal-Level Fix (upstream)

These primals only accept `--port` and hardcode `0.0.0.0` as the bind address.
They need a `--bind` or `--host` flag added to their CLI:

### P1: Songbird — HTTP server binds to all interfaces

**Port**: 9200 (HTTP)
**Current**: `--port 9200` → binds `*:9200`
**Needed**: `--bind` or `--listen-address` flag for the HTTP server
**Risk**: HIGH when NucBox intake goes live — Songbird's HTTP health
endpoint and BirdSong UDP multicast would be reachable from WAN
**Note**: Songbird already has `--listen` but it's for the internal
IPC socket, not the HTTP server

### P2: ToadStool — TCP server binds to all interfaces

**Port**: 9400
**Current**: `--port 9400` → binds `0.0.0.0:9400`
**Needed**: `--bind` or `--host` flag
**Risk**: MEDIUM — ToadStool accepts workload submissions; LAN exposure
means any device on the subnet could submit compute workloads
**Pattern**: Follow BarraCuda's pattern: `--bind host:port` overrides `--port`

### P3: skunkBat — TCP server binds to all interfaces

**Port**: 9140
**Current**: `--port 9140` → binds `0.0.0.0:9140`
**Needed**: `--bind` or `--host` flag
**Risk**: LOW-MEDIUM — skunkBat is defensive/read-only, but exposing
security.scan/detect/metrics to the LAN leaks security posture info
**Irony**: The defense primal is the one with the security gap

### P4: biomeOS — Neural API binds to all interfaces

**Port**: 9800
**Current**: `--port 9800` → binds `0.0.0.0:9800`
**Needed**: `--bind` or `--host` flag
**Risk**: MEDIUM — Neural API is the orchestration layer; LAN exposure
allows capability discovery and potentially workload routing
**Note**: Has `--tcp-only` flag but no bind address control

### P5: sweetGrass — Main TCP binds to all interfaces

**Port**: 9850 (newline-delimited JSON-RPC)
**Current**: `--port 9850` → binds `0.0.0.0:9850`
**Needed**: The existing `--http-address` only controls the HTTP endpoint;
the main newline-delimited TCP listener needs its own `--bind` flag
**Risk**: MEDIUM — sweetGrass holds attribution braids and BTSP keys
**Note**: BTSP enforcement at transport level mitigates somewhat

### P6: petalTongue — TCP server binds to all interfaces

**Port**: 9900
**Current**: `--port 9900` → binds `0.0.0.0:9900`
**Needed**: `--bind` or `--host` flag
**Risk**: LOW — petalTongue serves UI, minimal sensitive data
**Pattern**: Follow BarraCuda's `--bind host:port` pattern

---

## Cross-Cutting Recommendation: UniBin `--bind` Standard

The root cause is that `--port` is defined in UniBin v1.1 as the universal
entry point, but it doesn't specify the bind address. Seven primals already
implement some form of bind address control, but with six different flag names:

| Flag | Primals using it |
|------|-----------------|
| `--listen host:port` | BearDog, Songbird (IPC only) |
| `--bind host:port` | NestGate, Squirrel, BarraCuda |
| `--bind-address host` | loamSpine |
| `--host host` | rhizoCrypt |
| `--rpc-bind host:port` | CoralReef |
| `--http-address host:port` | sweetGrass (HTTP only) |

**Proposal**: Add `--bind <host:port>` to the UniBin v1.1 standard as
the universal bind address flag. When provided, it overrides `--port`.
Default should be `127.0.0.1` (localhost-only), not `0.0.0.0`.

This is a **one-line change** in each primal's CLI parser and a
**two-line change** in the UniBin standard doc.

---

## Other Security Findings

### Composition Level (fixed or documented)

| Finding | Severity | Owner | Status |
|---------|----------|-------|--------|
| `jupyterhub.sqlite` mode 644 | MEDIUM | projectNUCLEUS | FIXED (now 600) |
| UFW inactive | MEDIUM | ironGate admin | Documented (needs sudo) |
| TornadoServer version in headers | LOW | JupyterHub config | Documented |
| Missing X-Frame-Options header | LOW | JupyterHub config | Documented |
| Missing X-Content-Type-Options | LOW | JupyterHub config | Documented |

### Primal Level (needs upstream evolution)

| Finding | Severity | Owner | Status |
|---------|----------|-------|--------|
| NestGate `storage.list` unauthenticated | MEDIUM | NestGate team | Needs BTSP scoping |
| 6 primals bind 0.0.0.0 | HIGH | Each primal team | Needs `--bind` flag |
| sweetGrass ephemeral ports on 0.0.0.0 | LOW | sweetGrass team | Two extra ports exposed |

### Positive Findings (upstream should know)

| Finding | Significance |
|---------|-------------|
| All 4 fuzzed primals survived 7 malformed payloads | Rust serde + type system = strong default |
| All 3 probed primals reject 7 hidden method names | No debug/admin backdoors |
| sweetGrass rejects plaintext (BTSP enforced) | Transport-layer security works |
| rhizoCrypt rejects plaintext on tarpc port | BTSP enforcement works |
| JupyterHub returns 403 for unauthed API calls | Auth works correctly |
| Path traversal blocked (302 redirect to login) | Tornado handles this correctly |

---

## skunkBat Observations

skunkBat was live during all pen testing. Metrics after scan:
- Threats detected: 0
- Connections quarantined: 0
- Alerts sent: 0

This is expected — skunkBat needs baseline data to distinguish normal
from anomalous. The pen test payloads are training data. Recommendation:
skunkBat should learn to detect the fuzz/enumeration patterns observed
during this test, and alert when similar patterns appear in production.

---

## Action Summary

| Action | Owner | Priority | Effort |
|--------|-------|----------|--------|
| Add `--bind` to UniBin v1.1 standard | wateringHole | HIGH | 1 day |
| Add `--bind` to 6 primals (P1-P6) | Each primal team | HIGH | 1 hour each |
| NestGate BTSP scoping for storage.list | NestGate team | MEDIUM | 1 day |
| Activate UFW on ironGate | ironGate admin | MEDIUM | 5 minutes |
| skunkBat baseline learning from pen test data | skunkBat team | MEDIUM | Evolution |
| JupyterHub security headers | projectNUCLEUS | LOW | 30 minutes |
