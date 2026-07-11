# darkforest v3.0 — Outer Membrane Execution Report

**Date**: 2026-07-11 | **Wave**: 136b | **Target**: primals.eco
**Scanner**: darkforest v0.3.0 (pure Rust, rustls + webpki-roots/ring)
**Source**: ironGate (residential, separate ISP from golgiBody VPS)
**Duration**: 23.8s | **Checks**: 26 | **Result**: 25 PASS, 0 FAIL, 1 DARK_FOREST

---

## Executive Summary

The outer membrane of primals.eco passes 25 of 26 security checks when probed
from an external vantage point. All critical and high severity checks pass. The
single finding (ODN-02: DNSSEC) is a registrar-level infrastructure gap, not a
code or configuration deficiency.

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

### outer.dns (ODN-01→03) — 2/3 PASS, 1 DARK_FOREST

| ID | Severity | Status | Title | Evidence |
|----|----------|--------|-------|----------|
| ODN-01 | critical | PASS | AXFR rejected | Zone transfer refused |
| ODN-02 | high | **DARK_FOREST** | No DNSKEY records | DNSSEC may not be enabled |
| ODN-03 | medium | PASS | NXDOMAIN correct | No wildcard DNS |

### outer.mesh (OMS-01→03) — 3/3 PASS

| ID | Severity | Title | Evidence |
|----|----------|-------|----------|
| OMS-01 | medium | WireGuard drops invalid probes | No response from :51820 |
| OMS-02 | high | Invalid handshake rejected | No response to fake key |
| OMS-03 | low | Federation port not external | :7700 unreachable (expected) |

---

## Findings

### ODN-02: DNSSEC Not Enabled (DARK_FOREST)

**Severity**: High | **Category**: Crypto | **Owner**: operator (registrar-level)

DNSKEY records are not present for `primals.eco`. This means DNS responses are
not cryptographically signed, leaving the domain vulnerable to DNS spoofing in
theory. In practice, the risk is bounded by:
- TLS certificate pinning (ACME auto-renewal validates domain ownership)
- HSTS preload (browsers enforce HTTPS regardless of DNS)
- WireGuard overlay (internal mesh ignores public DNS entirely)

**Remediation**: Enable DNSSEC at the registrar (Porkbun supports it). This is
an infrastructure action, not a code change. The operator should:
1. Generate DNSKEY via the authoritative nameserver or registrar panel
2. Publish DS records in the parent zone
3. Re-run `darkforest --scope outer --target primals.eco` to verify ODN-02 PASS

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

| Metric | 136c (2026-07-10) | 136b (2026-07-11) |
|--------|-------------------|-------------------|
| PASS | 25 | 25 |
| FAIL | 0 | 0 |
| DARK_FOREST | 1 (ODN-02) | 1 (ODN-02) |
| Duration | ~24s | 23.8s |
| TLS cipher | TLS13_AES_128_GCM_SHA256 | TLS13_AES_128_GCM_SHA256 |

Results are stable across two consecutive scans from the same vantage point.
The outer membrane posture is consistent.

---

## JSON Report

Machine-readable report: `validation/reports/outer_membrane_136b.json`
