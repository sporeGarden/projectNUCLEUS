# Sovereignty Validation Protocol

**Last refreshed**: 2026-05-27 (Wave 56 — Nest Atomic LIVE, shadow 6/0/0, `--uds-only` VPS standard)

Master document governing the systematic replacement of every external
dependency with primal-native infrastructure. Each replacement follows a
rigorous validate-then-replace methodology: capture baseline, build
replacement, shadow run, prove parity, cut over.

This document ties together:
- [TUNNEL_EVOLUTION.md](TUNNEL_EVOLUTION.md) — the roadmap and implementation specs
- [SECURITY_VALIDATION.md](SECURITY_VALIDATION.md) — the three-layer security model
- `infra/benchScale/` — load generation and pen testing tools
- `validation/baselines/` — captured production metrics
- skunkBat — real-time security observability

---

## Guiding Principles

1. **Never remove before proving parity.** Every external dependency provides
   measurable value (latency, uptime, security). The primal replacement must
   demonstrably match or exceed that value before the external is removed.

2. **Shadow runs are mandatory.** Both paths (external + primal) run
   simultaneously for a minimum duration. Side-by-side comparison eliminates
   optimism bias.

3. **Rollback is always available.** Each step maintains a documented
   rollback procedure. The external dependency remains available for a
   minimum buffer period after cutover.

4. **Provenance tracks everything.** Every validation run, baseline capture,
   and parity report is tracked through the sweetGrass provenance pipeline.
   No validation result exists without a witness signature.

5. **Security never regresses.** The security posture matrix
   (TUNNEL_EVOLUTION.md §Security Posture by Step) must show monotonic
   improvement or equivalence at each step.

---

## External Dependencies — Current State

| # | Dependency | Current Provider | Primal Replacement | Step | Status |
|---|-----------|-----------------|-------------------|------|--------|
| 1 | PAM passwords | Linux PAM | BearDog ionic tokens | 2b | Specified |
| 2 | Static site hosting | GitHub Pages | NestGate + petalTongue | 3a | Specified |
| 3 | CDN / caching | Cloudflare CDN | NestGate content-addressing | 3a | Specified |
| 4 | TLS termination | Cloudflare edge | BearDog BTSP | 3b | Specified |
| 5 | NAT traversal / tunnel | cloudflared binary | Songbird NAT | 3c | Specified |
| 6 | DNS resolution | Cloudflare NS | Self-hosted auth DNS + BTSP DoH | 4 | Specified |

---

## Validation Framework

### For Each Replacement

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   BASELINE   │ →  │  SHADOW RUN  │ →  │   CUTOVER    │
│              │    │              │    │              │
│ Capture 7d   │    │ Both paths   │    │ Route to     │
│ of external  │    │ active for   │    │ primal path  │
│ metrics via  │    │ 7-14 days    │    │              │
│ benchScale   │    │              │    │ Keep fallback│
│              │    │ Compare via  │    │ for 7 days   │
│ Store in     │    │ benchScale   │    │              │
│ baselines/   │    │ scenarios    │    │ Then remove  │
│              │    │              │    │ external     │
└──────────────┘    └──────────────┘    └──────────────┘
```

### Baseline Phase

| Activity | Tool | Output |
|----------|------|--------|
| Capture latency (DNS, TCP, TLS, TTFB) | `baselines/capture_tunnel_metrics.sh` (cron) | `baselines/daily/tunnel_metrics_YYYY-MM-DD.csv` |
| Unified membrane telemetry | `nucleus-deploy telemetry` (cron) | `baselines/daily/membrane_telemetry_YYYY-MM-DD.csv` |
| Compute percentiles | `nucleus-deploy summary` | `baselines/membrane_7day.toml` (or legacy `cloudflare_tunnel_7day.toml`) |
| Run scenario-specific baseline | `scenarios/cloudflare_tunnel_baseline.sh` | `baselines/tunnel_baseline_*.toml` |
| Security scan | `pentest/three_layer_scan.sh` | `reports/security_*/benchscale_summary.toml` |
| Record in provenance | sweetGrass `braid.witness` | Braid URN in manifest |

Minimum baseline duration: **7 days** (DNS: 14 days).

### Shadow-Run Phase

Both the external service and primal replacement serve live traffic
simultaneously. benchScale scenarios run continuously (hourly cron) and
compare metrics in real time.

**Shadow-run infrastructure**:
1. Dual routing — DNS round-robin, client-side split, or separate hostnames
   (e.g., `staging.primals.eco` for primal path)
2. Continuous comparison — benchScale scenario runs hourly, writes to
   `reports/shadow_<step>_<date>.toml`
3. Alert on regression — if primal path metrics exceed baseline thresholds,
   log warning via skunkBat

**Shadow-run criteria matrix**:

| Metric | Threshold | Source |
|--------|-----------|--------|
| Latency (TTFB p95) | ≤ baseline p95 × 1.1 | benchScale scenario |
| Uptime | ≥ baseline uptime % | benchScale uptime log |
| TLS handshake p95 | ≤ baseline TLS p95 | benchScale scenario |
| Content parity | 100% hash match | `nestgate_content_parity.sh` |
| Auth success rate | ≥ 99.9% | JupyterHub auth logs |
| Security scan | 0 new FAIL findings | `three_layer_scan.sh` |
| Fuzz test | 0 crashes, 0 unexpected | `fuzz_jsonrpc.py` |

### Cutover Phase

1. Route 100% of production traffic to primal path
2. Monitor for 24 hours with heightened alerting
3. Keep external service as warm fallback for 7 days
4. Run final benchScale suite to confirm production metrics
5. Disable external service
6. Update `TUNNEL_EVOLUTION.md` dependency tracker
7. Witness the removal in provenance pipeline

---

## Step-by-Step Validation Plans

### Step 2b: BTSP Auth Replaces PAM

| Phase | Duration | benchScale Tool | Criteria |
|-------|----------|----------------|----------|
| Baseline | 7 days | Auth success rate from JupyterHub logs | PAM success rate ≥ 99% |
| Shadow | 7 days | Dual-auth (PAM + BTSP), compare login metrics | BTSP success ≥ 99.9%, latency < PAM + 50ms |
| Cutover | — | Disable PAM authenticator | No login failures |
| Rollback | Re-enable PAM authenticator in `jupyterhub_config.py` | < 5 min |

**Security validation**:
- `fuzz_jsonrpc.py` against BearDog token endpoint
- Cross-tier escalation test: `abg-compute` token must not access `abg-admin` methods
- Token expiry enforcement: expired tokens must be rejected
- Token replay: same token must not work from two different IPs simultaneously

**skunkBat integration**:
- Monitor `beardog.auth.verify_ionic` call rate during shadow run
- Alert on: auth failure spike, token issuance anomaly, cross-tier attempt

### Step 3a: NestGate Replaces GitHub Pages

| Phase | Duration | benchScale Tool | Criteria |
|-------|----------|----------------|----------|
| Baseline | 7 days | `cloudflare_tunnel_baseline.sh` against `primals.eco` | TTFB, total, throughput baselines |
| Shadow | 7 days | `nestgate_content_parity.sh` (hourly cron) | TTFB ≤ 110% of GH Pages, 100% content hash match |
| Cutover | — | Update Cloudflare route from GH Pages to cloudflared → petalTongue | Content served from NestGate |
| Rollback | Revert Cloudflare route to GH Pages | < 5 min |

**Security validation**:
- `tunnel_probe.sh` against petalTongue endpoint
- Verify NestGate content-addressing prevents content tampering
- Lighthouse audit: score must match or exceed GH Pages
- `three_layer_scan.sh` — no new findings after content migration

**skunkBat integration**:
- Monitor petalTongue request rate and error rate
- Alert on: 5xx responses, content hash mismatch, abnormal traffic patterns

### Step 3b: BTSP TLS Replaces Cloudflare TLS

| Phase | Duration | benchScale Tool | Criteria |
|-------|----------|----------------|----------|
| Baseline | 7 days | `cloudflare_tunnel_baseline.sh` (TLS metrics) | TLS handshake p50/p95/p99 |
| Shadow | 7 days | `btsp_tls_parity.sh` (hourly, port 8443 vs 443) | TLS p95 ≤ CF p95, zero cert errors |
| Load test | 1 day | `full_stack_load.sh --multiplier 5` against BearDog TLS | No degradation under 5x peak |
| Cutover | — | Cloudflare grey cloud (DNS-only) | BearDog serves TLS on 443 |
| Rollback | Cloudflare orange cloud (re-enable proxy) | < 5 min |

**Security validation**:
- `tunnel_probe.sh` against BearDog TLS endpoint
- TLS 1.0/1.1 rejection test (already in tunnel_probe.sh)
- Certificate chain validation from multiple clients
- Rate limiting effectiveness test: 10x burst must be blocked
- `fuzz_jsonrpc.py` through TLS channel — no bypass of TLS layer

**skunkBat integration**:
- Monitor TLS handshake rate, error rate, rate-limit trigger count
- Alert on: certificate expiry < 7 days, handshake failure rate > 1%

### Step 3c: Songbird NAT Replaces cloudflared

| Phase | Duration | benchScale Tool | Criteria |
|-------|----------|----------------|----------|
| Baseline | 7 days | Cloudflare tunnel metrics (already captured) | Connection reliability, establishment time |
| Shadow | 7 days | `songbird_nat_parity.sh` (hourly) | Reliability ≥ 99.5%, establishment p95 ≤ CF p95 |
| Load test | 1 day | `full_stack_load.sh --multiplier 2` through Songbird | No degradation |
| Cutover | — | Disable cloudflared service, Songbird primary | Direct browser path to the active gate |
| Rollback | Re-enable cloudflared systemd service | < 2 min |

**Security validation**:
- Verify STUN/TURN relay requires BearDog key authentication
- Test NAT punch-through recovery after ISP IP change
- Verify TURN relay carries only encrypted BTSP traffic
- `tunnel_probe.sh` against Songbird-served endpoint

**skunkBat integration**:
- Monitor Songbird connection establishment rate and failures
- Alert on: NAT mapping loss, TURN fallback activation, IP change events

### Step 4: Sovereign DNS Replaces Cloudflare NS

| Phase | Duration | benchScale Tool | Criteria |
|-------|----------|----------------|----------|
| Baseline | 14 days | DNS resolution time from 5+ geolocations | Resolution p95 < 100ms |
| Shadow | 14 days | Both NS sets active (CF primary, self-hosted secondary) | Resolution parity across geolocations |
| Cutover | — | Update registrar NS records to self-hosted | Self-hosted authoritative DNS |
| Rollback | Update registrar NS back to Cloudflare | ~24h (DNS propagation) |

**Security validation**:
- DNSSEC validation from external resolvers
- DNS amplification attack test (should not be exploitable)
- Zone transfer protection (AXFR must be denied)
- Verify dynamic IP update propagation time < 5 minutes

**skunkBat integration**:
- Monitor DNS query rate and resolution time
- Alert on: DNSSEC failure, zone transfer attempt, unusual query patterns

---

## Security Comparison Matrix

At each step, the security posture must improve or hold. This matrix tracks
what controls each step's security surface.

| Control | 2a (Current) | 2b | 3a | 3b | 3c | 4 |
|---------|-------------|----|----|----|----|---|
| **TLS** | Cloudflare | Cloudflare | Cloudflare | BTSP (BearDog) | BTSP | BTSP |
| **Auth** | PAM | BTSP ionic | BTSP ionic | BTSP ionic | BTSP ionic | BTSP ionic |
| **Tunnel** | cloudflared | cloudflared | cloudflared | cloudflared | Songbird NAT | Songbird NAT |
| **DNS** | Cloudflare | Cloudflare | Cloudflare | Cloudflare | Cloudflare | Sovereign |
| **DDoS** | Cloudflare | Cloudflare | Cloudflare | BearDog rate-limit | BearDog + Dark Forest | Full sovereign |
| **Content** | GitHub Pages | GitHub Pages | NestGate | NestGate | NestGate | NestGate |
| **Provenance** | sweetGrass | sweetGrass | sweetGrass | sweetGrass | sweetGrass | sweetGrass |
| **Observation** | skunkBat | skunkBat | skunkBat | skunkBat | skunkBat | skunkBat |
| **Sovereign %** | ~20% | ~35% | ~50% | ~70% | ~85% | 100% |

---

## benchScale Scenario Mapping

Which benchScale tool validates which replacement step.

| benchScale Tool | 2b | 3a | 3b | 3c | 4 |
|----------------|----|----|----|----|---|
| `cloudflare_tunnel_baseline.sh` | Baseline | Baseline | Baseline | Baseline | — |
| `btsp_tls_parity.sh` | — | — | **Primary** | — | — |
| `nestgate_content_parity.sh` | — | **Primary** | — | — | — |
| `songbird_nat_parity.sh` | — | — | — | **Primary** | — |
| `full_stack_load.sh` | Load test | Load test | Load test | Load test | Load test |
| `three_layer_scan.sh` | Security | Security | Security | Security | Security |
| `fuzz_jsonrpc.py` | Token fuzz | — | TLS fuzz | — | — |
| `tunnel_probe.sh` | — | Content probe | TLS probe | NAT probe | DNS probe |

---

## skunkBat Integration

skunkBat observes the entire validation pipeline and feeds metrics into
each step's go/no-go decision.

### Metrics Tracked

| Metric | RPC Method | Relevance |
|--------|-----------|-----------|
| Threats detected | `security.metrics` | Any nonzero → investigate before cutover |
| Connections quarantined | `security.metrics` | Validates rate limiting effectiveness |
| Alerts sent | `security.metrics` | Tracks anomaly detection |
| Auth failure rate | `security.detect` | Shadow-run auth comparison |
| Connection establishment | `security.detect` | Songbird reliability tracking |

### Per-Step skunkBat Checkpoints

Before approving any cutover:
1. Query `skunkBat.security.metrics` — threats must be 0 for the shadow-run period
2. Query `skunkBat.security.detect` — no anomalies flagged during load tests
3. Record metrics snapshot in provenance pipeline as a braid witness event

---

## Provenance Pipeline Integration

Every validation run produces artifacts that are tracked through sweetGrass.

### Artifact Types

| Artifact | Format | Provenance |
|----------|--------|------------|
| Baseline TOML | `baselines/membrane_7day.toml` (or `cloudflare_tunnel_7day.toml`) | BLAKE3 hash → rhizoCrypt DAG → braid witness |
| Parity report | `reports/btsp_tls_parity_*.toml` | BLAKE3 hash → rhizoCrypt DAG → braid witness |
| Security scan | `reports/security_*/benchscale_summary.toml` | BLAKE3 hash → rhizoCrypt DAG → braid witness |
| Fuzz report | `reports/fuzz_*.toml` | BLAKE3 hash → rhizoCrypt DAG → braid witness |
| Cutover record | `validation/cutover_<step>_<date>.toml` | BLAKE3 hash → rhizoCrypt DAG → braid witness |

### Provenance Flow

```
benchScale scenario run
  → output TOML artifact
  → BLAKE3 hash computed
  → rhizoCrypt DAG node created (links to previous validation)
  → loamSpine ledger entry (immutable append)
  → sweetGrass braid witnessed (ed25519 signature)
  → Artifact URN recorded in SOVEREIGNTY_VALIDATION_PROTOCOL.md
```

This creates an unbroken chain of evidence from the first Cloudflare
baseline to the final sovereign DNS cutover. Any future audit can trace
every validation decision back to its benchScale output and provenance
witness.

---

## Timeline Estimate

| Step | Earliest Start | Estimated Duration | Dependency |
|------|---------------|-------------------|------------|
| 2b (BTSP Auth) | After BearDog ionic token implementation | 3-4 weeks (build + 7d shadow) | BearDog `auth.issue_ionic` method |
| 3a (NestGate Content) | After petalTongue web mode ready | 2-3 weeks (build + 7d shadow) | petalTongue web config, NestGate content API |
| 3b (BTSP TLS) | After 3a validated | 3-4 weeks (build + 7d shadow + load test) | BearDog TLS listener, ACME integration |
| 3c (Songbird NAT) | After 3b validated | 4-6 weeks (VPS setup + build + 7d shadow) | Songbird STUN, TURN relay VPS |
| 4 (Sovereign DNS) | After 3c validated | 4-6 weeks (DNS setup + 14d shadow) | Authoritative DNS server, registrar NS change |

Total estimated time to full sovereignty: **4-6 months** from start of Step 2b.

Steps are sequential by design — each step's security validation depends on
the previous step being stable in production.

---

## Rollback Procedures

| Step | Rollback Action | Time to Restore | Automated? |
|------|----------------|-----------------|------------|
| 2b | Re-enable PAM authenticator in jupyterhub_config.py, restart | < 5 min | Manual |
| 3a | Revert Cloudflare route to GH Pages | < 5 min | Manual (Cloudflare dashboard) |
| 3b | Cloudflare orange cloud (re-enable proxy) | < 5 min | Manual (Cloudflare dashboard) |
| 3c | `systemctl --user start cloudflared-tunnel` | < 2 min | Manual |
| 4 | Update registrar NS back to Cloudflare | ~24h (propagation) | Manual (registrar) |

Step 4 has the longest rollback time due to DNS propagation. This is why
it requires a 14-day shadow run and is the last step.

---

## Document History

| Date | Change |
|------|--------|
| 2026-05-07 | Initial version. Phase 2a complete, Steps 2b-4 specified with shadow-run protocols. |
