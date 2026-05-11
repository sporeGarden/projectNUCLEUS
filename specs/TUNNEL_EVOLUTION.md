# Tunnel Evolution — Systematic External Dependency Replacement

How projectNUCLEUS progressively replaces external infrastructure with
primal compositions. Each step uses the previous step's production
metrics as validation targets. Nothing is removed until the replacement
proves parity or superiority under real load.

---

## Principle: Calibrate, Replace, Validate

External services (Cloudflare, GitHub Pages, Tailscale) are not
compromises — they are **calibration instruments**. They provide
production-grade baselines:

- Latency (p50, p95, p99)
- Availability (uptime %)
- Throughput (requests/sec, MB/sec)
- Error rates (4xx, 5xx)
- Security (DDoS mitigation, TLS handshake time)

Each primal replacement must meet or exceed these baselines before the
external dependency is removed. `infra/benchScale` provides the load
generation and penetration testing framework for validation.

---

## Where We Are (2026-05-10)

### Cell Membrane Model

The infrastructure uses a **cell membrane architecture**: the public face
(primals.eco) lives permanently on GitHub Pages + Cloudflare CDN
(extracellular). The tunnel is a membrane carrying only inward-bound
traffic to sovereign compute. Inside the membrane, the gate has total
control.

This delineation produces clean data for skunkBat: every tunnel event is
an authenticated membrane crossing, not CDN noise. External/internal
traffic separation is structural, not policy-based.

### Infrastructure

| Component | Current State | Layer |
|-----------|--------------|-------|
| Public site | **primals.eco → GitHub Pages + Cloudflare CDN** (permanent) | Extracellular |
| DNS | Cloudflare nameservers, A records to GitHub Pages IPs | Extracellular |
| TLS termination | Cloudflare edge (public) / tunnel (membrane) | Both |
| Membrane channels | **lab.primals.eco, git.primals.eco → tunnel** | Membrane |
| Tunnel replicas | Named tunnel `nucleus-lab` with multi-gate replicas | Membrane |
| Membrane watchdog | `gate_watchdog.sh` — logs state transitions every 30s | Membrane |
| JupyterHub | System service on primary gate:8000, PAM auth | Intracellular |
| Primal composition | **13 primals** on primary gate, systemd user service | Intracellular |
| Provenance pipeline | Full 9-phase pipeline operational | Intracellular |
| Firewall | **UFW active**: deny-by-default | Intracellular |
| benchScale | 5 scenarios, 3 pentest tools | Intracellular |

### Validated Results (Phase 2a — Current)

- **Cell membrane architecture**: public face on CDN, tunnel as membrane only
- 13 primals deployed via `deploy.sh --composition full --gate <active-gate>`
- Named Cloudflare tunnel with multi-gate replicas (membrane failover)
- `gate_provision.sh` provisions replica gates via SSH
- `gate_watchdog.sh` membrane health monitor (30s interval, logs for skunkBat)
- `tunnelKeeper v0.2.0` health command reports replica count, unique origins, edge colos
- JupyterHub as system service with PAM auth and tiered ABG access
- UFW deny-by-default (all primals bind 127.0.0.1 since Phase 60)
- Five-layer pen test: **265 PASS, 0 FAIL, 0 KNOWN_GAP** (Phase 60 enforced mode)
- BTSP enforcement validated (sweetGrass/rhizoCrypt reject plaintext)
- Full provenance chain: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid

### Hardware for Phase 2

| Node | Role | Link to the active gate |
|------|------|-----------------|
| GMKtec NucBox M6 (32 GB) | Intake / tunnel termination | USB-C ethernet (10.99.0.0/30) |
| The active gate (i9-14900K, 96 GB DDR5) | Compute + provenance | Direct (USB-C point-to-point) |

The USB-C ethernet link creates a physically isolated two-node covalent
bond. No switch, no LAN contamination, no accidental discovery of other
gates. Songbird BirdSong UDP multicast scopes to this link only.

---

## Where We Are Going

### Step 2a: Cloudflare Tunnel to JupyterHub ✅ COMPLETE

**Goal**: ABG member accesses JupyterHub via `lab.primals.eco`

**Status**: Live and validated as of 2026-05-06.

```
Browser → lab.primals.eco (Cloudflare DNS + CDN)
       → cloudflared named tunnel (nucleus-lab) on the active gate
       → 127.0.0.1:8000 (JupyterHub system service)
```

**Completed**:
1. Named tunnel `nucleus-lab` created with `cloudflared` on the active gate
2. Three systemd services: NUCLEUS primals, JupyterHub (system), cloudflared tunnel
3. PAM authentication with tiered ABG access (abg-compute, abg-admin, abg-pi)
4. Local admin account blocked from tunnel login via `post_auth_hook`
5. UFW deny-by-default, security headers applied
6. Pen test: 24 PASS, 0 critical failures

**Baselines captured**:
- Hourly cron → `validation/baselines/daily/tunnel_metrics_YYYY-MM-DD.csv`
- Metrics: DNS lookup, TCP connect, TLS handshake, TTFB, total, throughput
- Initial samples: TTFB p50 ~112ms, TLS p50 ~68ms via Cloudflare edge
- Summarization script: `validation/baselines/summarize_baselines.sh` (run after 7 days)

### Step 2b: BTSP Authentication Inside CF Tunnel

**Goal**: Ionic capability tokens scope ABG access, replacing PAM passwords.

**Prerequisites discovered during JupyterHub deployment** (see
`validation/JUPYTERHUB_PATTERNS_HANDBACK.md`):

- **JH-0 (Critical)**: Before ionic tokens can scope access, every primal's
  RPC dispatcher must check capability tokens before executing methods. The
  JupyterHub deployment revealed that convention-based access control
  (`NUCLEUS_READONLY=1` env var) provides zero enforcement — reviewer-tier
  users could execute arbitrary code and call any primal RPC on localhost.
  The dispatcher check pattern must be defined and implemented across all
  primals before Step 2b can provide real security.
- **JH-1 (High)**: BearDog needs `identity.create` for canonical identity
  management. PAM username case-sensitivity bugs (`ABGreviewer` vs
  `abgreviewer`) showed that identity normalization must be handled at the
  identity layer, not delegated to OS-level PAM.
- **JH-4 (Medium)**: Token delivery UX for non-technical ABG members. The
  manual `chpasswd` workflow exposed the real credential lifecycle that
  ionic tokens must support end-to-end.

```
Browser → lab.primals.eco → cloudflared tunnel
       → BearDog BTSP handshake (inside tunnel, on the active gate)
       → JupyterHub (token-authenticated, scoped to ABG tier)
```

**What to build**:
1. **RPC dispatcher capability check** (all primals) — before `auth.issue_ionic`
   can be meaningful, every primal must enforce tokens at the dispatch layer.
   Pattern: token → allowlist check → execute or reject. Without this, tokens
   are advisory, not enforcing. See JH-0 in patterns handback.
2. **BearDog ionic token issuer** — `beardog.auth.issue_ionic` method that
   creates time-limited capability tokens scoped to ABG tier (compute, admin, pi).
   Token encodes: user identity, tier, expiry, allowed primal methods.
   Also requires `beardog.identity.create` for canonical identity management
   (DID-based, case-insensitive lookup). See JH-1 in patterns handback.
3. **JupyterHub BTSP authenticator plugin** — custom `Authenticator` class that
   validates BTSP ionic tokens instead of PAM passwords. Located at
   `deploy/jupyterhub_btsp_auth.py`. Must implement:
   - `authenticate()` — validate token via `beardog.auth.verify_ionic` RPC
   - `pre_spawn_hook()` — inject token-scoped environment into notebook server
   - Fallback to PAM during shadow run (dual-auth mode)
4. **Token distribution** — BearDog CLI command or web form to issue tokens to
   ABG members. Token delivered via secure channel (not email). Must be usable
   by non-technical researchers. See JH-4 in patterns handback.

**Implementation detail**:
```python
# jupyterhub_btsp_auth.py (skeleton)
class BTSPAuthenticator(Authenticator):
    async def authenticate(self, handler, data):
        token = data.get('password', '')  # token submitted in password field
        resp = await beardog_rpc('auth.verify_ionic', {'token': token})
        if resp.get('valid') and resp['tier'] in ALLOWED_TIERS:
            return {'name': resp['identity'], 'auth_state': {'tier': resp['tier']}}
        return None
```

**Shadow-run protocol** (7 days minimum):
1. Enable dual-auth: PAM and BTSP both accepted simultaneously
2. Log which auth method each login uses
3. Compare: auth success rates, latency overhead of BTSP vs PAM
4. Monitor BearDog token issuance/revocation
5. Criteria to proceed: BTSP auth success rate ≥ 99.9%, latency overhead < 50ms

**benchScale validation**:
- `pentest/fuzz_jsonrpc.py --all-primals` to ensure BearDog token endpoint
  rejects malformed tokens, expired tokens, and cross-tier escalation
- `scenarios/full_stack_load.sh` to verify no auth bottleneck under 2x peak

**Validation target**: Token-scoped access works — tamison (abg-compute) can
run notebooks but cannot call `nestgate.storage.list` or `beardog.auth.issue_ionic`.

**External dependency unchanged**: Cloudflare edge still handles TLS.

### Step 3a: sporePrint Content to NestGate

**Goal**: Replace GitHub Pages with sovereign content serving — primals.eco served from NestGate + petalTongue.

**Cell membrane context (2026-05-10)**: In the current cell membrane architecture,
`primals.eco` lives on GitHub Pages (extracellular layer). Step 3a replaces this
with sovereign content serving, moving primals.eco from extracellular to membrane.
The CDN remains as a cache/fallback until parity is confirmed.

```
Browser → primals.eco (Cloudflare DNS + CDN proxy)
       → cloudflared tunnel → the active gate
       → petalTongue web mode → NestGate content-addressed store
```

**What to build**:
1. **NestGate content ingestion pipeline** — script that builds sporePrint
   with Zola, then pushes rendered HTML/CSS/JS to NestGate via
   `nestgate.content.put` RPC. Each file stored by its BLAKE3 hash.
   To be created at `deploy/publish_sporeprint.sh`.
   ```bash
   # Build and publish flow
   cd infra/sporePrint && zola build
   for file in public/**/*; do
       nestgate_rpc content.put --path "$file" --data "$(base64 < "$file")"
   done
   ```
2. **petalTongue web server mode** — petalTongue already has a web serving
   capability. Configure it to resolve URL paths to NestGate content hashes.
   Routing config (to be created) at `deploy/petaltongue_web.toml`:
   ```toml
   [web]
   listen = "127.0.0.1:9901"
   backend = "nestgate"
   root_collection = "sporeprint-latest"
   cache_ttl_secs = 3600
   ```
3. **Cloudflare route update** — add ingress rule in cloudflared config:
   ```yaml
   ingress:
     - hostname: primals.eco
       service: http://127.0.0.1:9901  # petalTongue web
     - hostname: lab.primals.eco
       service: http://127.0.0.1:8000  # JupyterHub
     - service: http_status:404
   ```
4. **CI integration** — sporePrint GitHub Action (or local hook) triggers
   `publish_sporeprint.sh` on push to main. GitHub Pages disabled once
   parity confirmed.

**Shadow-run protocol** (7 days minimum):
1. Both paths active: GitHub Pages at `primals.eco`, NestGate at
   `staging.primals.eco` (temporary Cloudflare route)
2. benchScale runs `scenarios/nestgate_content_parity.sh` hourly
3. Metrics compared: TTFB, total load time, content hash parity
4. Lighthouse scores compared side-by-side (automated via Lighthouse CI)
5. Criteria to proceed: TTFB within 10% of GitHub Pages, 100% content parity,
   Lighthouse score ≥ GitHub Pages score

**benchScale validation**:
- `scenarios/nestgate_content_parity.sh` — automated TTFB/content comparison
- `scenarios/full_stack_load.sh --target hub` at 2x peak, verifying petalTongue
  serves content without degradation while JupyterHub is under load

**Dependency removed**: GitHub Pages. Cloudflare CDN still proxies but content
originates from NestGate on the active gate.

### Step 3b: BTSP Replaces Cloudflare TLS

**Goal**: BearDog handles TLS termination — Cloudflare becomes DNS-only.

```
Browser → primals.eco (Cloudflare DNS-only, grey cloud)
       → BearDog TLS termination on the active gate:443
       → ChaCha20-Poly1305 AEAD channel
       → petalTongue (content) + JupyterHub (compute)
```

**What to build**:
1. **BearDog TLS listener** — BearDog accepts TLS connections on port 443
   with a standard X.509 certificate (for browser compatibility) layered
   over BTSP's ChaCha20-Poly1305. Implementation requires:
   - ACME client for Let's Encrypt certificate (certbot or built-in ACME)
   - Certificate auto-renewal (cron or BearDog-managed)
   - SNI routing: `primals.eco` → petalTongue, `lab.primals.eco` → JupyterHub
   - BTSP-aware clients get ChaCha20-Poly1305 directly (no X.509 overhead)
2. **Cloudflare DNS-only mode** — change `primals.eco` A record from proxied
   (orange cloud) to DNS-only (grey cloud). This removes Cloudflare's TLS
   termination and CDN caching.
3. **Rate limiting** — without Cloudflare's DDoS protection, BearDog must
   implement its own rate limiting:
   - Connection rate: max 100 new TLS handshakes/sec per source IP
   - Request rate: max 50 req/sec per authenticated session
   - Blackhole: auto-block IPs exceeding 10x rate limit for 1 hour
4. **Fallback route** — if BearDog TLS goes down, Cloudflare can be
   re-enabled (orange cloud) within minutes via API or dashboard.

**Shadow-run protocol** (7 days minimum):
1. BearDog TLS on port 8443 (non-standard), Cloudflare still on 443
2. `scenarios/btsp_tls_parity.sh` runs hourly against both endpoints
3. Metrics compared: TLS handshake time, TTFB, error rate, certificate chain
4. External pen test: `pentest/tunnel_probe.sh --url https://primals.eco:8443`
5. DDoS simulation: `scenarios/full_stack_load.sh --multiplier 5` against BearDog
6. Criteria to proceed: TLS handshake p95 ≤ CF baseline p95, zero certificate
   errors, rate limiter correctly blocks synthetic DDoS

**benchScale validation**:
- `scenarios/btsp_tls_parity.sh` — head-to-head TLS comparison
- `pentest/tunnel_probe.sh` — external attack surface against BearDog TLS
- `pentest/fuzz_jsonrpc.py` — verify BearDog TLS doesn't introduce new attack vectors

**Known tradeoff**: Residential IP is less resilient than CF edge network for
volumetric DDoS. Accepted: NUCLEUS serves ABG researchers, not high-traffic
public services. BearDog rate limiting + Dark Forest pattern sufficient.

**Dependency removed**: Cloudflare TLS/CDN proxy.

### Step 3c: Songbird NAT Replaces cloudflared

**Goal**: Remove `cloudflared` binary dependency — direct browser path to the active gate.

```
Browser → primals.eco (DNS-only, A record to public IP)
       → BearDog TLS on the active gate:443
       → Songbird NAT traversal (if behind NAT/CGNAT)
       → petalTongue + JupyterHub
```

**What to build**:
1. **Songbird STUN client** — Songbird maintains a UDP punch-through to a
   known STUN relay, keeping the NAT mapping alive. When a browser connects:
   - BearDog TLS accepts on port 443 (already running from Step 3b)
   - If direct connection succeeds (port forwarded or UPnP), Songbird not needed
   - If behind CGNAT, Songbird negotiates TURN relay as fallback
2. **Self-hosted STUN/TURN relay** — small VPS or second NUC at a different
   location. Requirements:
   - Public IP with ports 3478 (STUN) and 443 (TURN/TLS relay)
   - BearDog key-authenticated relay (no anonymous relay abuse)
   - Estimated cost: $5/month VPS (Hetzner, OVH)
   - Relay only carries encrypted BTSP traffic — no content inspection
3. **Connection fallback chain**:
   ```
   Try 1: Direct TCP to the active gate public IP:443
   Try 2: Songbird UDP punch-through via STUN
   Try 3: TURN relay via VPS
   Try 4: (emergency) Re-enable cloudflared tunnel
   ```
4. **Dynamic DNS** — if the active gate's IP changes (residential ISP), Songbird
   updates the DNS A record via Cloudflare API (DNS-only mode) or a sovereign
   DNS update mechanism.

**Shadow-run protocol** (7 days minimum):
1. Songbird NAT active alongside cloudflared tunnel
2. Route 50% of traffic through each path (DNS round-robin or client-side)
3. `scenarios/songbird_nat_parity.sh` compares both paths hourly
4. Metrics: connection establishment time, reliability, throughput
5. Test NAT re-establishment after: ISP IP change, router reboot, 24h uptime
6. Criteria to proceed: connection reliability ≥ 99.5%, establishment time
   p95 ≤ cloudflared baseline p95

**benchScale validation**:
- `scenarios/songbird_nat_parity.sh` — head-to-head with cloudflared
- `scenarios/full_stack_load.sh --multiplier 2` through Songbird path
- Verify TURN relay handles graceful fallback under load

**Dependency removed**: Cloudflare Tunnel (`cloudflared`). Cloudflare DNS remains.

### Step 4: Sovereign DNS

**Goal**: Zero external dependencies in the full path — complete sovereignty.

```
Browser → sovereign DNS resolution
       → BearDog TLS (ChaCha20-Poly1305 + X.509)
       → Songbird transport (direct or TURN relay)
       → petalTongue + JupyterHub + full NUCLEUS
```

**Options** (in order of preference):

1. **Self-hosted authoritative DNS** — run a lightweight authoritative DNS
   server (e.g., `knot-dns` or a custom primal) on the STUN/TURN VPS.
   Transfer `primals.eco` nameserver records from Cloudflare to self-hosted.
   - Pro: Complete control, no external dependencies
   - Con: Requires VPS with stable IP, DNS expertise, DNSSEC management
   - Implementation: `knot-dns` + DNSSEC + monitoring via skunkBat

2. **DNS-over-HTTPS (DoH) with BTSP** — serve DNS responses over BTSP
   channels for known clients (ABG members with ecosystem tooling).
   - Pro: Encrypted DNS, no plaintext DNS queries
   - Con: Only works for BTSP-aware clients, not standard browsers

3. **Hybrid** — self-hosted authoritative DNS for public resolution +
   BTSP DoH for ecosystem clients. Standard browsers use DNS, BTSP clients
   bypass DNS entirely (hardcoded IP or mDNS discovery).

**What to build** (Option 3 — Hybrid):
1. **Authoritative DNS on VPS** — `knot-dns` serving `primals.eco` zone.
   Records: A (gate public IP), AAAA (if available), MX (future), CAA (Let's Encrypt).
   DNSSEC signed. Monitoring via skunkBat health check.
2. **Registrar NS transfer** — update `primals.eco` registrar to point NS
   records at VPS IP instead of Cloudflare nameservers.
3. **BTSP direct resolution** — ecosystem clients (BTSP-aware) resolve
   `primals.eco` via BearDog's built-in service discovery. No DNS needed.
4. **Dynamic IP update** — Songbird notifies the authoritative DNS server
   when the active gate's public IP changes (replaces Cloudflare API call from 3c).

**Shadow-run protocol** (14 days minimum — DNS requires longer validation):
1. Both NS sets active: Cloudflare + self-hosted (secondary)
2. Validate DNS resolution from multiple geographic locations
3. Monitor: resolution time, DNSSEC validation, propagation after IP change
4. Criteria to proceed: resolution time p95 < 100ms from 5+ geolocations,
   zero DNSSEC failures, IP update propagation < 5 minutes

**Dependency removed**: Cloudflare DNS. **Fully sovereign.**

The only remaining external dependency is the domain registrar (for NS
delegation) and the VPS hosting the STUN relay + DNS server. The VPS is
a commodity resource — replaceable with any provider or a second NUC at
a friend's house.

---

## Replacement Validation Protocol

For each component replacement:

1. **Capture baseline** — record Cloudflare/GitHub metrics for the
   component being replaced (minimum 7 days of production data)

2. **Implement replacement** — build the primal-based alternative

3. **Shadow run** — run both paths simultaneously (Cloudflare + primal)
   for minimum 7 days. Compare metrics side-by-side.

4. **benchScale load test** — use `infra/benchScale` to generate
   synthetic load matching 2× peak observed traffic. Both paths must
   survive without degradation.

5. **benchScale pen test** — targeted security testing against the
   primal replacement. Verify no regression from Cloudflare's protection.

6. **Cut over** — route production traffic to primal path. Keep
   Cloudflare path as fallback for 7 days.

7. **Remove dependency** — disable the external service. Document the
   removal in this spec and update PHASES.md.

---

## Dependency Elimination Tracker

In the cell membrane model, dependencies are classified by layer:

| Dependency | Layer | Primal Replacement | Status |
|-----------|-------|-------------------|--------|
| GitHub Pages | Extracellular | NestGate + petalTongue | **Load-bearing** — primals.eco primary. Replace last. |
| Cloudflare CDN | Extracellular | NestGate content-addressing | **Load-bearing** — fronts GitHub Pages |
| Cloudflare Tunnel | Membrane | Songbird NAT traversal | **Active** — membrane channels for lab/git |
| Cloudflare TLS | Membrane | BearDog BTSP | Planned (Step 3b) — replace at membrane layer |
| Cloudflare DNS | Extracellular | Self-hosted authoritative | Planned (Step 4) — replace at extracellular layer |
| PAM passwords | Intracellular | BearDog ionic tokens | Planned (Step 2b) — ion channel selectivity |

Replacement order follows the membrane model: intracellular first (ionic
tokens), then membrane (Songbird transport, BearDog TLS), then
extracellular last (NestGate content, sovereign DNS). Each layer can be
replaced independently.

---

## Security Posture by Step

| Step | TLS | Auth | Tunnel | DDoS | Isolation |
|------|-----|------|--------|------|-----------|
| 2a | Cloudflare | PAM (JupyterHub) | cloudflared | Cloudflare | NucBox intake |
| 2b | Cloudflare | BTSP ionic token | cloudflared | Cloudflare | NucBox + BTSP |
| 3a | Cloudflare | BTSP ionic token | cloudflared | Cloudflare | NucBox + BTSP |
| 3b | BTSP (BearDog) | BTSP ionic token | cloudflared | BearDog rate-limit | NucBox + BTSP |
| 3c | BTSP | BTSP ionic token | Songbird NAT | BearDog + Dark Forest | NucBox + BTSP |
| 4 | BTSP | BTSP ionic token | Songbird NAT | Full sovereign | NucBox + BTSP |

At each step, the security surface area that projectNUCLEUS controls
grows. External dependencies shrink monotonically.

---

## Relationship to Upstream

- **primalSpring** defines the composition patterns and bonding models.
  projectNUCLEUS deploys and validates them on real hardware.
- **wateringHole** defines IPC, BTSP, and capability standards.
  projectNUCLEUS's tunnel evolution is a concrete application of these.
- **benchScale** provides load and security testing infrastructure.
  Each replacement step produces benchScale reports as evidence.
- **foundation** owns the institutional relationship narrative.
  projectNUCLEUS provides the proof that the infrastructure works.

Every successful Cloudflare replacement is evidence that primalSpring's
patterns work in production. Gaps discovered during replacement flow
back upstream via handoff documents.
