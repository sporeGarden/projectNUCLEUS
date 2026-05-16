# projectNUCLEUS Experiments

Validation experiments for sovereign NUCLEUS deployment.

## Active Experiments

| ID | Name | Crate/Tool | Tests | Status |
|----|------|-----------|-------|--------|
| E1 | Security boundary validation | `validation/darkforest/` | 34 | PASS |
| E2 | Transport sovereignty | `validation/tunnelKeeper/` | 21 | PASS |
| E3 | Sovereignty parity | `infra/benchScale/` | — | ACTIVE |

## E1: Dark Forest Security Validation

Pure Rust security validator. Probes all 13 NUCLEUS primals via:
- JSON-RPC protocol fuzzing (20+ malformed payloads per primal)
- HTTP pentest (external, compute, readonly tiers)
- Crypto strength (cookie entropy, shadow hash, BTSP negotiation)
- Observer-tier access control

**Discovery evolution** (Wave 18): Replaced hardcoded primal port table with
3-tier capability-based discovery: biomeOS `primal.list` → env vars → compiled
defaults. Each primal probed via `health.liveness` and `capability.list`.

## E2: Tunnel Keeper Transport Validation

Pure Rust tunnel manager validates Cloudflare transport layer:
- Health probes: process, connectivity, DNS, config, replicas
- Credential encryption: ChaCha20-Poly1305 (BearDog pattern)
- reqwest 0.13 with rustls (ring dependency eliminated)

## E3: Sovereignty Parity (benchScale)

Shadow-run protocol comparing external dependencies vs sovereign replacements:
- Cloudflare tunnel baseline → BearDog TLS channel
- GitHub Pages → NestGate + petalTongue
- Cloudflare DNS → sovereign knot-dns (pending)
- Songbird NAT relay validation

## Test Summary

```
darkforest   34 tests (crypto, check, report, discovery)
tunnelKeeper 21 tests (config, crypto, health)
─────────────────────────────────────────────
Total        55 tests, 0 failures
```

All crates: `#![forbid(unsafe_code)]`, zero clippy warnings (pedantic+nursery),
cargo fmt clean, graphs synchronized to primalSpring v3.0.0.
