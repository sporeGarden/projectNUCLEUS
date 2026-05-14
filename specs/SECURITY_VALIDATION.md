# Security Validation — Five-Layer Penetration Testing

How projectNUCLEUS validates security posture below, at, and above the
primal layer. Every tunnel evolution step (from `TUNNEL_EVOLUTION.md`)
is tested here before and after replacement.

## Current State (2026-05-14)

**267+ PASS, 0 FAIL, 0 KNOWN_GAP** — `deploy/security_validation.sh` (gate-local)

- **Five layers**: OS/network, primal APIs, application, ABG tier enforcement, dark forest (pentest + fuzz)
- **MethodGate enforced**: 13/13 primals confirmed via TCP. All unauthenticated RPC calls return `-32001`
- **All 14 primal ports on 127.0.0.1** (Phase 60 PG-55 default)
- **Ionic tokens live**: Ed25519-signed, scope-checked, expiry-verified
- **UFW active**, hidepid=2, iptables outbound DROP for ABG UIDs, DNS exfil closed
- **cellMembrane LIVE**: fieldMouse VPS on 157.230.3.183 (new external attack surface — Layer 6 below)
- **BearDog TLS shadow LIVE**: :8443 alongside Cloudflare :443 (not yet in darkforest)
- See `validation/REVALIDATION_PHASE60_MAY08_2026.md` for full results

> The May 6 baselines below are preserved as fossil record — they document
> the initial security posture and the gap discovery process.

---

## Layer 6: External Membrane (cellMembrane fieldMouse)

**Added 2026-05-14** — ownership transfer from primalSpring.

The cellMembrane is projectNUCLEUS's first deployment on **external substrate**
(DigitalOcean VPS). Unlike the gate, the substrate provider has theoretical root
access to the hardware. The Dark Forest principle applies: the provider is a
non-family observer. Everything sensitive must be encrypted at rest; the provider
sees only noise.

This is a fundamentally different security domain from Layers 1-5 (which assume
trusted hardware). External substrate requires its own threat model.

### What's Deployed

| Component | Value | Risk Profile |
|-----------|-------|-------------|
| VPS | `membrane-relay`, 157.230.3.183, Debian 12 x64, DigitalOcean nyc1 | Provider has hypervisor access |
| Channel 2 | Songbird TURN relay, UDP :3478 | Public relay endpoint |
| SSH | Key-only, ed25519, fail2ban active | Management surface |
| Firewall | UFW: 22/tcp + 3478/tcp+udp, default deny | Minimal surface |
| Composition | Relay only (Phase 0 — Tower not yet deployed) | Static, no biomeOS |

### Threat Model: External Substrate

| Threat | Severity | Mitigation (current) | Mitigation (future) |
|--------|----------|---------------------|-------------------|
| Provider reads disk | HIGH | TURN credentials in `/etc/songbird/relay-credentials` (plaintext) | BearDog Vault encrypts at rest (Phase 2) |
| Provider reads memory | HIGH | Relay processes only encrypted BTSP bytes (opaque to observer) | Unchanged — BTSP handles this by design |
| Provider snapshots VM | MEDIUM | No family seeds on VPS (relay mode, no BearDog yet) | BingoCube challenge on boot |
| Unauthorized relay abuse | HIGH | Credential-authenticated TURN (username + HMAC key) | BearDog BTSP handshake for relay access |
| SSH brute force | MEDIUM | fail2ban (5 attempts, 1h ban), key-only auth | Rotate keys periodically |
| TURN amplification/abuse | MEDIUM | Authenticated relay (no anonymous TURN) | SkunkBat defense audit (Tower Phase 1) |
| VPS compromise → pivot inward | HIGH | Relay is stateless — no inbound initiation. LAN gates connect outward only | BTSP mutual auth on all relay connections |
| Credential exposure in repo | HIGH | `cellMembrane` repo is private, `.gitignore` covers `*.age`, tokens, keys | BearDog `secrets.store` eliminates files |
| Provider-level network sniffing | LOW | All relayed traffic is BTSP-encrypted end-to-end | Unchanged |

### Hardening Verification (May 14, 2026)

Verified via SSH from ironGate:

| Check | Result | Evidence |
|-------|--------|----------|
| SSH access (key-only) | **PASS** | `Permission denied (publickey)` for password auth |
| songbird-relay active | **PASS** | `systemctl is-active songbird-relay` → `active` (PID 3110) |
| fail2ban sshd jail | **PASS** | 13 failed attempts caught, 0 banned (working correctly) |
| Firewall posture | **PASS** | UFW: 22/tcp + 3478/tcp+udp only, default deny |
| TURN relay (UDP) | **PASS** | `nc -z -u 157.230.3.183 3478` → reachable |
| journald persistence | **PASS** | `/var/log/journal/` exists |
| exim4 removed | **PASS** | No mail service in `ss -tlnp` |
| No unexpected listeners | **PASS** | Only sshd(:22), systemd-resolved(:53 localhost), songbird(:3478 UDP) |
| Disk/memory headroom | **PASS** | 14% disk, 105Mi/457Mi RAM — healthy margins |

### What the Provider Sees (Dark Forest Analysis)

| Layer | Provider observation | Actual content |
|-------|---------------------|---------------|
| Network | UDP packets to/from :3478 | BTSP-encrypted relay bytes — opaque |
| Binaries | `/opt/membrane/songbird` | Static musl ELF — public, published in plasmidBin |
| Config | `/etc/songbird/relay-credentials` | HMAC shared secret — **currently plaintext** |
| systemd | `songbird-relay.service` | Public template from plasmidBin |
| Firewall | UFW rules | Port list — standard TURN + SSH |
| Logs | journald entries | Operational metadata — connection counts, errors |

**Current gap**: TURN credentials are plaintext on disk. Provider can read them
via hypervisor access and relay traffic through the TURN server. Mitigation
roadmap: `share_credentials.sh` encrypts with `age` (Phase 1), BearDog Vault
encrypts at rest (Phase 2), BingoCube eliminates credential files entirely (Phase 3).

### darkforest Coverage Gap

darkforest v0.2.1 has **zero coverage** of the cellMembrane VPS. The validator
runs against `--host` (localhost) and uses TCP-only networking (`net.rs`).

**Proposed `membrane` suite (MEM-01 → MEM-10):**

| ID | Check | How | Severity |
|----|-------|-----|----------|
| MEM-01 | SSH password auth disabled | `ssh -o PreferredAuthentications=password` → rejected | HIGH |
| MEM-02 | fail2ban sshd jail active | `fail2ban-client status sshd` via SSH | HIGH |
| MEM-03 | UFW posture (22+3478 only, default deny) | `ufw status` via SSH | HIGH |
| MEM-04 | TURN relay reachable (UDP :3478) | UDP probe from gate | MEDIUM |
| MEM-05 | TURN rejects unauthenticated relay | STUN allocate without credentials → rejected | HIGH |
| MEM-06 | No unnecessary services (exim4, droplet-agent) | `systemctl list-units` via SSH | MEDIUM |
| MEM-07 | journald persistence configured | Check `/var/log/journal/` via SSH | LOW |
| MEM-08 | Credential file permissions | `stat /etc/songbird/relay-credentials` → 600/root | HIGH |
| MEM-09 | Songbird binary integrity | BLAKE3 hash vs plasmidBin checksum | MEDIUM |
| MEM-10 | No unexpected listening ports | `ss -tlnp` + `ss -ulnp` audit via SSH | HIGH |

**Additional darkforest gaps (non-membrane):**

| ID | Check | Current status |
|----|-------|---------------|
| BearDog TLS :8443 | Shadow running — not in primal port list or fuzz targets | Should add to fuzz suite |
| sporePrint local :8880 | Dev preview server — not tested | Low priority (localhost only) |
| SweetGrass BTSP :9851 | In `nucleus_config.sh` — not in darkforest primal list | Should verify or remove |

### Escalation Ladder (Security Posture by Phase)

```
Phase 0 (current): Relay only — TURN credentials plaintext on disk
  └── Dark Forest: provider sees credentials, but relayed traffic is BTSP-encrypted
  └── Risk: provider could abuse relay allocation. Impact: relay traffic interception
     (but content is BTSP-encrypted, so provider sees only encrypted bytes)
  └── Verification: ironGate SSH + manual checks ← WE ARE HERE

Phase 1: age-encrypted credentials on VPS
  └── share_credentials.sh encrypts all sensitive files
  └── Provider sees only age-encrypted blobs — noise
  └── Decryption requires ironGate's SSH ed25519 private key

Phase 2: Tower composition (BearDog + Songbird + SkunkBat)
  └── BearDog Vault encrypts credentials at rest
  └── SkunkBat monitors relay abuse patterns
  └── Family seed never stored plaintext — encrypted by BearDog on boot

Phase 3: BingoCube zero-knowledge access
  └── No credential files on VPS at all
  └── Access proven via progressive commitment reveal
  └── Provider has nothing to decrypt — only commitment proofs on disk

Phase 4: Full autonomy
  └── BearDog rotates credentials autonomously
  └── Operator only provisions initial FAMILY_SEED + domain registration
  └── biomeOS auto-provisions membrane channels
```

### Ownership Boundary (Security Responsibility)

| Domain | Owner | Security Contact |
|--------|-------|-----------------|
| VPS operations, uptime, credential rotation | **projectNUCLEUS / ironGate** | This team |
| Deployment tooling (`deploy_membrane.sh`, systemd units) | primalSpring | Upstream |
| Channel deployment decisions (DNS, TLS) | **projectNUCLEUS / ironGate** | This team |
| Upstream capability evolution (BearDog Vault, BingoCube) | primalSpring | Upstream |
| `sporeGarden/cellMembrane` repo (private) | **projectNUCLEUS / ironGate** | This team |

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
- TURN relay reachability probe (`songbird_nat_parity.sh`)
- Shadow run orchestrator tying all parity tests together

benchScale topologies model **multi-node** security scenarios (untrusted
external node probing a defended mesh). `security_validation.sh` tests
**single-gate** security posture. darkforest `--suite membrane` (when built)
tests external substrate posture. All three are needed:

```
security_validation.sh     →  single gate, production posture (Layers 1-5)
darkforest --suite membrane →  external substrate, VPS posture (Layer 6)
benchScale topologies      →  multi-node, simulated adversary
benchScale scenarios       →  sovereignty parity (TLS, NAT, DoT, content)
skunkBat showcase/         →  violation detection scenarios
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
