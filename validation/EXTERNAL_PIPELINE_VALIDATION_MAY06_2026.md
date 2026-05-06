# External Pipeline Validation — Phase 2a

**Date**: 2026-05-06
**System**: ironGate (i9-14900K, 96 GB DDR5, RTX 5070)
**Composition**: 13/13 primals (full NUCLEUS, dynamically discovered from plasmidBin)
**JupyterHub**: v5.4.5, PAM auth, LocalProcessSpawner
**Tunnel**: cloudflared 2026.3.0 quick tunnel (trycloudflare.com)

---

## What Was Validated

The full external access path from browser to science compute:

```
Browser → Cloudflare edge (TLS termination)
       → cloudflared quick tunnel (QUIC, X25519MLKEM768)
       → ironGate:8000 (JupyterHub)
       → Primal composition (13 primals on localhost)
       → Provenance pipeline (rhizoCrypt → loamSpine → sweetGrass)
```

This is **Step 2a** from `specs/TUNNEL_EVOLUTION.md` — the Cloudflare Tunnel
baseline that all subsequent primal replacements are measured against.

---

## Results Summary

| Stage | Tests | Result |
|-------|-------|--------|
| Local validation (12 checks) | Prerequisites, JupyterHub, Primal APIs, Provenance | **12/12 PASS** |
| External tunnel validation (15 checks) | Endpoints, Latency, Primal APIs, Provenance | **15/15 PASS** |
| Provenance pipeline (run 2) | 11 workloads, 9-phase chain | **8 PASS, 2 RUN, 1 FAIL** |

### Provenance Pipeline (Run 2)

| Workload | Checks | Time | Status | Output BLAKE3 |
|----------|--------|------|--------|---------------|
| wetspring-16s-python-baseline | 0 | 2000ms | RUN | a02b41d2964aeb9d… |
| wetspring-16s-rust-validation | 37 | 0ms | PASS | 7186c78c7b2fbe78… |
| wetspring-algae-16s-rust | 34 | 23000ms | PASS | 1cbcdf2d63d597ec… |
| wetspring-benchmark-python-baseline | 0 | 4000ms | RUN | 8835d19badd4e529… |
| wetspring-cold-seep-pipeline | 8 | 0ms | PASS | ab69c4811638afcc… |
| wetspring-diversity-rust-validation | 27 | 0ms | PASS | b19187af3e891ea3… |
| wetspring-exp001-python-baseline | 0 | 0ms | FAIL | 01f8ca58ed9f464f… |
| wetspring-fajgenbaum-pathway | 8 | 0ms | PASS | a2096e1c4b940b5a… |
| wetspring-gonzales-cpu-parity | 43 | 1000ms | PASS | cf1448b103749264… |
| wetspring-real-ncbi-pipeline | 25 | 0ms | PASS | 5df408e683ef4de3… |
| wetspring-r-industry-parity | 53 | 0ms | PASS | 4d1e507311373cc3… |

**Provenance chain**: DAG session → Merkle root → loamSpine commit → sweetGrass braid
- Session: `019dfe80-8e3f-79e3-a11c-d0efba59543a`
- Merkle root: `cfcd2e136c38d11e5c850c2c346c0edc3ac85cabdce9f78c81dd8fab0bbd956a`
- Braid: `urn:braid:cfcd2e136c38d11e5c850c2c346c0edc3ac85cabdce9f78c81dd8fab0bbd956a`
- Witness: ed25519 (BearDog family seed derived)
- Events: 26

### External Tunnel Metrics (Cloudflare Baseline)

| Metric | Value | Notes |
|--------|-------|-------|
| Tunnel protocol | QUIC | X25519MLKEM768 key exchange |
| Edge location | ORD (Chicago) | Nearest Cloudflare POP |
| /hub/api/ latency (p50) | **270ms** | 5-sample median |
| /hub/api/ latency (min) | 182ms | Best observed |
| /hub/api/ latency (max) | 283ms | Worst observed |
| /hub/login | HTTP 200 | PAM login page serves |
| /hub/health | HTTP 200 | Health check passes |
| /hub/api/ | HTTP 200 | JSON API responds |

These metrics become the **parity targets** for Step 2b (BTSP auth inside
tunnel) and Step 3c (Songbird NAT replaces cloudflared).

---

## 13-Primal Composition Health

All primals dynamically discovered from `plasmidBin/primals/` and verified:

| Primal | Port | Role | Status |
|--------|------|------|--------|
| BearDog | 9100 | Security (BTSP) | ALIVE |
| Songbird | 9200 | Network discovery | ALIVE |
| Squirrel | 9300 | Coordination | ALIVE |
| ToadStool | 9400 | Compute orchestration | ALIVE |
| NestGate | 9500 | Content-addressed storage | ALIVE |
| rhizoCrypt | 9602 | DAG provenance | ALIVE |
| loamSpine | 9700 | Permanent ledger | ALIVE |
| CoralReef | 9730 | Visualization | ALIVE |
| BarraCuda | 9740 | Math/compute | ALIVE |
| skunkBat | 9140 | Metadata defense | ALIVE |
| biomeOS | 9800 | Neural orchestration | ALIVE |
| sweetGrass | 9850 | Attribution braids | ALIVE |
| petalTongue | 9900 | UI rendering | ALIVE |

---

## JupyterHub Configuration

- **Bind**: 127.0.0.1:8000 (localhost only — tunnel provides external access)
- **Auth**: PAM (Linux system users)
- **Spawner**: LocalProcessSpawner
- **Default UI**: JupyterLab (`/lab`)
- **Memory limit**: 32 GB per user
- **CPU limit**: 8 cores per user
- **Kernels**: Python (conda:bioinfo), R (conda:r-bioinfo), base Python
- **Idle culling**: 1 hour
- **Admin**: irongate

### ABG Notebook

`~/notebooks/abg-wetspring-validation.ipynb` — ready for ABG members:
1. Health check all 13 primals
2. Query ToadStool capabilities
3. Run wetSpring 16S workloads (11 TOML definitions)
4. Inspect provenance manifest and braid
5. Query individual primal APIs (NestGate, Squirrel, BarraCuda, etc.)
6. Run individual workloads interactively
7. Test external access endpoint

---

## Known Issues

| Issue | Severity | Notes |
|-------|----------|-------|
| `wetspring-exp001-python-baseline` FAIL | Low | Missing dependencies in exp001 workload — wetSpring issue |
| 2 RUN workloads (no checks) | Info | Python baselines produce output but no structured checks |
| Quick tunnel is ephemeral | Expected | Production setup requires named tunnel with Cloudflare account |
| UDP buffer size warning | Info | cloudflared wants 7168 KiB, got 416 KiB — QUIC performance non-blocking |
| No BTSP on tunnel path yet | Expected | Step 2b adds BearDog BTSP auth inside tunnel |

---

## Files Created/Modified

| File | Purpose |
|------|---------|
| `deploy/external_validation.sh` | External validation pipeline script (--local / --tunnel modes) |
| `notebooks/abg-wetspring-validation.ipynb` | Updated for 13 primals + external access cell |
| `validation/external-tunnel-20260506/` | Tunnel validation logs and results |
| `validation/external-20260506-142358/` | Local validation results (12/12) |
| This document | Comprehensive validation report |

---

## Next Steps (from `specs/TUNNEL_EVOLUTION.md`)

1. **Step 2a complete** — Cloudflare Tunnel baseline captured (270ms p50)
2. **Step 2b**: Add BearDog BTSP ionic token auth inside the tunnel
3. **Step 3a**: Move sporePrint content from GitHub Pages to NestGate
4. **Step 3b**: BTSP replaces Cloudflare TLS termination
5. **Step 3c**: Songbird NAT traversal replaces cloudflared
6. **Step 4**: Sovereign DNS — zero external dependencies

Each step validated against the baselines captured here via `infra/benchScale`.
