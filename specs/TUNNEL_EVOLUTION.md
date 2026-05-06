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

## Where We Are (2026-05-06)

### Infrastructure

| Component | Current State | External Dependency |
|-----------|--------------|---------------------|
| DNS | Cloudflare nameservers for `primals.eco` | Cloudflare |
| TLS termination | Cloudflare edge | Cloudflare |
| CDN / caching | Cloudflare CDN | Cloudflare |
| Static site | GitHub Pages (sporePrint repo, Zola) | GitHub |
| Tunnel to ironGate | **cloudflared quick tunnel validated** (p50: 270ms) | Cloudflare |
| JupyterHub | Running on ironGate:8000, externally accessible via tunnel | — |
| Primal composition | **13 primals** live on ironGate (dynamic discovery) | — |
| Provenance pipeline | Full 9-phase pipeline operational (run 2: 26 events) | — |

### Validated Results (Phase 1)

- 13 primals deployed via `deploy.sh --composition full --gate irongate`
- 235+ science checks pass across 11 wetSpring workloads
- Full provenance chain: BLAKE3 → rhizoCrypt DAG → loamSpine ledger → sweetGrass braid
- ed25519 witness on braid, PROV-O compliant, DID attribution
- JupyterHub with 3 kernels (Python, bioinfo, R)

### Hardware for Phase 2

| Node | Role | Link to ironGate |
|------|------|-----------------|
| GMKtec NucBox M6 (32 GB) | Intake / tunnel termination | USB-C ethernet (10.99.0.0/30) |
| ironGate (i9-14900K, 96 GB DDR5) | Compute + provenance | Direct (USB-C point-to-point) |

The USB-C ethernet link creates a physically isolated two-node covalent
bond. No switch, no LAN contamination, no accidental discovery of other
gates. Songbird BirdSong UDP multicast scopes to this link only.

---

## Where We Are Going

### Step 2a: Cloudflare Tunnel to JupyterHub

**Goal**: ABG member accesses JupyterHub via `primals.eco/compute`

```
Browser → primals.eco (Cloudflare DNS + CDN)
       → cloudflared on NucBox M6
       → USB-C ethernet → ironGate:8000 (JupyterHub)
```

**What to do**:
1. Install `cloudflared` on NucBox M6
2. Create Cloudflare Tunnel pointing to `10.99.0.2:8000`
3. Add `/compute` route in Cloudflare dashboard
4. sporePrint static site continues on GitHub Pages (`/`)
5. Validate: ABG member runs notebook, submits workload, gets provenance

**Metrics to capture** (Cloudflare dashboard):
- Request latency to `/compute` (p50, p99)
- Tunnel uptime
- Bandwidth consumed
- Error rates

**New external dependency**: `cloudflared` binary on NucBox M6 (outbound
connection to Cloudflare edge). No port forwarding needed.

### Step 2b: BTSP Authentication Inside CF Tunnel

**Goal**: Ionic capability tokens scope ABG access

```
Browser → primals.eco → cloudflared
       → BearDog BTSP handshake (inside tunnel)
       → JupyterHub (authenticated)
```

**What to do**:
1. BearDog issues ionic capability token to ABG member
2. Token presented during BTSP handshake at intake
3. Cloudflare tunnel carries BTSP-authenticated traffic
4. JupyterHub only accessible with valid token

**Validation target**: Token-scoped access works — ABG member can run
notebooks but cannot discover internal gates or access raw storage.

**External dependency unchanged**: Cloudflare edge still handles TLS.

### Step 3a: sporePrint Content to NestGate

**Goal**: Remove GitHub Pages dependency

```
Browser → primals.eco → Cloudflare CDN
       → cloudflared → NucBox M6
       → petalTongue (static content from NestGate)
```

**What to do**:
1. Move sporePrint Zola output to NestGate content-addressed store
2. petalTongue `web` mode serves content from NestGate
3. Update Cloudflare route: `/` → NucBox M6 (petalTongue) instead of GitHub Pages
4. Validate: same page load times, same content, Lighthouse score matches

**Metrics parity**: Cloudflare CDN cache hit ratio, TTFB, page load time.

**Dependency removed**: GitHub Pages.

### Step 3b: BTSP Replaces Cloudflare TLS

**Goal**: BearDog handles TLS termination

```
Browser → primals.eco (Cloudflare DNS-only, no proxy)
       → BTSP handshake with NucBox M6 (BearDog ChaCha20-Poly1305)
       → petalTongue + JupyterHub
```

**What to do**:
1. BearDog serves TLS on port 443 at the intake node
2. Cloudflare configuration changes from "proxy" to "DNS-only" (grey cloud)
3. Validate: BTSP handshake completes in < 200ms, no browser warnings

**Metrics parity**: TLS handshake time (CF baseline vs BTSP), error rates.

**Dependency removed**: Cloudflare TLS/CDN proxy.

**benchScale validation**: Penetration testing against BTSP endpoint.
Verify DDoS resilience without Cloudflare protection. Acceptable
tradeoff: residential IP is less resilient than CF edge, but the
security chain is now fully sovereign.

### Step 3c: Songbird NAT Replaces cloudflared

**Goal**: Remove `cloudflared` binary dependency

```
Browser → primals.eco (DNS-only)
       → Songbird NAT traversal (STUN + BearDog keys)
       → NucBox M6 intake → ironGate
```

**What to do**:
1. Songbird NAT traversal establishes direct connection
2. Self-hosted STUN relay on a NUC or VPS (eliminates Cloudflare tunnel)
3. Validate: connection reliability matches `cloudflared` under benchScale load

**Dependency removed**: Cloudflare Tunnel (`cloudflared`).

### Step 4: Sovereign DNS

**Goal**: Zero external dependencies in the full path

```
Browser → sovereign DNS resolution (or hardcoded IP / `.local`)
       → BTSP handshake → Songbird transport
       → petalTongue + JupyterHub + full NUCLEUS
```

**Dependency removed**: Cloudflare DNS. Fully sovereign.

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

| Dependency | Introduced | Primal Replacement | Status | Removed |
|-----------|-----------|-------------------|--------|---------|
| GitHub Pages | Day 0 | NestGate + petalTongue | Planned (Step 3a) | — |
| Cloudflare CDN | Day 0 | NestGate content-addressing | Planned (Step 3a) | — |
| Cloudflare TLS | Day 0 | BearDog BTSP Phase 3 | Planned (Step 3b) | — |
| Cloudflare Tunnel | Step 2a (validated 2026-05-06) | Songbird NAT traversal | Active — baseline captured (p50: 270ms) | — |
| Cloudflare DNS | Day 0 | Sovereign resolution | Planned (Step 4) | — |

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
