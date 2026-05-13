# Evolution Gaps — projectNUCLEUS

Living tracker of remaining gaps across three horizons. Updated as gaps
close and new ones emerge. Each gap is local — actionable by projectNUCLEUS
without waiting on upstream unless noted.

**Last updated**: 2026-05-13 (Atomic deployment phase — 13/13 primals zero debt, 8/8 springs zero debt, Tower LIVE, Nest ready, Node in progress)
**Validation baseline**: 267 PASS, 0 FAIL, 0 KNOWN_GAP (bash 5-layer)
**Rust validator**: darkforest v0.2.1 — 8 modules, `--suite observer` static surface validation
**Multi-tier tests**: observer (darkforest Rust) + reviewer + compute + hub + pappusCast + sporePrint (`deploy/tier_test_all.sh`)
**Architecture**: Cell membrane model — primals.eco on GitHub Pages CDN (extracellular), lab/git.primals.eco via tunnel (membrane), sovereign compute inside
**Upstream status**: All upstream gaps resolved. 13/13 primals zero code debt, 8/8 delta springs zero debt (8,486+ tests). Tower atomic LIVE (ludoSpring 6/6). Nest atomic ready (GAP-36 resolved). Node atomic AMD live, NV FECS-gated.
**Tier 2 Science API**: `toadstool.validate` IMPLEMENTED (S250, 74 methods), `barracuda.precision.route` v0.4.0 (649 tests), `composition.deploy.shadow` biomeOS v3.53 SHIPPED, `biomeos.spring_status` IMPLEMENTED (v3.54). Registry at **415 methods**.

Related specs:
- [TUNNEL_EVOLUTION.md](TUNNEL_EVOLUTION.md) — sovereignty replacement roadmap
- [SECURITY_VALIDATION.md](SECURITY_VALIDATION.md) — five-layer validation model
- [SOVEREIGNTY_VALIDATION_PROTOCOL.md](SOVEREIGNTY_VALIDATION_PROTOCOL.md) — replacement methodology
- [COMPLETE_DEPENDENCY_INVENTORY.md](COMPLETE_DEPENDENCY_INVENTORY.md) — full dependency map

**Rust evolution**: `validation/darkforest/` v0.2.1 — modular auditable security framework (zero
runtime deps). 8 source modules: `check.rs` (structured types + env-var-driven primal config),
`net.rs` (TCP/HTTP helpers), `pentest.rs` (3 threat actors), `fuzz.rs` (14 primals + JupyterHub),
`crypto.rs` (13 crypto strength checks, gate-agnostic paths), `observer.rs` (static HTML quality:
theme, nav, links, tracebacks, source stripping, HTTP headers, directory blocking),
`report.rs` (pipe + JSON output), `main.rs` (CLI + runner). All ports and paths resolve from
environment variables with compiled defaults — zero hardcoded gate paths.

**Observer surface**: Static pre-rendered HTML via pappusCast + `observer_server.py` on port 8866.
Centralized dark theme (`observer_theme.css`). Validated by `darkforest --suite observer`
(86 PASS, 0 FAIL — theme, nav, links, tracebacks, source stripping, headers, directory blocking).

---

## Horizon 1: External Security & ABG Hosting

Hardening what's live now — JupyterHub, Cloudflare tunnel, PAM auth,
systemd services. No primal evolution required. All local work.

### Open

None. All Horizon 1 gaps resolved.

### Resolved (fossil record)

| ID | What | When | How |
|----|------|------|-----|
| JH-0 | Unauthenticated RPC | 2026-05-08 | MethodGate enforced, 10/13 confirmed TCP |
| JH-1 | No caller identity | 2026-05-08 | BearDog ionic tokens live |
| JH-2 | No resource limits | 2026-05-08 | biomeOS + ToadStool enforce envelopes |
| JH-3 | Full restart required | 2026-05-08 | biomeOS `composition.reload` |
| JH-4 | Session UX | 2026-05-08 | `auth.issue_session` with presets |
| JH-5 | No audit log | 2026-05-08 | skunkBat ring buffer, Phase 2 |
| JH-8 | DNS exfil open | 2026-05-08 | iptables DNS restricted to local resolver |
| JH-9 | Conda envs group-writable | 2026-05-08 | Root-owned, 755 |
| DF-1 | 5 primals bind 0.0.0.0 | 2026-05-08 | Phase 60 PG-55 default binding |
| H1-01 | `/hub/api/` version disclosure | 2026-05-08 | Headers suppressed; body is upstream built-in (JH-10). Accepted risk — localhost only |
| H1-02 | Voila executes as hub user | 2026-05-08 | Dedicated `voila` system user (UID 998). `user: 'voila'` in JupyterHub service config |
| H1-03 | rustdesk on 0.0.0.0 | 2026-05-08 | Already binds LAN IP (192.168.1.238), not 0.0.0.0. No action needed |
| H1-06 | Cookie secret rotation | 2026-05-08 | `deploy/rotate_cookie_secret.sh` created. Manual rotation — run monthly or after events |
| H1-07 | Baseline 7-day summary | 2026-05-08 | Cron capturing hourly (deduplicated). Run `summarize_baselines.sh` after May 14 |
| H1-04 | systemd enumeration | 2026-05-08 | `setfacl` deny execute on `/usr/bin/systemctl` for abg-compute/reviewer/observer groups |
| H1-05 | Reviewer python3 access | 2026-05-08 | `setfacl` deny execute on `/usr/bin/python3` for abg-reviewer/observer. Compute and admin unaffected |

---

## Horizon 2: Sovereignty — Replacing External Services

Progressive elimination of Cloudflare, GitHub, and PAM. Each step follows
the calibrate → shadow-run → cutover protocol in `SOVEREIGNTY_VALIDATION_PROTOCOL.md`.

### Step 2b: BTSP Auth (replaces PAM passwords)

**Status**: Ready to build. All primal prerequisites resolved.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-01 | ~~Build `jupyterhub_btsp_auth.py` authenticator plugin~~ | — | **DONE** — `deploy/jupyterhub_btsp_auth.py` + `deploy/deploy_btsp_auth_shadow.sh`. Dual-auth mode, PAM fallback, auth logging, pre_spawn_hook for tier injection |
| H2-02 | Token distribution UX | Low | CLI: `auth.issue_session(purpose="jupyterhub")` → user pastes token as password. No web UI needed yet |
| H2-03 | Dual-auth shadow run (7 days) | Ops | PAM + BTSP both accepted. Log which auth each login uses. Compare success rates and latency |
| H2-04 | PAM cutover criteria | — | BTSP auth success rate ≥ 99.9%, latency overhead < 50ms, 7-day shadow run clean |

**Not blocked by JH-11**: The authenticator only calls beardog (same-primal),
not cross-primal methods. Cross-primal federation is Horizon 3.

### Step 3a: sporePrint sovereign rendering (petalTongue replaces GitHub Pages)

**Status**: CELL MEMBRANE LIVE. `primals.eco` permanently on GitHub Pages CDN (extracellular).
`lab/git.primals.eco` via tunnel replicas (membrane). Full primal path (NestGate + petalTongue)
remains Phase 3 target for sovereign extracellular rendering.

**Cell membrane architecture (operational May 10, 2026)**:
- `primals.eco` DNS permanently set to GitHub Pages A records (extracellular)
- `lab.primals.eco` + `git.primals.eco` via Cloudflare tunnel replica pool (membrane)
- `deploy/gate_provision.sh` provisions new membrane replicas
- `deploy/gate_watchdog.sh` monitors membrane health (logs for skunkBat, no DNS swapping)
- `tunnelKeeper v0.2.0` reports replica count, edge colos, unique origins
- `sporeprint-local.service` demoted to dev/preview (not production path)
- `deploy/sporeprint_dns.sh` emergency use only (sovereign/external switching)

**DNS sovereignty (operational May 9, 2026)**:
- Gate resolves via DNS-over-TLS to 1.1.1.1 (Cloudflare) with Quad9 fallback
- ISP resolver (AT&T) bypassed — no DNS metadata leak to ISP
- `/etc/systemd/resolved.conf` configured with `DNSOverTLS=yes`

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-05 | ~~Build NestGate content pipeline~~ | **DONE** | NestGate Session 60: `content.put/get/exists/list/publish/resolve/promote/collections` on all 4 transports. `publish_sporeprint.sh` ready to wire. |
| H2-06 | Configure petalTongue web mode | Low | `--docroot` resolved Phase 60. `backend=nestgate` **UNBLOCKED** — SPA + CORS shipped. Wire `content.get`/`content.resolve` for production. |
| H2-07 | ~~Cloudflare ingress route~~ | **DONE** | `primals.eco` in tunnel config, DNS switchable via API |
| H2-08 | Shadow run: Zola/tunnel vs NestGate/petalTongue (7 days) | Ops | **UNBLOCKED** — `nestgate_content_parity.sh` ready. Compare TTFB, Lighthouse scores. |
| H2-09 | Cutover: petalTongue replaces Zola static server | — | 100% content parity, TTFB within 10% of Zola |

### Step 3b: BearDog TLS (replaces Cloudflare TLS)

**Status**: Upstream shipped. Ready for local shadow run.

bearDog Wave 100 shipped rustls X.509 TLS termination + per-IP sliding-window rate
limiter (sovereignty horizons H2-10/H2-11). Local work: shadow run + cutover.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-10 | ~~BearDog TLS listener~~ | — | **RESOLVED** — bearDog Wave 100: rustls X.509 termination, SNI routing, cert auto-renewal |
| H2-11 | ~~BearDog rate limiting~~ | — | **RESOLVED** — bearDog Wave 100: per-IP sliding-window rate limiter |
| H2-12 | ~~Shadow run on port 8443 alongside Cloudflare 443~~ | — | **RUNNING** — BearDog v0.9.0 on :8443 (PID live), 10ms RPC vs 120ms Cloudflare. `btsp_tls_parity.sh` ready for 7-day comparison |

### Step 3c: Songbird NAT (replaces cloudflared)

**Status**: Upstream shipped. Ready for local integration.

songbird Wave 196-197 shipped full NAT traversal chain: STUN wire-compliant (RFC 5389),
RFC 5766 TURN client, Cloudflare DDNS, 5-tier `ConnectionFallbackChain`. Local work:
VPS relay provisioning + integration testing.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-13 | ~~Songbird STUN client~~ | — | **RESOLVED** — songbird Wave 196: STUN wire-compliant (RFC 5389) |
| H2-14 | Self-hosted STUN/TURN VPS relay | Ops | **OPS-READY** — `deploy_songbird_relay.sh` staged, systemd unit in upstream, binary in plasmidBin. Provision ~$5/mo VPS + `bash deploy_songbird_relay.sh --host <vps-ip>` |
| H2-15 | ~~Dynamic DNS update~~ | — | **RESOLVED** — songbird Wave 197: Cloudflare DDNS integration |
| H2-16 | ~~Connection fallback chain~~ | — | **RESOLVED** — songbird Wave 197: 5-tier `ConnectionFallbackChain` (direct → STUN → TURN → cloudflared → offline) |

### Step 4: Sovereign DNS (replaces Cloudflare NS)

**Status**: INTERMEDIATE. Gate DNS queries encrypted (DoT). Full sovereignty not started.

**Intermediate layer (operational May 9, 2026)**:
- `/etc/systemd/resolved.conf`: `DNSOverTLS=yes`, primary 1.1.1.1, fallback 9.9.9.9
- ISP (AT&T) resolver bypassed — AT&T sees encrypted traffic to 1.1.1.1, no query content
- Fixes `.eco` TLD resolution failures on ISP resolver
- Still trusts Cloudflare resolver — better than ISP, but not sovereign

**Metadata leak analysis**:
- **Closed**: ISP cannot see DNS query content (encrypted transport)
- **Remaining**: Cloudflare (1.1.1.1) sees query content (trusted, not sovereign)
- **Remaining**: Cloudflare proxy sees all HTTP traffic (tunnel terminates at their edge)
- **Phase 3 closes all**: BTSP P2P resolution eliminates DNS, Songbird eliminates tunnel

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-17 | `knot-dns` authoritative on VPS | Medium | DNSSEC, monitoring via skunkBat |
| H2-18 | NS transfer from Cloudflare registrar | Ops | Update registrar to point NS at VPS |
| H2-19 | BTSP direct resolution for ecosystem clients | Low | Bypass DNS for BTSP-aware tools |
| H2-20 | Local recursive resolver (unbound) | Low | Eliminate Cloudflare DoT dependency for non-BTSP queries |

---

## Horizon 3: Primal-Only (zero external dependencies)

Long-term evolution. Each item replaces an external service with a primal
composition. Not actionable until Horizon 2 steps validate the patterns.

| ID | External Service | Primal Replacement | Status | Blocks |
|----|-----------------|-------------------|--------|--------|
| H3-01 | JupyterHub (notebook UX) | petalTongue dashboards + biomeOS dispatch | Gap — no notebook execution in petalTongue | Horizon 2 complete |
| H3-02 | GitHub Releases (plasmidBin) | NestGate blob storage + manifest queries | **UNBLOCKED** — NestGate `content.put` shipped (Session 60). Wire `fetch.sh` to NestGate. | Local: update fetch URLs |
| H3-03 | GitHub Actions (CI/CD) | Forgejo Actions or self-hosted runners | Gap — 74 workflow files to port | Forgejo primary adoption |
| H3-04 | GitHub repos (source hosting) | Forgejo primary, GitHub mirror | Calibration — Forgejo installed, not primary | Forgejo Actions working |
| H3-05 | Docker Hub / ghcr.io | NestGate OCI blob store | **UNBLOCKED** — NestGate `content.put` shipped. Wire toadStool config. | Local: OCI store wiring |
| H3-06 | Anthropic / OpenAI | Ollama + barraCuda WGSL inference | Partial — Ollama works locally | barraCuda shader maturity |
| H3-07 | JH-11 cross-primal token federation | biomeOS composition forwarding with `_resource_envelope` | **UNBLOCKED** — bearDog `auth.public_key` + biomeOS `BearDogVerifier` live | Local: wire `CompositionContext` |
| H3-08 | JH-5 cross-primal audit forwarding | skunkBat → rhizoCrypt DAG + sweetGrass braids | **UNBLOCKED** — skunkBat Phase 2 complete (7 event kinds) | Local: wire into deploy graphs |
| H3-09 | conda/pip/crates.io | Vendored deps, private registry | Low priority — offline modes exist | Not blocking |
| H3-10 | NCBI / UniProt / KEGG | Local mirror + `abg_data.sh` provenance | Partial — data registry operational | Not blocking (data, not service) |

### Irreducible Externals (never sovereign)

These are not gaps — they are accepted constraints:

- Domain registrar (`primals.eco`)
- Linux kernel / systemd
- NVIDIA GPU drivers
- Let's Encrypt / ACME (browser trust chain)
- VPS for STUN relay (~$5/mo, commodity)

---

## Upstream Dependencies (primal teams)

**River delta Push 3+ (May 13, 2026)**: Zero open upstream gaps. 13/13 primals at zero
code debt. 8/8 delta springs at zero debt, 8,486+ tests. Tower atomic LIVE (ludoSpring 6/6).
Nest atomic ready. Node atomic AMD live, NV FECS-gated. `toadstool.validate` S250 (74 methods).
`barracuda.precision.route` v0.4.0 (649 tests). `composition.deploy.shadow` biomeOS v3.53.
`biomeos.spring_status` IMPLEMENTED (v3.54). Registry at **415 methods** (was 413).
`content.put/get` NestGate Session 60 (4-surface parity). BTSP auth pipeline live (13/13 primals).
skunkBat audit pipeline JH-5 Phase 3 operational. Tier 2 JSON-RPC on all 7 springs (`--format json`).
76 wire routing misroutes fixed — `security.audit_log` → skunkBat, crypto methods base64-encoded,
`provenance.*` → sweetGrass. `s_routing_consistency` scenario prevents drift.

See: `infra/wateringHole/handoffs/PRIMALSPRING_POST_INTERSTADIAL_DOWNSTREAM_HANDOFF_MAY10_2026.md`

### All Resolved

| ID | What | Resolved by | When |
|----|------|-------------|------|
| JH-11 | Cross-primal token federation | bearDog Wave 99 `auth.public_key` + biomeOS v3.51 `BearDogVerifier` | May 10 |
| GAP-03 | biomeOS cell graph live deploy | biomeOS v3.51 `composition.deploy` route alias | May 10 |
| GAP-06 | rhizoCrypt no UDS transport | rhizoCrypt S66 — operational since S23, integration test added | May 10 |
| GAP-09 | biomeOS Neural API registration | biomeOS v3.51 `method.register` endpoint | May 10 |
| GAP-12 | ludoSpring IPC method registration | 28 `game.*` methods registered (415 canonical, zero drift) | May 13 |
| U1 | primalSpring CHECKSUMS stale | Regenerated (25 files, BLAKE3) | May 10 |
| U2 | Deploy graphs missing `by_capability` | False positive — manifests, not node-bearing graphs | May 10 |
| U3 | Profile graphs missing `bonding_policy` | 9/9 already have `bonding_policy` | May 10 |
| DF-2 | toadStool `TOADSTOOL_AUTH_MODE` env | toadStool S233 — auth.mode env + eprintln→tracing | May 8 |
| DF-3 | songbird/squirrel silent on `auth.mode` TCP | songbird — CallerContext wired (TCP transport-aware) | May 8 |
| U5 | sweetGrass port 39085 vs 9850 | sweetGrass v0.7.32 — port 9850 canonical | May 8 |

### Absorption Targets (local work, now unblocked)

These are not upstream gaps — they are composition patterns we should absorb
from the resolved upstream work. All local, actionable now.

| Target | What to wire | Source | Priority |
|--------|-------------|--------|----------|
| `composition.deploy(graph)` | `deploy_graph.sh` created — `--graph-deploy` reads TOML, starts in dependency order | biomeOS v3.51 `composition.deploy` route alias | **WIRED** |
| ~~`composition.deploy.shadow`~~ | `shadow_deploy()` in `deploy_graph.sh` — dry-run graph validation, port conflict detection, toadstool.validate pre-flight | biomeOS v3.53 | **WIRED** (May 13) |
| ~~toadstool.validate workloads~~ | 12 validation workload TOMLs wired: `[output] schema = "toadstool-validate-v1"` + `--format json` across all 7 springs | toadStool S250 | **WIRED** (May 13) |
| ~~DoT parity baseline~~ | `dot_sovereign_parity.sh` + orchestrator integration — DNS timing, DoT status, sovereign resolver comparison | H2-4 / H2-17→20 | **WIRED** (May 13) |
| `composition.status` | Wire `{ active_users, primal_health, resource_pressure }` into monitoring | biomeOS v3.51 | Medium |
| `method.register` | Dynamic spring method registration (no manual biomeOS config) | biomeOS v3.51 | Medium |
| Tier 4 rewiring | IPC-first defaults, `barracuda` optional, feature-gate imports, `CompositionContext` | JH-11 resolution | Medium |
| ~~skunkBat in smaller compositions~~ | ~~Add skunkBat node to `node_atomic_compute.toml` and `nest_atomic.toml`~~ | **DONE** (May 11) | ~~High~~ |
| skunkBat audit forwarding | Wire skunkBat `security.audit_log` → rhizoCrypt DAG + sweetGrass braid in deploy graphs | skunkBat Phase 2 + JH-5 Phase 3 | Medium |
| ~~MethodGate parity~~ | **RESOLVED** — squirrel shipped `method_gate.rs` (JH-0 + JH-2). 13/13 at primalSpring gate. | **DONE** (May 11) | ~~Low~~ |
| ~~foundation integration~~ | Wire provenance results to `sporeGarden/foundation/` — **Thread 5 LTEE + Thread 4 targets + Thread 10 provenance created; THREAD_INDEX hygiene done; paper 02 thread fix** | **DONE** (May 11) | ~~Low~~ |
| ~~systemd unit portability~~ | ~~Parameterize `/home/irongate` in systemd units~~ — **DONE** via `EnvironmentFile` + `gate.env.template` | **DONE** (May 11, deep debt sweep) | ~~Low~~ |
| ~~BearDog TLS shadow run (H2-12)~~ | ~~Configure BearDog TLS on port 8443, run `btsp_tls_parity.sh` hourly for 7 days~~ — **RUNNING** (BearDog v0.9.0 on :8443, 10ms RPC latency vs 120ms Cloudflare) | **LIVE** | ~~High (ops)~~ |
| Songbird NAT VPS relay (H2-14) | Provision ~$5/mo VPS, deploy BearDog-authenticated STUN/TURN relay — `deploy_songbird_relay.sh` staged | songbird Wave 202 OPS-READY | Medium (ops) |
| plasmidBin binary workflow | Update workload TOMLs to support fetched binaries (plasmidBin `fetch.sh` → `$PLASMIDBIN_DIR/springs/`) | Springs shipping release binaries | Medium |
| Future horizons | Tor relay, QUIC multi-path, `cloudflared` orchestration, TURN refresh, Plasmodium | songbird/biomeOS — none blocked | Future |

---

## Interstadial / Stadial Phase Tagging

Interstadial exit criteria: `infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md`

### Interstadial Targets (pre-wire — shadow runs can begin)

| ID | Target | Pillar | Status |
|----|--------|--------|--------|
| H2-01→04 | BTSP auth dual-auth shadow run | P2 (NUCLEUS) | **BUILT** — `jupyterhub_btsp_auth.py` + `deploy_btsp_auth_shadow.sh`. Dual-auth mode with PAM fallback. Awaiting shadow run start |
| H2-05 | NestGate content pipeline | P2 (NUCLEUS) | **DONE** (Session 60) — `publish_sporeprint.sh` READY |
| H2-06→09 | petalTongue content serving + extracellular | P2 (NUCLEUS) | UNBLOCKED — NestGate `content.*` live |
| H2-12 | BearDog TLS shadow on :8443 | P1 (Primal) + P2 (NUCLEUS) | **RUNNING** — BearDog v0.9.0 live on :8443, baseline comparison captured |
| H2-14 | Songbird NAT VPS relay | P1 (Primal) + P2 (NUCLEUS) | **OPS-READY** — Wave 202, `deploy_songbird_relay.sh` + systemd + creds shipped |
| H2-17→20 | DoT sovereign DNS | P2 (NUCLEUS) | **BASELINE CAPTURED** — DoT ACTIVE (Cloudflare 1.0.0.1), 3-8ms latency, 10/10 success. Sovereign resolver pending |
| TIER-2 | Tier 2 Science API (toadstool.validate) | P2 (NUCLEUS) | **WIRED** — S250 implemented, 12 workload TOMLs with `toadstool-validate-v1` schema |
| SHADOW | composition.deploy.shadow | P2 (NUCLEUS) | **WIRED** — `shadow_deploy()` in `deploy_graph.sh`, biomeOS v3.53 |
| ABG-WCM | Thread 1 WCM compositions via provenance trio | P3 (ABG) | Graph capabilities reconciled (GAP-36 canonical names); provenance_pipeline.sh exercises dag.*/spine.*/braid.* |
| LITHO-INT | lithoSpore workload integration | P4 (lithoSpore) | **EXCEEDED** — 6/7 modules PASS Tier 2, litho-core shared library, liveSpore tracking |
| FND-10 | Foundation 10/10 threads active | P5 (Foundation) | **DONE** — all threads seeded, TOML-driven fetch |

### Stadial Targets (external validation drives cutover)

| ID | Target | External Driver | Status |
|----|--------|-----------------|--------|
| H2-12→cutover | BearDog TLS cutover from Cloudflare | Cloudflare baselines | Awaiting interstadial shadow |
| H2-14→cutover | Songbird NAT replaces cloudflared | VPS relay parity | Awaiting interstadial integration |
| H2-09 | petalTongue replaces Zola/GitHub Pages | Content parity | Awaiting interstadial pipeline |
| H3-01→06 | Primal-only replacements | GitHub → Forgejo, crates.io | Future |
| LITHO-EXT | lithoSpore USB to Barrick Lab | External deployment | Phase 5 (after spring reproductions) |
| UPSTREAM-CRATES | wgsl-precision, proc-sysinfo extraction | crates.io community | Q2 2026 |

### ABG Whole-Cell Model (WCM) Composition Milestone

Thread 1 WCM compositions must be exercised through NUCLEUS deploy graphs
with the provenance trio (rhizoCrypt + LoamSpine + sweetGrass) producing
verifiable output. This is Pillar 3 of the interstadial exit criteria.

Current state:
- `ABG_WHOLE_CELL_REBUILD.md` maps compositions but not all are exercised
- Provenance trio validated on active gate (Full NUCLEUS, 13 primals)
- Thread 1 workloads running (genome fetch/hash, KEGG)
- Remaining: drive WCM compositions through NUCLEUS `composition.deploy(graph)`

### lithoSpore Integration

lithoSpore (`sporeGarden/lithoSpore`) is a projectNUCLEUS subsystem — the
first Targeted GuideStone artifact.

- Workload TOMLs: `litho-validate-tier2.toml`, `litho-validate-tier3.toml`
- **Phase 2 active**: 6/7 modules PASS Tier 2 (Rust) — modules 1-4, 6, 7 live; module 5 (biobricks) awaits B6 spring data
- Shared `litho-core` library: `discovery`, `harness`, `stats` modules — cross-module reuse pattern
- Integration point: NUCLEUS dispatches lithoSpore workloads via `composition.deploy(graph)`
- USB deployment: Tier 3 deploy graph (`graphs/ltee_guidestone.toml`) composes
  NUCLEUS primals around lithoSpore science modules
- Interstadial target: 2+ modules PASS at Tier 1 (Python) with real data — **EXCEEDED** (6/7 modules Tier 2)
- Stadial target: USB to Barrick Lab (Phase 5) — GuideStone Phase 1 DONE (architecture + queue seeding)

---

## Scoring

```
Horizon 1 (external security):    ██████████  COMPLETE — all resolved, darkforest v0.2.1 authoritative
Horizon 2 (sovereignty):          ████████░░  2a done, 2b ready, 3a LIVE, 3b/3c shadow-staged + deploy scripts, 4 INTERMEDIATE (DoT baseline wired)
Horizon 3 (primal-only):          ██░░░░░░░░  H3-07/H3-08 UNBLOCKED (JH-11 + JH-5 resolved)
Upstream (waiting):                ██████████  ZERO OPEN — 13/13 primals, 8/8 springs at zero debt (May 13)
Interstadial exit:                ██████░░░░  BearDog TLS LIVE, provenance trio reconciled, lithoSpore 6/7 EXCEEDED. 3 items remain: Songbird relay, BTSP dual-auth, WCM through trio
```

---

## Changelog

| Date | Change |
|------|--------|
| 2026-05-08 | Initial spec. Phase 60 enforced. 3 horizons, 37 gaps tracked. |
| 2026-05-08 | H1-01→H1-03, H1-06, H1-07 resolved. Voila isolated (UID 998). Cron deduplicated. |
| 2026-05-08 | H1-04, H1-05 resolved. systemctl + python3 ACLs for ABG users. Voila home dir fix (500→200). Revalidation: **267 PASS, 0 FAIL**. Horizon 1 COMPLETE. |
| 2026-05-08 | Pure Rust `darkforest` validator created. Replaces bash+python pen/fuzz tools. 863KB binary, 159 PASS 0 FAIL 3 DARK_FOREST. Primal ecosystem validated in Rust. |
| 2026-05-08 | `darkforest` v0.2.0 — modular auditable framework. 7 source modules. 13 crypto strength checks (CRY-01→CRY-13). JSON report output (`--output json`). Per-check severity, evidence, remediation. **177 PASS, 0 FAIL, 4 DARK_FOREST**. Cloudflare config.yml fixed (664→600). |
| 2026-05-08 | Workspace scaffolding: pilot lifecycle, per-user scratch, reviewer showcase-only visibility, welcome notebooks, validation dashboard. Compute usability: per-user venvs, wheelhouse, offline pip. darkforest revalidation post-scaffolding: **175 PASS, 0 FAIL, 6 DARK_FOREST** (2 new: reviewer-visibility items detected at filesystem level, remediated via symlink isolation). |
| 2026-05-09 | Open observer landing: Voila public surface at `lab.primals.eco` (no credentials). Observer is default; reviewer/user gated by Cloudflare Access + PAM. Root redirect to Welcome.ipynb, source stripping, internal directory blocking, page titles on all notebooks. |
| 2026-05-09 | Multi-tier test suite: `tier_test_observer.py` (structural, execution, HTTP), `tier_test_reviewer.py` (access, parse, no-write), `tier_test_compute.py` (venv, packages, kernels, execution). `tier_test_all.sh` runner. Identified and fixed: kernel mismatches, missing metadata, relative path errors, dashboard KeyError, package import issues. |
| 2026-05-09 | `pappusCast` auto-propagation daemon: tiered validation (light/medium/heavy), adaptive rate limiting (scales with active users), snapshot architecture (copies not symlinks), quarantine for failures, changelog tracking. Python-first with Rust evolution path. |
| 2026-05-09 | `tunnelKeeper` Rust crate: programmatic Cloudflare tunnel health checks, DNS resolution fallback chain, config file parsing. Integrated into darkforest A6 pen test. First step toward Rust-native Cloudflare interaction. |
| 2026-05-09 | Dual-hosted primals.eco: Zola v0.22.1 builds sporePrint locally, served on port 8880 via `sporeprint-local.service`. Tunnel CNAME added to `~/.cloudflared/config.yml`. `sporeprint_dns.sh` manages DNS routing via Cloudflare API (sovereign/external switching). `sporeprint_verify.sh` checks both origins. H2-07 DONE. |
| 2026-05-09 | DNS metadata leak closed: `/etc/systemd/resolved.conf` switched to DNS-over-TLS (1.1.1.1 primary, 9.9.9.9 fallback). ISP resolver bypassed. Fixes `.eco` TLD resolution. H2 Step 4 INTERMEDIATE. |
| 2026-05-09 | ISP negative cache issue: AT&T resolver cached NXDOMAIN during A→CNAME gap, causing LAN devices (still on ISP DNS) to fail resolving primals.eco. Gate unaffected (DoT). Lesson: `sporeprint_dns.sh` must avoid delete-then-create gaps — update atomically. LAN-wide fix: change AT&T gateway DNS to 1.1.1.1. |
| 2026-05-09 | **Deep debt sweep**: `nucleus_config.sh` centralized with `$GATE_HOME` indirection (all `/home/irongate` hardcoding removed from deploy scripts). `nucleus_paths.py` for Python. tunnelKeeper: `Client::new()` returns `Result`, tokio slimmed, `rand`→`rand_core`, `CLOUDFLARED_DIR` env-var-driven. darkforest: PRIMALS array env-var-driven with compiled fallback, rhizoCrypt RPC 9602 added, crypto/pentest paths gate-agnostic. `security_validation.sh` invokes Rust darkforest (replaces archived bash/python). `pappusCast.py` exception types narrowed. 96 ironGate display references scrubbed across 23 docs. Zero TODO/FIXME/HACK. |
| 2026-05-10 | Static observer surface: Voila replaced with pre-rendered HTML via pappusCast + `observer_server.py` on port 8866. Centralized dark theme (`observer_theme.css`). Navigation bar injected. Voila link rewriting. `observer-static.service` replaces `voila-public.service`. |
| 2026-05-10 | darkforest v0.2.1: `observer.rs` module — 9 check groups (OBS-01→OBS-09) for static surface validation: theme CSS, nav bar, Voila link remnants, tracebacks, source stripping, HTTP 200 root, security headers, directory blocking. 86 PASS, 0 FAIL. Observer tier test migrated from Python to Rust in `tier_test_all.sh`. |
| 2026-05-10 | **Post-interstadial gap closure**: All 11 upstream gaps resolved by primal teams (JH-11, GAP-03/06/09/12, U1-U3, DF-2/3, U5). H2-10/11 (bearDog TLS + rate limiting) shipped. H2-13/15/16 (songbird NAT chain) shipped. H3-07/H3-08 unblocked. 6 absorption targets identified for local wiring. |
| 2026-05-10 | **Cell membrane architecture**: Architectural inversion — `primals.eco` permanently on GitHub Pages CDN (extracellular), `lab/git.primals.eco` via tunnel replicas (membrane), sovereign compute inside (intracellular). `gate_provision.sh` provisions replicas. `gate_watchdog.sh` monitors membrane health. `tunnelKeeper v0.2.0` reports replica count + edge colos. `sporeprint-local.service` demoted to dev. Key insight: accept uncontrolled extracellular, total control intracellular, selective permeability at the membrane. |
| 2026-05-11 | **Cross-ecosystem audit**: MethodGate corrected to 12/13 (squirrel pending; toadStool has full JH-0 + JH-2 method_gate.rs). skunkBat added to `node_atomic_compute.toml`, `nest_atomic.toml`, and `deploy.sh` composition lists. `deploy.sh` confirmed as nohup-loop, not `composition.deploy` — graph-driven germination is key absorption target. foundation integration is docs-only (no code path). 4 new absorption targets added. Spring readiness: all 7 workloads gate-agnostic, barraCuda version skew across springs (0.3.7–0.3.13), airSpring `barracuda` not optional. |
| 2026-05-11 | **River delta Push 2 verified**: 8/8 springs confirmed skunkBat Rust IPC (airSpring, groundSpring, hotSpring, wetSpring all added `skunkbat.rs`), 8/8 `method.register`, 8/8 CI cross-sync 413. 12,900+ tests. 3 new healthSpring workloads pulled. BearDog TLS shadow run (H2-12) and Songbird NAT VPS relay (H2-14) added as operational targets. plasmidBin binary workflow target added. airSpring `barracuda` still required (Tier 4 pending). |
| 2026-05-11 | **Interstadial exit criteria**: 5 pillars defined (Primal Sovereignty, NUCLEUS Deployments, ABG Hosting, lithoSpore, River Delta). H2/H3 items tagged interstadial vs stadial. ABG WCM composition milestone added. lithoSpore integration section (workload TOMLs, deploy graph, Phase 2 dependency). Stadial boundary: external validation drives cutover. |
| 2026-05-11 | **MethodGate 13/13 + foundation integration**: toadStool + squirrel resolved upstream. Deep debt evolution sweep committed. foundation Thread 5 LTEE + Thread 4 targets + Thread 10 provenance created. THREAD_INDEX v1.2.0 hygiene. publish_sporeprint.sh stub created for H2-05. 3 absorption targets closed. |
| 2026-05-11 | **Stadial-ready — NestGate Session 60 + full debt resolution**: NestGate shipped `content.*` transport parity (8 methods, 4 transports). H2-05 DONE. H2-06/08/09 UNBLOCKED. H3-02/05 UNBLOCKED. All per-primal debt closed (toadStool env contract, squirrel RemoteComputeProvider, barraCuda crypto→bearDog IPC, loamSpine method aliases, skunkBat JH-5 Phase 3, petalTongue SPA+CORS). L1 CLEAN: 13/13 structural+semantic. primalSpring Wave 7-9: 413 methods, 301 exercised. Compute trio (toadStool/coralReef/barraCuda) evolving HOW/WHERE/WHAT in parallel — doesn't block us. |
| 2026-05-11 | **lithoSpore Tier 1 PASS + composition.deploy + shadow prep**: lithoSpore modules 1+2 Tier 1 Python baselines ported from groundSpring B2/B1 (8/8 + 7/7 PASS). Fetch scripts created (`fetch_wiser_2013.sh`, `fetch_barrick_2009.sh`). Expected values from groundSpring cross-validated. Rust crates wired to dispatch Python Tier 1. `ltee-cli validate` dispatches live modules. `composition.deploy(graph)` absorbed: `deploy_graph.sh` reads graph TOML, starts primals in dependency order with health checks (`--graph-deploy` flag in `deploy.sh`). Shadow run orchestrator created (`shadow_run_orchestrator.sh`) — ties NestGate content, BearDog TLS, Songbird NAT parity tests together. Pillar 4 interstadial exit gate MET (2+ modules PASS at Tier 1). |
| 2026-05-11 | **Deep debt evolution sweep COMPLETE (all three products)**: foundation: THREAD_INDEX v1.3.0 fixed (4 threads), 7 workload TOMLs gate-agnostic, `fetch_sources.sh` covers 10/10 threads via TOML manifests, `foundation_validate.sh` Phase 6 target comparison added. projectNUCLEUS: zero hardcoded paths, `deploy.sh` modularized (`deploy_primal_start.sh` + `deploy_health_check.sh`), darkforest `pentest.rs` split into 3 submodules, `crypto.rs` split into 3 submodules, tunnelKeeper 9/11 `.clone()` eliminated. lithoSpore: Tier 2 Rust wired for modules 1+2 (Nelder-Mead curve fitting, Kimura fixation, Poisson accumulation), `cmd_refresh` stub evolved to real implementation, 9 `expect()` calls replaced with `match`, first `liveSpore.json` entry seeded. All stale MethodGate references reconciled to 13/13. `publish_sporeprint.sh` status STUB→READY. |
| 2026-05-12 | **May 12 upstream absorption — zero sentinel blockers**: `toadstool.validate` IMPLEMENTED (S250) — Tier 2 Science API unblocked. `barracuda.precision.route` IMPLEMENTED (649 tests). Songbird VPS relay OPS-READY (Wave 202) — TURN server + CLI + systemd + credentials + deployment guide shipped. coralReef Sprint 7 FECS stability proof (4,790 tests). plasmidBin pipeline hardened: auto-harvest + BLAKE3 post-validate + idempotent. Local `specs/LIVE_SCIENCE_API.md` rewritten to match upstream contract (`workload_path`/`dry_run` params, not old `workload`/`format`). Shadow deploy scripts created: `deploy_beardog_tls_shadow.sh` (H2-12), `deploy_songbird_relay.sh` (H2-14). Interstadial targets table updated: H2-05 DONE, H2-14 OPS-READY, TIER-2 UNBLOCKED, FND-10 DONE. `fetch_primals.sh` updated to verify BLAKE3 checksums from plasmidBin `checksums.toml`. |
| 2026-05-13 | **Deep debt resolution — hardcoded path elimination + doc drift fix**: 7 workload TOMLs (airspring x6, groundspring x1) evolved from `${SPRINGS_ROOT:-/home/eastgate/...}` fallbacks to bare `$SPRINGS_ROOT` (matching all other workloads). `deploy_songbird_relay.sh` `SONGBIRD_SRC` evolved from `/home/irongate/...` to `${ECOPRIMALS_ROOT}` pattern. foundation `specs/EVOLUTION_GAPS.md` (194L stale copy, May 09) replaced with cross-reference to projectNUCLEUS canonical. Upstream flag: barraCuda `registry_tests.rs` asserts 71 methods but `REGISTERED_METHODS` has 72 after Sprint 58 `precision.route` — noted in `PRIMAL_DEEP_DEBT_HANDBACK.md` addendum. |
| 2026-05-13 | **Atomic deployment phase absorption**: `composition.deploy.shadow` wired — `shadow_deploy()` in `deploy_graph.sh` does dry-run graph validation (binary existence, port conflicts, dependency ordering, toadstool.validate pre-flight). 12 validation workload TOMLs across all 7 springs wired with `[output] schema = "toadstool-validate-v1"` + `--format json` for Tier 2 notebook integration. `dot_sovereign_parity.sh` created — DNS-over-TLS vs sovereign resolver timing/accuracy comparison (H2-4/H2-17→20), integrated into `shadow_run_orchestrator.sh`. Wire notes added to `LIVE_SCIENCE_API.md`: bearDog base64 signing, skunkBat `security.audit_log` path, NestGate `content.*` vs `storage.*` domain separation, BTSP auth pipeline, composition.deploy.shadow. River delta Push 3 absorbed. Interstadial exit score advanced. |
| 2026-05-13 | **Shadow run execution + baseline fixes**: (1) DoT baseline fixed — replaced `dig` (not installed) with `resolvectl query` fallback chain, DoT detection regex fixed to match `+DNSOverTLS` from `resolvectl status`. Result: 10/10 success, 3-8ms latency, DoT ACTIVE via Cloudflare 1.0.0.1. (2) Tunnel baseline uptime calculation fixed — added `tunnel_reachable_pct` (100%, TLS connected) alongside `uptime_pct` (0%, service not running). (3) BearDog TLS shadow **LIVE** on :8443 (v0.9.0, 200+ methods, BTSP v2.0) — updated `deploy_beardog_tls_shadow.sh` to use `--listen`/`--family-id`/`--audit-dir` (not `--tls-cert`/`--tls-key`). 10ms RPC latency vs 120ms Cloudflare. (4) Fixed `nucleus_config.sh` source path in 6 benchScale scripts. (5) `parse_graph_nodes` extended for `fragment.nodes`. (6) Proposed API methods resolved: `biomeos.spring_status` → use `capabilities.list`, `nestgate.artifact_query` → use `content.get`/`content.resolve`, `rhizocrypt.dag_summary` → use `dag.session.get`. |
| 2026-05-13 | **Interstadial exit execution — provenance trio + lithoSpore 7/7 + API absorption**: (1) Provenance trio graph capabilities reconciled — `nucleus_complete.toml` loamSpine capabilities updated from stale `session.commit`/`entry.append` to canonical `spine.create`/`spine.get`/`spine.seal`/`entry.append`/`entry.get`/`certificate.mint` (GAP-36 resolved). sweetGrass capabilities updated from `provenance.create_braid`/`provenance.lineage`/`provenance.graph` to canonical `braid.create`/`braid.commit`/`braid.get`/`anchoring.anchor`/`anchoring.verify`. `rootpulse_commit.toml` workflow methods reconciled: `dag.dehydrate`→`dag.dehydration.trigger`, `commit.session`→`spine.seal`, `provenance.create_braid`→`braid.create`. (2) lithoSpore module 7 (`ltee-anderson`) activated — `artifact/data/anderson_predictions/` created with provenance README. 5/5 PASS Tier 2 (no plateau, diminishing returns, GOE/Poisson, variance, 12 populations). Full suite: 6/7 PASS, 1 SKIP (biobricks awaits B6 data). (3) `biomeos.spring_status` absorbed as IMPLEMENTED (v3.54). Registry updated to 415 methods. (4) Upstream absorbed: litho-core shared library (discovery, harness, stats), plasmidBin checksum refresh, rhizoCrypt GAP-36 aliases, biomeOS routing updates. |
