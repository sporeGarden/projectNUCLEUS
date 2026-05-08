# Revalidation Report — Phase 60 Upstream Absorption

**Date**: 2026-05-08
**From**: projectNUCLEUS (ironGate)
**For**: primalSpring, primal teams, wateringHole
**Binaries**: plasmidBin v2026.05.08 (all 13 primals checksum-verified via `sync.sh`)

---

## Summary

Full 5-layer security validation run after absorbing primalSpring Phase 60
binaries. All upstream security gaps from our Multi-User Hardening pentest
are confirmed resolved or adopted.

**Result**: 263 PASS, 0 FAIL, 2 WARN, 12 INFO

Previous run (pre-Phase 60): 250 PASS, 0 FAIL, 5 WARN

---

## Resolved Gaps

| Gap | Status | Evidence |
|-----|--------|----------|
| **DF-1** (5 primals bind 0.0.0.0) | **RESOLVED** | All 14 primal ports verified on `127.0.0.1`. PG-55 default binding shipped in Phase 60. DF-1 workaround code removed from deploy.sh, security_validation.sh, darkforest_pentest.sh |
| **JH-0** (unauthenticated RPC) | **ADOPTED** | MethodGate `auth.mode` returns `"permissive"` on beardog:9100. All 4 ABG tiers detect permissive mode. Logged but allowed — set `NUCLEUS_AUTH_MODE=enforced` for scope-based rejection |
| **JH-1** (no caller identity) | **RESOLVED** | BearDog `identity.create` → `auth.issue_ionic` → `auth.verify_ionic`. Ed25519-signed ionic tokens with scope, expiry, JTI |
| **JH-2** (no resource limits) | **RESOLVED** | biomeOS v3.48 enforces `timeout_ms`. ToadStool S232 enforces `mem_mb`, `cpu_cores`, `max_timeout_ms` |
| **JH-3** (full restart required) | **RESOLVED** | biomeOS `composition.reload` — hot-swap single primal without restart |
| **JH-4** (session UX) | **RESOLVED** | BearDog `auth.issue_session` — purpose-based presets |
| **JH-5** (no audit log) | **Phase 2 COMPLETE** | skunkBat `security.audit_log` — 1024-event ring buffer, 7 event kinds, cursor-based polling |
| **GAP-11** (barraCuda methods) | **CLOSED** | 18/18 methods (71 total JSON-RPC methods, 389 registered across 82 domains) |

## Remaining Findings

### KNOWN_GAP (1)

| ID | Finding | Severity | Mitigation | Resolution Path |
|----|---------|----------|------------|-----------------|
| KG-1 | `nestgate storage.list` accessible without auth | Medium | MethodGate permissive mode logs the call. UFW + iptables block remote. Only localhost processes can reach it | Activate `NUCLEUS_AUTH_MODE=enforced` — MethodGate will reject unauthenticated calls |

### DARK_FOREST (6)

These are information leaks or attack surface findings that reveal presence
or capability to an attacker. None are exploitable given current controls.

| ID | Finding | Threat Actor | Mitigation |
|----|---------|-------------|------------|
| DF-V1 | Hub API leaks version 5.4.5 at `/hub/api/` | External | JH-10 upstream gap — built-in handler, cannot override in JupyterHub config. Block at tunnel level |
| DF-S1 | Compute user can enumerate system services (jupyterhub, cloudflared visible via `systemctl`) | Compute | Informational only — does not grant control. Consider `--no-pager` restriction in PAM |
| DF-P1 | Reviewer can execute `python3` directly | Reviewer | Terminals are disabled (no JupyterLab terminal). Only exploitable if a terminal bypass exists. NoKernelManager blocks Jupyter kernel creation |
| DF-F1 | Oversized cookie returns HTTP 431 | External | Correct behavior (RFC 6585). No information leak beyond HTTP stack presence |
| DF-F2 | Null byte username reflected in 403 error page | External | Authentication correctly rejected. CSP prevents XSS. Input reflection is cosmetic — Tornado default error template |
| DF-F3 | Hub API version disclosure at /hub/api/ | Tunnel | Same as DF-V1. Mitigated by rate limiting at Cloudflare edge |

### WARN (2)

| Finding | Source | Risk |
|---------|--------|------|
| 3 non-localhost listeners (systemd-resolve, rustdesk, sweetgrass ephemeral) | OS-level | systemd-resolve is expected. rustdesk is LAN management tool. sweetgrass ephemeral port was transient (not present post-restart). None are primal ports |
| `nestgate storage.list` accessible without auth | Primal API | Same as KG-1. Logs under MethodGate permissive |

---

## Validation by Layer

| Layer | Tests | PASS | FAIL | Notes |
|-------|-------|------|------|-------|
| 1: Below (OS/Network) | Port scan, firewall, permissions | 19 | 0 | All 14 primal ports on 127.0.0.1 |
| 2: At (Primal APIs) | Auth probes, fuzzing, method enum, BTSP | 14 | 0 | 4 primals survived 7 fuzz payloads each |
| 3: Above (Application) | JupyterHub headers, auth, traversal | 10 | 0 | All security headers present, traversal blocked |
| 4: Tiers (ABG Enforcement) | 44 OS + 18 API assertions | 62+ | 0 | All 4 tiers enforce correctly |
| 5: Dark Forest | Pen test + protocol fuzz | 158+ | 0 | 6 DARK_FOREST findings (informational) |

---

## plasmidBin Deployment Gap (Found and Fixed)

During absorption, discovered that `git pull` on plasmidBin updates
`checksums.toml` but leaves stale local binaries in place. This means
the deploy pipeline thinks it has Phase 60 binaries but is actually
running Phase 59 code.

**Fix**: Created `plasmidBin/sync.sh` — validates local binaries against
`checksums.toml` using `b3sum`, then re-fetches stale/missing ones from
GitHub Releases. Also fixed a bug in `fetch.sh --force` that didn't
delete the old binary before re-downloading.

This is a plasmidBin evolution item, not a primal gap.

---

## Enforced Mode Results (2026-05-08)

After switching `NUCLEUS_AUTH_MODE` from `permissive` to `enforced`:

**Result**: 265 PASS, 0 FAIL, 0 KNOWN_GAP, 1 WARN, 5 DARK_FOREST

### What Changed

| Finding | Permissive | Enforced |
|---------|-----------|----------|
| `nestgate storage.list` unauthenticated | KNOWN_GAP (logged) | **PASS** (rejected -32001) |
| `nestgate storage.store_blob` | KNOWN_GAP | **PASS** (rejected) |
| `loamspine spine.status` | KNOWN_GAP | **PASS** (rejected) |
| `beardog crypto.list_keys` | KNOWN_GAP | **PASS** (rejected) |
| `biomeos composition.list` | KNOWN_GAP | **PASS** (rejected) |
| `toadstool job.list` | KNOWN_GAP | **PASS** (rejected -32601) |
| MethodGate status (all tiers) | "permissive — logged" | **"enforced — blocked"** |

### Ionic Token Flow Validated

```
identity.create → DID (did:key:z6Mkk...)
auth.issue_session(purpose="jupyterhub") → scoped token (scope: ["crypto.*","health.*","capabilities.*","identity.*","auth.verify_ionic"])
auth.verify_ionic(token) → valid: true, scope_ok: true, Ed25519 signature verified
capabilities.list + _bearer_token → OK (in scope)
crypto.list_keys + _bearer_token → -32601 (past gate, method not found — correct)
storage.list on nestgate + beardog token → -32001 (cross-primal: token not verifiable — JH-11)
```

### New Gaps Found

| ID | Finding | Severity | Owner |
|----|---------|----------|-------|
| **JH-11** | Cross-primal token federation: beardog-issued tokens not verifiable by other primals | Medium | primalSpring / biomeOS team |
| **DF-2** | toadstool reads `TOADSTOOL_AUTH_MODE=enforced` but reports `permissive` via `auth.mode` | Low | toadstool team |
| **DF-3** | songbird, squirrel, petaltongue don't expose `auth.mode` on TCP | Info | Each primal team |

**JH-11 context**: Each primal's MethodGate validates independently. biomeOS composition forwarding (`_resource_envelope`) is the intended cross-primal auth path. For NUCLEUS deployment, this means a user who issues a token from beardog can authenticate to beardog methods, but calling nestgate/loamspine/etc. requires either per-primal token issuance or biomeOS-mediated forwarding.

---

## Next Steps

1. **JH-5 cross-primal forwarding**: skunkBat → rhizoCrypt DAG + sweetGrass braids (next evolution cycle)
2. **JH-11 resolution**: biomeOS composition forwarding with `_resource_envelope` carrying auth context across primals
3. **DF-2 fix**: toadstool team needs to map `TOADSTOOL_AUTH_MODE` env var to MethodGate enforcement
4. **JH-10 mitigation**: Cloudflare WAF rule to block `/hub/api/` version disclosure
5. **Composition parity tests**: primalSpring validation items 1-4

---

## Ecosystem Posture

| Metric | Value |
|--------|-------|
| Primals deployed | 13/13 |
| Primals on 127.0.0.1 | 14/14 ports |
| MethodGate enforced | 10/13 confirmed on TCP, 3 silent (UDS-only or BTSP-gated) |
| Ionic tokens | Live (BearDog Ed25519, verified, scope-checked) |
| Resource envelopes | Enforced (biomeOS + ToadStool) |
| Audit log | Live (skunkBat ring buffer) |
| Security validation | **265 PASS, 0 FAIL, 0 KNOWN_GAP** |
| Pen test assertions | 158+ adversarial probes, 0 exploitable |
| ABG tier enforcement | 62+ assertions, 4 tiers, 0 violations |
| plasmidBin sync | 13/13 checksum-verified |
| Cross-primal auth | JH-11 gap (deferred — biomeOS forwarding path) |
