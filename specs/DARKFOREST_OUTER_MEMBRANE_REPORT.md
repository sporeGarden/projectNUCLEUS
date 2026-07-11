# darkforest v3.0 — Outer Membrane Execution Report

**Date**: 2026-07-11 | **Wave**: 136b | **Target**: primals.eco
**Scanner**: darkforest v0.3.0 (pure Rust, rustls + webpki-roots/ring)
**Source**: ironGate (residential, separate ISP from golgiBody VPS)
**Duration**: 13.8s | **Checks**: 26 | **Result**: 26 PASS, 0 FAIL, 0 DARK_FOREST

---

## Executive Summary

The outer membrane of primals.eco passes **all 26** security checks when probed
from an external vantage point. All critical, high, and medium severity checks
pass. ODN-02 (DNSSEC) was resolved on 2026-07-11 when Cloudflare DNSSEC was
enabled with DS record at Porkbun (keyTag 2371, algorithm 13 ECDSAP256SHA256).

**Clean sweep: zero findings.**

The scan validates that the defense-in-depth posture deployed in Wave 136a —
security headers, Caddy hardening, fail2ban, WireGuard mesh isolation, Forgejo
SSH lockdown, depot read-only enforcement — is functioning as intended from an
external adversary's perspective.

---

## Results by Module

### outer.tls (OTR-01→06) — 6/6 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| OTR-01 | critical | TLS endpoint reachable | HTTPS GET / → 200 |
| OTR-02 | high | HSTS present (preload + includeSubDomains) | Full HSTS configuration |
| OTR-03 | high | TLS version modern | TLSv1_3, TLS13_AES_128_GCM_SHA256 |
| OTR-04 | medium | Server header suppressed | No Server header leaked |
| OTR-05 | critical | Certificate accepted by client | Chain valid (ACME auto-renewal) |
| OTR-06 | high | HTTP→HTTPS redirect | Port 80 → 308 → https://primals.eco/ |

### outer.http (OHT-01→06) — 6/6 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| OHT-01 | critical | Security headers present | nosniff + referrer + permissions |
| OHT-02 | high | Proper 404 | Nonexistent path → 404 |
| OHT-03 | medium | Dangerous methods blocked | TRACE→405, DELETE→405, PUT→405 |
| OHT-04 | critical | Path traversal blocked | `/../../../etc/passwd` → 404 |
| OHT-05 | medium | No directory listing | No listing indicators |
| OHT-06 | high | X-Frame-Options present | Clickjacking protection active |

### outer.depot (ODP-01→04) — 4/4 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| ODP-01 | high | Depot reachable | membrane.primals.eco GET /depot/ → 200 |
| ODP-02 | critical | Write methods rejected | PUT/POST/DELETE → 404 (rejected) |
| ODP-03 | high | checksums.toml served | 3632 bytes of BLAKE3 hash data |
| ODP-04 | medium | Listing disabled/styled | No raw directory listing |

### outer.forge (OFG-01→04) — 4/4 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| OFG-01 | high | Forgejo SSH reachable | Banner: SSH-2.0-Go (port 2222) |
| OFG-02 | critical | Password auth disabled | Password authentication rejected |
| OFG-03 | medium | No version leak (web) | Status 200, no version headers |
| OFG-04 | medium | No public repo listing | Explore page does not expose repos |

### outer.dns (ODN-01→03) — 3/3 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| ODN-01 | critical | AXFR rejected | Zone transfer refused |
| ODN-02 | high | DNSSEC keys published | DNSKEY records present (CF DNSSEC, keyTag 2371, alg 13) |
| ODN-03 | medium | NXDOMAIN correct | No wildcard DNS |

### outer.mesh (OMS-01→03) — 3/3 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| OMS-01 | medium | WireGuard drops invalid probes | No response from :51820 |
| OMS-02 | high | Invalid handshake rejected | No response to fake key |
| OMS-03 | low | Federation port not external | :7700 unreachable (expected) |

---

## Findings

**None.** All 26 checks pass. ODN-02 (DNSSEC) was the last open finding —
resolved 2026-07-11 when Cloudflare DNSSEC was enabled with DS record at
Porkbun (keyTag 2371, algorithm 13 ECDSAP256SHA256). Re-scan confirmed PASS.

---

## Scanner Architecture

darkforest v3.0 outer membrane scanning uses:
- **rustls** (ring crypto backend) for TLS 1.2/1.3 handshakes
- **webpki-roots** for Mozilla root certificate store
- DNS resolution via `std::net::ToSocketAddrs` (system resolver)
- Raw TCP probes for SSH banner detection and WireGuard UDP probing
- `dig` via subprocess for AXFR, DNSKEY, and NXDOMAIN checks

All probes originate from the scanner's public IP (ironGate residential) and
target the domain's publicly resolved address. No internal network access is
used. This validates the same attack surface an external adversary would see.

---

## Comparison: Wave 136c → 136b

| Metric | 136c (2026-07-10) | 136b pre-DNSSEC | 136b post-DNSSEC |
|--------|-------------------|-----------------|------------------|
| PASS | 25 | 25 | **26** |
| FAIL | 0 | 0 | 0 |
| DARK_FOREST | 1 (ODN-02) | 1 (ODN-02) | **0** |
| Duration | ~24s | 23.8s | 13.8s |
| TLS cipher | TLS13_AES_128_GCM_SHA256 | TLS13_AES_128_GCM_SHA256 | TLS13_AES_128_GCM_SHA256 |

Post-DNSSEC scan achieves clean sweep. Duration improvement (23.8s→13.8s) is
due to ODN-02 no longer timing out on DNSKEY lookup.

---

## JSON Reports

- Pre-DNSSEC: `validation/reports/outer_membrane_136b.json`
- Post-DNSSEC: `validation/reports/outer_membrane_136b_dnssec.json`
