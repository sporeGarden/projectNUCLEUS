# projectNUCLEUS Deep Debt + Evolution AAR — Aug 2, 2026

**Date**: 2026-08-02 | **Wave**: 155n (Springs+Gardens Phase)
**Gate**: ironGate (i9-14900K, RTX 5070, NUCLEUS 13/13 LIVE)
**From**: ironGate overwatch
**Scope**: Workspace modernization, dependency evolution, hardcoding elimination,
test coverage expansion, documentation reconciliation.

---

## What Was Done

### 1. Root Workspace Unification

Created `/Cargo.toml` workspace root unifying all 4 crates under a single
`cargo test --workspace` / `cargo clippy --workspace` surface:

- `deploy/nucleus-deploy` (CLI binary, 9 subcommands)
- `deploy/nucleus-primals` (shared registry library)
- `validation/darkforest` (security validator, 14 modules)
- `validation/tunnelKeeper` (tunnel health, dual transport)

Workspace-level `[workspace.package]` (edition, license, rust-version) and
`[workspace.lints]` (unsafe_code forbid, clippy pedantic+nursery) eliminate
4× duplication. Crates inherit via `edition.workspace = true`.

### 2. Edition 2024 Migration

`nucleus-deploy` and `nucleus-primals` evolved from edition 2021 → 2024.
darkforest and tunnelKeeper were already 2024. All 4 crates now on edition
2024 with workspace inheritance.

### 3. chrono Elimination

Removed the `chrono` crate (C-backed time via `iana-time-zone` → libc)
from `nucleus-deploy`. Replaced with pure `std::time::SystemTime` helpers
in `util.rs`:

- `utc_timestamp()` — ISO 8601 UTC
- `utc_compact()` — filename-safe timestamps
- `time_stamp()` — log prefix
- `utc_date()` — date only
- `utc_serial()` — DNS SOA serial format
- `utc_timestamp_secs()` — epoch seconds
- `utc_date_days_ago(days)` — rolling window dates

10 source files migrated. Zero transitive chrono dependency remains.

### 4. Infrastructure Port Constants

Added 8 named infrastructure port constants to `nucleus-primals`:

| Constant | Port | Was |
|----------|------|-----|
| `JUPYTERHUB_DEFAULT_PORT` | 8000 | Magic literal in 2 files |
| `OBSERVER_DEFAULT_PORT` | 8866 | Magic literal in 2 files |
| `SONGBIRD_FEDERATION_DEFAULT_PORT` | 7700 | Magic literal in 2 files |
| `BTSP_SHADOW_DEFAULT_PORT` | 8443 | Magic literal in 1 file |
| `RUSTDESK_HBBS_DEFAULT_PORT` | 21116 | Magic literal in 1 file |
| `MEMBRANE_HTTP_DEFAULT_PORT` | 80 | Magic literal in 1 file |
| `FORGEJO_SSH_DEFAULT_PORT` | 2222 | Magic literal in 1 file |
| `WIREGUARD_DEFAULT_PORT` | 51820 | Magic literal in 1 file |

Collision tests verify no infra port overlaps with the 15-entry primal
port registry.

### 5. Hardcoding Evolution

- **Port literals** → `nucleus_primals` constants across 8 source files
- **`/home/nobody` fallback** → `/tmp/darkforest` (crypto/observer modules)
- **Duplicated slug lists** in darkforest tests → derived from `nucleus_primals::COMP_FULL`
- **tunnelKeeper** now depends on `nucleus-primals` (shared port constants)

### 6. Clippy + Fmt Polish

- `dispatch()` in main.rs: `#[expect(clippy::too_many_lines)]` with reason
- Doc comment backticks in nucleus-primals
- Unnecessary `collect()` → direct iterator chain
- All warnings resolved: clippy pedantic+nursery clean across workspace

### 7. Documentation Reconciliation

All 3 root docs (README.md, PHASES.md, EVOLUTION_GAPS.md) updated:

- Test counts: 245 (stale) → **265** (actual)
- Per-crate: darkforest 149, tunnelKeeper 48+1 ignored, nucleus-deploy 49, nucleus-primals 19
- biomeOS version: v3.x → v4.56
- Edition: noted 2024 workspace
- Changelog entry added to EVOLUTION_GAPS.md

---

## Metrics

| Metric | Before | After |
|--------|--------|-------|
| Workspace | 4 independent crates | Unified root workspace |
| Edition | 2021 (2 crates) + 2024 (2 crates) | **2024** (all 4) |
| Tests | 245 (documented) / 258 (actual) | **265** (1 ignored) |
| External deps eliminated | chrono (C-backed time) | **Pure std** |
| Hardcoded port literals | 12+ across 8 files | **0** (8 named constants) |
| Clippy warnings | 0 | **0** |
| Unsafe blocks | 0 | **0** |
| TODO/FIXME/HACK | 0 | **0** |
| Files >800 lines | 0 | **0** (max 744L) |
| Production mocks | 0 | **0** |
| Files changed | — | **57** (+553 / -273) |

---

## What Remains — Live Primal Replacement Opportunities

With NUCLEUS 13/13 LIVE on ironGate (26 active sockets), many remaining
external systems in EVOLUTION_GAPS can now be replaced with live primal
compositions rather than shadow runs or scripts:

### Ready to Replace NOW (primals are running)

| External System | Primal Replacement | Status |
|----------------|-------------------|--------|
| **JupyterHub auth (PAM)** | BearDog BTSP `auth.issue_session` | Shadow LIVE. Dual-auth deployed. Cut PAM when 7-day clean. |
| **GitHub Pages content** | NestGate `content.*` + petalTongue `backend=nestgate` | HTTP parity PASS (67ms vs 89ms). Mirror content → NestGate, DNS switch. |
| **Cloudflare TLS** | BearDog TLS on :8443 | Shadow LIVE. 3ms vs 120ms (40×). 7-day p95 measurement → cutover. |
| **cloudflared tunnel** | Songbird TURN relay on VPS | LIVE 100% reachable. Dual-path shadow → cutover. |
| **Observer Python server** | petalTongue static serving | petalTongue web mode LIVE on VPS :8080. Replace `observer_server.py`. |
| **pappusCast Python** | biomeOS `composition.deploy` + toadStool dispatch | Deploy graphs + workload TOMLs already wired. Python layer vestigial. |
| **Cloudflare DNS** | knot-dns on VPS (H2-17 DEPLOYED) | Authoritative zone live. NS registrar cutover pending (human action). |
| **`publish_sporeprint.sh`** | NestGate `content.put` pipeline | Script ready. Blocked only on BearDog `content.*` scope (SP-4). |
| **GitHub Actions CI** | Forgejo Actions / Sovereign CI | sporeGate sovereign CI LIVE. 74 workflows to port (H3-03). |

### Next Sovereignty Sprint Priorities

1. **SP-4 upstream ask**: BearDog `auth.issue_session` scope expansion for `content.*`
   → Unblocks `publish_sporeprint.sh`, NestGate content pipeline, sovereign content hosting
2. **S1 TLS cutover**: BearDog :8443 7-day p95 measurement → DNS switch
3. **S3 content mirror**: `content.put` GitHub Pages → NestGate, DNS staging subdomain
4. **Observer replacement**: petalTongue `--docroot` replaces `observer_server.py` (8866)
5. **NS registrar cutover**: knot-dns authoritative → register NS records (human)

### Architecture Insight

The ironGate NUCLEUS substrate makes most Python deploy tooling vestigial.
The remaining Python files (`pappusCast.py`, `observer_server.py`,
`tier_test_*.py`, `jupyterhub_btsp_auth.py`) are all candidates for
absorption into Rust binaries or live primal IPC:

- `pappusCast` auto-propagation → biomeOS `composition.deploy` + toadStool dispatch
- `observer_server.py` → petalTongue static web mode
- `tier_test_*.py` → darkforest test modules
- `jupyterhub_btsp_auth.py` → BearDog native BTSP auth (post-JupyterHub retirement, H3-01)

The diesel engine doesn't care what card it's talking to. The primals
don't care what service they're replacing. Same pattern: calibrate →
shadow → cutover.

---

## Deferred (analyzed, not blocking)

| Item | Assessment | When |
|------|-----------|------|
| **reqwest in tunnelKeeper** | Contained to `api.rs` (4 Cloudflare API calls). Replace with raw TCP/TLS like darkforest. Medium effort. | Dedicated sprint |
| **ring ASM in rustls** | Irreducible. Monitored in `deny.toml`. No pure-Rust alternative at equivalent performance. | N/A (accepted) |
| **Python deploy layer** | 17 `.py` files. Evolution path: Python → Rust → primal IPC. | Incremental per above |

---

*projectNUCLEUS Wave 155n — 265 tests, 0 unsafe, 0 chrono, 0 hardcoded ports.
Workspace unified. Edition 2024. 57 files evolved. Live primals ready to
replace 9 external systems. The substrate is the product.*
