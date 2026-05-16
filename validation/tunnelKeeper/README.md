# tunnelKeeper

Pure Rust Cloudflare tunnel manager for NUCLEUS. Manages tunnel configuration,
health probes, ingress routing, and credential encryption at rest using BearDog
crypto patterns.

## Dual-Architecture Evolution

tunnelKeeper is designed for incremental sovereignty — Cloudflare remains the
primary transport while primal equivalents are built and validated:

| Version | Transport | Status |
|---------|-----------|--------|
| v0.1 | `CloudflareTunnelTransport` — wraps `cloudflared` process | Superseded |
| v0.2 | Health, config, routing, credential encryption (pure Rust) | **Current** |
| v0.3 | `SongbirdTransport` — `songbird-quic` + `songbird-tls` as library deps | Planned |
| v0.3 | `BearDogAuthTransport` — `beardog-auth` replaces CF Access | Planned |

The `TunnelTransport` trait defines the boundary. Shadow-run protocol validates
parity before cutover (see `specs/TUNNEL_EVOLUTION.md` Step 3).

## Usage

```
tunnelKeeper health                         # probe tunnel + DNS + config
tunnelKeeper --json health                  # machine-readable output
tunnelKeeper config show                    # display current config
tunnelKeeper route list                     # list ingress rules
tunnelKeeper route add --hostname X --service Y
tunnelKeeper creds encrypt                  # ChaCha20-Poly1305 at-rest encryption
tunnelKeeper creds decrypt                  # restore for cloudflared
```

## Dependencies (all pure Rust)

- `clap` — CLI (same pattern as darkforest)
- `serde` / `serde_json` / `serde_yaml` — config parsing
- `reqwest` 0.13 with `rustls` + `webpki-roots` — CF API v4 client (no OpenSSL, no `ring`)
- `tokio` — async runtime
- `chacha20poly1305` — credential encryption (BearDog pattern)
- `ed25519-dalek` — key derivation (BearDog pattern)
- `chrono` — timestamps

## Primal Integration Points

- **BearDog**: `ed25519-dalek` + `chacha20poly1305` for credential wrapping
  (mirrors `beardog-tunnel` patterns). Future: `beardog-auth` for ionic tokens.
- **Songbird**: Future `songbird-quic` + `songbird-tls` for direct QUIC tunnel
  transport, replacing `cloudflared` binary dependency.
- **darkforest**: `tunnelKeeper health --json` output consumed by darkforest
  tunnel health checks.
