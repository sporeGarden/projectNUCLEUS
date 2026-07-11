# Evolution Gaps — projectNUCLEUS

Living tracker of remaining gaps across three horizons. Updated as gaps
close and new ones emerge. Each gap is local — actionable by projectNUCLEUS
without waiting on upstream unless noted.

**Last updated**: 2026-07-10 (Wave 136a — HARDENING. Warming event contained. EXP-01/02/03/04 PATCHED — security headers, 404 fix, cert auto-renewal confirmed, fail2ban active. Criterion 8: 3/5 met. darkforest v3.0 shipped (146 tests). Remaining 136b-d sprint: CSP, depot rate-limiting, signing, resilience drills, monitoring. primalSpring v0.9.33 128 scenarios / 1101 tests / KNOWN_DEBT=0.)
**Validation baseline**: 267 PASS, 0 FAIL, 0 KNOWN_GAP (bash 5-layer) + 33 PASS Dark Forest gate. **253 Rust tests** (darkforest 146, tunnelKeeper 48, nucleus-deploy 47, nucleus-primals 12), coverage: darkforest 40.77%, tunnelKeeper 52.67%
**Rust validator**: darkforest v3.0 — 14 modules (8 inner + 6 outer), `--scope inner|outer|full`, `--suite observer` static surface validation
**Dark Forest Gate**: 5-pillar structural validation PASS (`validation/dark_forest_gate_local.sh`)
**Multi-tier tests**: observer (darkforest Rust) + reviewer + compute + hub + pappusCast + sporePrint (`deploy/tier_test_all.sh`)
**Architecture**: Cell membrane model — primals.eco on GitHub Pages CDN (extracellular), lab/git.primals.eco via tunnel (membrane), cellMembrane fieldMouse on DigitalOcean VPS (external membrane), sovereign compute inside
**Upstream status**: All upstream gaps resolved. 13/13 primals zero code debt, 8/8 delta springs zero debt (8,486+ tests). Tower atomic LIVE (ludoSpring 6/6). Nest atomic ready (GAP-36 resolved). Node atomic AMD live, NV FECS-gated.
**Tier 2 Science API**: `toadstool.validate` IMPLEMENTED (S325, 112 methods, 9,074 tests), `barracuda.precision.route` v0.4.0 (649 tests), biomeOS v4.31 (8,351 tests, 88% coverage). Registry at **502+ methods** (Wave 136a). Pepti 100% — 34/34 builds, 0 failures, 16 binaries × 2 triples. WAN-DISPATCH-01 transport PASS (10/10, 142ms p50). DNS cutover COMPLETE — site LIVE + HARDENED (EXP-01/02/03/04 patched). darkforest v3.0 outer membrane scope shipped (26 checks, 146 tests). Cooling sprint 136b-d in progress. primalSpring v0.9.33 — 128 scenarios, 1101 tests, KNOWN_DEBT=0.
**cellMembrane**: fieldMouse deployment LIVE on 157.230.3.183 (DigitalOcean nyc1, **$12/mo 2GB RAM**). **Nest Atomic composition (Wave 38)**: Tower (BearDog :9100, SkunkBat :9140, Songbird :3478) + NestGate :9500 + rhizoCrypt :9602 + loamSpine :9700 + sweetGrass :9850 + RustDesk :21115-17 + Caddy TLS :80/:443 + petalTongue :8080 + BearDog TLS shadow :8443. **11 services, 7 primals**. Provenance trio pipeline verified end-to-end (10/10 PASS). darkforest membrane 21 PASS, 0 FAIL, 1 SKIP. Shadow 6/0/0 FULL PASS (S1-S5 + DNS). DO token encrypted (BearDog AES-256-GCM). 1.6 GB free. Owned by ironGate/projectNUCLEUS.
**Forgejo**: PRIMARY git host — 39 repos across 3 orgs (sporeGarden, ecoPrimals, syntheticChemistry). K-Derm diderm relay: gate → golgiBody (cis) → peptidoglycan (sync) → golgiBody-ext (trans) → GitHub. Push to forgejo only — relay handles GitHub automatically.

Related specs:
- [TUNNEL_EVOLUTION.md](TUNNEL_EVOLUTION.md) — sovereignty replacement roadmap
- [SECURITY_VALIDATION.md](SECURITY_VALIDATION.md) — five-layer validation model
- [SOVEREIGNTY_VALIDATION_PROTOCOL.md](SOVEREIGNTY_VALIDATION_PROTOCOL.md) — replacement methodology
- [COMPLETE_DEPENDENCY_INVENTORY.md](COMPLETE_DEPENDENCY_INVENTORY.md) — full dependency map
- `infra/wateringHole/REPO_MEMBRANE_BOUNDARY.md` — inner/outer membrane repo classification

**Rust evolution**: `validation/darkforest/` v3.0 — modular auditable security framework (zero
runtime deps). 14 source modules: **inner** (8): `check.rs` (structured types + env-var-driven primal config),
`net.rs` (TCP/HTTP/UDP helpers), `pentest.rs` (3 threat actors), `fuzz.rs` (14 primals + JupyterHub),
`crypto.rs` (13 crypto strength checks, gate-agnostic paths), `observer.rs` (static HTML quality),
`report.rs` (pipe + JSON output), `main.rs` (CLI + runner with `--scope inner|outer|full`).
**outer** (6): `outer/tls.rs` (OTR-01→06: cipher audit, HSTS, cert chain, protocol downgrade),
`outer/http.rs` (OHT-01→06: security headers, 404 behavior, verb fuzz, path traversal, directory listing, clickjacking),
`outer/depot.rs` (ODP-01→04: write rejection, checksums, enumeration),
`outer/forge.rs` (OFG-01→04: SSH handshake, auth bypass, repo enumeration),
`outer/dns.rs` (ODN-01→03: zone transfer, DNSSEC, NXDOMAIN behavior),
`outer/mesh.rs` (OMS-01→03: WireGuard probe, invalid handshake, mesh surface).
All ports and paths resolve from environment variables with compiled defaults — zero hardcoded gate paths.

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
| H1-07 | Baseline 7-day summary | 2026-05-08 | Cron capturing hourly. 9-day summary generated. Subsumed by `membrane_summary.sh` |
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

**Status**: CELL MEMBRANE LIVE + CONTENT PARITY. `primals.eco` permanently on GitHub Pages CDN (extracellular).
`lab/git.primals.eco` via tunnel replicas (membrane). VPS NestGate cache serving real sporePrint (19MB synced).
`membrane.primals.eco` TLS endpoint live (ACME cert, Let's Encrypt E8). HTTP parity: VPS 68ms TTFB vs GitHub Pages 89ms (**PASS**).
Full primal path (NestGate + petalTongue) remains Phase 3 target for sovereign extracellular rendering.

**Cell membrane architecture (operational May 10, 2026)**:
- `primals.eco` DNS permanently set to GitHub Pages A records (extracellular)
- `lab.primals.eco` + `git.primals.eco` via Cloudflare tunnel replica pool (membrane)
- `nucleus-deploy provision` provisions new membrane replicas
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
| H2-05 | ~~Build NestGate content pipeline~~ | **DONE** | NestGate Session 60: `content.put/get/exists/list/publish/resolve/promote/collections` on all 4 transports. `publish_sporeprint.sh` structurally complete but **blocked on BTSP scope**: BearDog `auth.issue_session` returns fixed default scopes (`crypto.*`, `health.*`, `capabilities.*`, `identity.*`) — does not include `content.*`. NestGate MethodGate rejects. **Upstream ask**: BearDog scope expansion for pipeline tokens (SP-4 blocker). |
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
| H2-12 | ~~Shadow run on port 8443 alongside Cloudflare 443~~ | — | **RUNNING** — BearDog v0.9.0 on :8443 (PID live), **3ms RPC** vs 120ms Cloudflare (40x). Telemetry probe fixed (`/dev/tcp` + `read -t 1`). `btsp_tls_parity.sh` ready for 7-day comparison |

### Step 3c: Songbird NAT (replaces cloudflared)

**Status**: **cellMembrane LIVE + HTTP parity PASS** — Channel 2 (Songbird TURN relay) active on 157.230.3.183:3478.
VPS hardened (fail2ban, UFW 22+3478 only, exim4 purged). HTTP parity: VPS 68ms vs GitHub Pages 89ms (**PASS**).
TLS parity via `membrane.primals.eco`: 130ms TTFB (within 35% of CDN — expected for single VPS vs global CDN).

songbird Wave 196-197 shipped full NAT traversal chain: STUN wire-compliant (RFC 5389),
RFC 5766 TURN client, Cloudflare DDNS, 5-tier `ConnectionFallbackChain`.
primalSpring deployed and hardened the VPS (cellMembrane fieldMouse). Ownership
transferred to ironGate/projectNUCLEUS on May 14. Local work: NAT shadow run
validation, Tower composition deployment, Channel 1+3 when ready.

| ID | Gap | Effort | Notes |
|----|-----|--------|-------|
| H2-13 | ~~Songbird STUN client~~ | — | **RESOLVED** — songbird Wave 196: STUN wire-compliant (RFC 5389) |
| H2-14 | ~~Self-hosted STUN/TURN VPS relay~~ | — | **LIVE (Tower)** — cellMembrane fieldMouse on 157.230.3.183 (DigitalOcean nyc1, ~$12/mo 2GB). 6 services: Songbird TURN :3478 + RustDesk :21115-17 + BearDog :9100 + SkunkBat :9140 + Caddy :80. DO token encrypted. Ops: `plasmidBin/deploy_membrane.sh`. Private ops repo: `sporeGarden/cellMembrane` |
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
| H3-04 | GitHub repos (source hosting) | Forgejo primary, GitHub mirror | **ACTIVE** — 39 repos, K-Derm diderm relay (push forgejo only, relay → GitHub) | CI still on GitHub Actions (H3-03 open) |
| H3-05 | Docker Hub / ghcr.io | NestGate OCI blob store | **UNBLOCKED** — NestGate `content.put` shipped. Wire toadStool config. | Local: OCI store wiring |
| H3-06 | Anthropic / OpenAI | Ollama + barraCuda WGSL inference | Partial — Ollama works locally | barraCuda shader maturity |
| H3-07 | JH-11 cross-primal token federation | biomeOS composition forwarding with `_resource_envelope` | **UNBLOCKED** — bearDog `auth.public_key` + biomeOS `BearDogVerifier` live | Local: wire `CompositionContext` |
| H3-08 | JH-5 cross-primal audit forwarding | skunkBat → rhizoCrypt DAG + sweetGrass braids | **UNBLOCKED** — skunkBat Phase 2 complete (7 event kinds) | Local: wire into deploy graphs |
| H3-09 | conda/pip/crates.io | Vendored deps, private registry | Low priority — offline modes exist | Not blocking |
| H3-10 | NCBI / UniProt / KEGG | Local mirror + `abg_data.sh` provenance | Partial — data registry operational | Not blocking (data, not service) |
| H3-11 | FlockGate cross-WAN deployment | Songbird TURN + cellMembrane relay + covalent mesh | **DESIGNED** — `gates/flockgate.toml` manifest exists, `basement_hpc_covalent.toml` graph ready. cellMembrane TURN relay LIVE. Needs: FlockGate NUCLEUS deploy, cross-gate routing via TURN, BTSP covalent authentication. | cellMembrane TURN + FlockGate hardware provisioning |

### Niche Climate Gaps (Wave 56 — pre-stadial)

**Context**: primalSpring `NICHE_CLIMATE_EVOLUTION.md` defines NC-1→NC-5 as the path to stadial entry.
projectNUCLEUS is the ironGate sovereign deployment validator. NC-2 (multi-gate mesh) and NC-3
(cellMembrane sovereignty) are directly owned. NC-1 (spore gateway) blocks us indirectly.

| ID | Gap | Owner | Status | Blocks |
|----|-----|-------|--------|--------|
| NC-1 | postPrimordial spore gateway (`biomeos nucleus ingest/emit`) | biomeOS + lithoSpore | **CODE COMPLETE** — biomeOS v3.84 shipped `biomeos-pseudospore` + emit materialization. NC-1.3 COMPLETE. NC-1.4 RESOLVED. Live column U gated on VPS deploy (P0) + 2 spring column U passes. | Stadial entry; columns U/V/W |
| NC-2.1 | southGate 13/13 health | Songbird + wetSpring | **IN PROGRESS** — 7/13 responding. Wave 54 redeploy fixes ready. Songbird mesh seed bug identified. | NC-2 multi-gate mesh |
| NC-2.3 | Cross-gate capability call via cellMembrane relay | projectNUCLEUS + cellMembrane | **OPEN** — ironGate VPS TURN LIVE, mesh not yet bidirectional | NC-2 stadial |
| NC-3.1 | cellMembrane Nest Atomic docs sync | cellMembrane | **DONE** — VPS_STATE.md/GLACIAL_SHIFT synced to Nest Atomic (deployed May 22) | Unblocks NC-3.2 |
| NC-3.2 | `membrane.toml` → `composition = "nest"` | cellMembrane | **DONE** — composition = "nest" published, signal channel enabled | Unblocks `s_kderm_boundary` |
| NC-3.3 | knot-dns NS cutover | cellMembrane + registrar | **OPEN** — knot-dns DEPLOYED (H2-17), NS record cutover pending registrar action | Sovereignty DNS (S5) |
| NC-3.4 | Forgejo Releases (sovereign binary channel) | cellMembrane + plasmidBin | **OPEN** — coordinate `auto-harvest.yml` for Forgejo | H3-02 parallel |
| NC-3.5 | sporePrint living content via NestGate | cellMembrane + bearDog | **BLOCKED** — needs bearDog `content.*` scope on BTSP tokens (SP-4) | Sovereign content hosting |

### cellMembrane Security Gaps (darkforest coverage — Layer 6)

**Status**: `validation/darkforest_membrane.sh` implements MEM-01 through MEM-17 (21 PASS, 0 FAIL, 1 SKIP — Nest Atomic checks MEM-14→17 added Wave 38). MEM-09 needs `b3sum` on VPS.

| ID | Gap | Status | Notes |
|----|-----|--------|-------|
| MEM-01 | darkforest `--suite membrane` module | **DONE** | `validation/darkforest_membrane.sh` — bash implementation (Rust module deferred) |
| MEM-02 | SSH password auth disabled | **PASS** | `ssh -o PreferredAuthentications=password` correctly rejected |
| MEM-03 | fail2ban sshd jail active | **PASS** | Active, 3 total bans |
| MEM-04 | UFW firewall posture (22+3478) | **PASS** | Correct deny-default + targeted allows |
| MEM-05 | TURN unauthenticated rejection | **PASS** | 0 bytes returned to unauthenticated allocate |
| MEM-06 | No unnecessary services | **PASS** | exim4, droplet-agent, snapd all absent |
| MEM-07 | journald persistence | **PASS** | `/var/log/journal/` present |
| MEM-08 | Credential file permissions | **PASS** | 600, root-owned |
| MEM-09 | Songbird binary integrity (BLAKE3) | **SKIP** | `b3sum` not installed on VPS |
| MEM-10 | No unexpected listeners | **PASS** | All listeners accounted for |
| MEM-11 | RustDesk hbbs/hbbr active | **PASS** | Both systemd services active |
| MEM-12 | RustDesk relay key present | **PASS** | `/opt/membrane/rustdesk/id_ed25519.pub` present |
| MEM-13 | RustDesk :21116 reachable | **PASS** | TCP probe succeeded |
| MEM-14 | BearDog TLS :8443 in fuzz/crypto suites | Open | Future: add to `DEFAULT_PRIMALS` or shadow check |
| MEM-15 | Credential-at-rest encryption | **DONE** | BearDog AES-256-GCM with Argon2id KDF — DO token encrypted at `/opt/membrane/credentials.age` |

### Irreducible Externals (never sovereign)

These are not gaps — they are accepted constraints:

- Domain registrar (`primals.eco`)
- Linux kernel / systemd
- NVIDIA GPU drivers
- Let's Encrypt / ACME (browser trust chain)
- VPS for membrane channels (~$12/mo, DigitalOcean — cellMembrane fieldMouse, Tower composition)

---

## Upstream Dependencies (primal teams)

**River delta Push 3+ (May 13→27, 2026)**: Zero open upstream gaps. 13/13 primals at zero
code debt. 8/8 delta springs at zero debt, 8,486+ tests. Tower atomic LIVE (ludoSpring 6/6).
Nest atomic ready. Node atomic AMD live, NV FECS-gated. `toadstool.validate` S250 (74 methods).
`barracuda.precision.route` v0.4.0 (649 tests). `composition.deploy.shadow` biomeOS v3.53+.
`biomeos.spring_status` IMPLEMENTED (v3.84). Registry at **460 methods** (Wave 56, was 458 at Wave 46, 445 at Wave 20, 427 at Wave 12). `primal.list` / `capability.list` canonical schemas SHIPPED. `nucleus.ingest_spore` + `nucleus.emit_spore` registered. **Wave 56**: `--uds-only` VPS standard, cell graph `vps_standard` tagging, 12 primordial scripts archived, `primalspring checksums` + `primalspring registry` subcommands.
`content.put/get` NestGate Session 60 (4-surface parity). BTSP auth pipeline live (13/13 primals).
skunkBat audit pipeline JH-5 Phase 3 operational. Tier 2 JSON-RPC on all 7 springs (`--format json`).
76 wire routing misroutes fixed — `security.audit_log` → skunkBat, crypto methods base64-encoded,
`provenance.*` → sweetGrass. `s_routing_consistency` scenario prevents drift.

Post-interstadial downstream handoff delivered May 10, 2026 (upstream wateringHole; see interstadial exit criteria in `infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md`).

### All Resolved

| ID | What | Resolved by | When |
|----|------|-------------|------|
| JH-11 | Cross-primal token federation | bearDog Wave 99 `auth.public_key` + biomeOS v3.51 `BearDogVerifier` | May 10 |
| GAP-03 | biomeOS cell graph live deploy | biomeOS v3.51 `composition.deploy` route alias | May 10 |
| GAP-06 | rhizoCrypt no UDS transport | rhizoCrypt S66 — operational since S23, integration test added | May 10 |
| GAP-09 | biomeOS Neural API registration | biomeOS v3.51 `method.register` endpoint | May 10 |
| GAP-12 | ludoSpring IPC method registration | 28 `game.*` methods registered (427 canonical, zero drift) | May 13 |
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
| ~~`composition.deploy.shadow`~~ | `shadow_deploy()` in `deploy_graph.sh` — dry-run + `--live` mode (12/12 LIVE validated). toadstool.validate pre-flight. | biomeOS v3.53 | **WIRED** (May 13, live mode May 15) |
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
| ~~Songbird NAT VPS relay (H2-14)~~ | ~~Provision VPS, deploy STUN/TURN relay~~ — **LIVE** cellMembrane fieldMouse on 157.230.3.183:3478 (DigitalOcean nyc1). Ops via `plasmidBin/deploy_membrane.sh`. Private repo: `sporeGarden/cellMembrane` | **LIVE** (May 14) | ~~Medium (ops)~~ |
| plasmidBin binary workflow | Update workload TOMLs to support fetched binaries (plasmidBin `fetch.sh` → `$PLASMIDBIN_DIR/springs/`) | Springs shipping release binaries | Medium |
| Future horizons | Tor relay, QUIC multi-path, `cloudflared` orchestration, TURN refresh, Plasmodium | songbird/biomeOS — none blocked | Future |

---

## Validation Gate Matrix — Sovereignty Phase Transitions

Each sovereignty transition is gated by a specific validation system.
No cutover proceeds without a PASS from the corresponding gate.

### Validation Systems Inventory

| System | Location | Scope | Output |
|--------|----------|-------|--------|
| `nucleus-deploy security` | `deploy/nucleus-deploy/` | Multi-layer security sweep (L1–L6) | PASS/FAIL per layer |
| `darkforest` (Rust) | `validation/darkforest/` | 34 modular pentest checks, crypto, observer | JSON report, per-check severity |
| `dark_forest_gate_local.sh` | `validation/` | 33 structural checks across 5 pillars for deploy graphs | PASS/FAIL per pillar |
| `darkforest_membrane.sh` | `validation/` | MEM-01 through MEM-13 remote VPS validation | PASS/FAIL/SKIP per MEM-ID |
| `nucleus-deploy telemetry` | `deploy/nucleus-deploy/` | Unified probe: Caddy, TURN, BearDog TLS, SSH, Cloudflare, primal RPC, TTFB (was `membrane_telemetry.sh` in `deploy/legacy/`) | CSV + daily append |
| `nucleus-deploy summary` | `deploy/nucleus-deploy/` | Rolling 7-day `membrane_7day.toml` with parity checks and cutover gates (was `membrane_summary.sh` in `deploy/legacy/`) | TOML report |
| `tier_test_all.sh` | `deploy/` | Multi-tier user validation: observer, reviewer, compute, hub | Per-tier PASS/FAIL |
| `btsp_tls_parity.sh` | `deploy/` | BearDog TLS vs Cloudflare TLS latency/availability comparison | Parity metrics |
| `songbird_nat_parity.sh` | `deploy/` | Songbird TURN relay vs cloudflared NAT comparison | Reachability + latency |
| `dot_sovereign_parity.sh` | `deploy/` | DoT sovereign resolver vs Cloudflare DNS comparison | Timing + accuracy |
| `shadow_run_orchestrator.sh` | `deploy/` | Unified shadow run across all sovereignty channels | Composite TOML |

### Phase Transition → Validation Gate Mapping

| Phase Transition | Pre-Transition Gate | During Shadow | Cutover Gate | Post-Cutover Verify |
|-----------------|---------------------|---------------|--------------|---------------------|
| **H2-01→04: BTSP Auth (replaces PAM)** | `tier_test_all.sh` baseline (all tiers PASS with PAM) | `nucleus-deploy telemetry` BTSP auth events, dual-auth success rate logging (was `membrane_telemetry.sh`) | BTSP success ≥ 99.9%, latency < 50ms overhead, 7-day clean shadow | `tier_test_all.sh` re-run (all tiers PASS with BTSP-only) |
| **H2-05→09: petalTongue Content (replaces GitHub Pages)** | `nucleus-deploy telemetry` content TTFB baseline, `nucleus-deploy summary` parity snapshot (was `membrane_telemetry.sh` / `membrane_summary.sh`) | `nucleus-deploy telemetry` NestGate TTFB vs GitHub Pages TTFB | Content parity 100%, TTFB within 10% of Zola/Pages | `darkforest --suite observer` (OBS-01→OBS-09 against sovereign surface) |
| **H2-10→12: BearDog TLS (replaces Cloudflare TLS)** | `btsp_tls_parity.sh` baseline capture (7 days) | `nucleus-deploy telemetry` BearDog :8443 latency, `nucleus-deploy summary` daily parity (was `membrane_telemetry.sh` / `membrane_summary.sh`) | `nucleus-deploy summary` cutover gate MET (latency + availability parity) | `nucleus-deploy security` full sweep, `darkforest` crypto suite (CRY-01→CRY-13) |
| **H2-13→16: Songbird NAT (replaces cloudflared)** | `songbird_nat_parity.sh` baseline, `darkforest_membrane.sh` MEM-01→MEM-13 | `nucleus-deploy telemetry` TURN reachability, `nucleus-deploy summary` parity (was `membrane_telemetry.sh` / `membrane_summary.sh`) | `darkforest_membrane.sh` full PASS, `songbird_nat_parity.sh` parity MET | `nucleus-deploy security` Layer 6 re-sweep |
| **H2-17→20: Sovereign DNS (replaces Cloudflare NS)** | `dot_sovereign_parity.sh` DoT baseline (accuracy + timing) | `dot_sovereign_parity.sh` sovereign vs Cloudflare daily | 10/10 accuracy, latency within 20% of DoT, 7-day clean | `nucleus-deploy security` full sweep after each NS change |
| **H3-03: Forgejo Actions (replaces GitHub Actions)** | Port `notify-sporeprint.yml` first (smallest CI surface) | Manual comparison: Forgejo run output vs GitHub Actions output | 100% workflow parity on pilot workflow | Expand to remaining 73 workflow files incrementally |
| **H3-04: Forgejo Primary (replaces GitHub repos)** | `dark_forest_gate_local.sh` (all graphs valid post-remote-switch) | K-Derm diderm relay verification (push forgejo → relay → GitHub) | Forgejo post-receive relay to GitHub via peptidoglycan/golgiBody-ext | Periodic `dark_forest_gate_local.sh` + temporal position parity |

### Validation Cadence

| Frequency | Validation | Purpose |
|-----------|-----------|---------|
| Continuous (15-min cron) | `nucleus-deploy telemetry` (was `membrane_telemetry.sh`) | Sovereignty channel health |
| Daily | `nucleus-deploy summary` (was `membrane_summary.sh`) | Rolling 7-day parity report |
| Pre/post any deployment | `dark_forest_gate_local.sh` | Deploy graph structural integrity |
| Pre/post any VPS change | `darkforest_membrane.sh` | Membrane security posture |
| Weekly | `nucleus-deploy security` | Full multi-layer security sweep |
| Per sovereignty cutover | Corresponding parity script | Phase-specific cutover gate |

---

## Interstadial / Stadial Phase Tagging

Interstadial exit criteria: `infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md`

### Interstadial Targets (pre-wire — shadow runs can begin)

| ID | Target | Pillar | Status |
|----|--------|--------|--------|
| H2-01→04 | BTSP auth dual-auth shadow run | P2 (NUCLEUS) | **ACTIVE** — `jupyterhub_btsp_auth.py` + `deploy_btsp_auth_shadow.sh`. Dual-auth LIVE on JupyterHub (May 14). Auth events accumulating |
| H2-05 | NestGate content pipeline | P2 (NUCLEUS) | **DONE** (Session 60) — `publish_sporeprint.sh` READY |
| H2-06→09 | petalTongue content serving + extracellular | P2 (NUCLEUS) | UNBLOCKED — NestGate `content.*` live |
| H2-12 | BearDog TLS shadow on :8443 | P1 (Primal) + P2 (NUCLEUS) | **RUNNING** — BearDog v0.9.0 live on :8443, 3ms RPC (probe fixed). Baseline comparison captured |
| H2-14 | Songbird NAT VPS relay | P1 (Primal) + P2 (NUCLEUS) | **LIVE + HTTP PARITY PASS** — cellMembrane fieldMouse on 157.230.3.183:3478. VPS 68ms vs GitHub Pages 89ms. `membrane.primals.eco` TLS live (ACME cert) |
| H2-17→20 | DoT sovereign DNS | P2 (NUCLEUS) | **H2-17 DEPLOYED** — knot-dns v3.2.6 on cellMembrane VPS (157.230.3.183:53), DNSSEC ECDSAP256SHA256 auto-signed, `primals.eco` zone authoritative, 45ms from gate. H2-18 (NS transfer) pending registrar action. H2-19/H2-20 (BTSP direct + unbound recursive) planned. |
| TIER-2 | Tier 2 Science API (toadstool.validate) | P2 (NUCLEUS) | **WIRED** — S250 implemented, 12 workload TOMLs with `toadstool-validate-v1` schema |
| SHADOW | composition.deploy.shadow | P2 (NUCLEUS) | **WIRED** — `shadow_deploy()` in `deploy_graph.sh`, biomeOS v3.53 |
| ABG-WCM | Thread 1 WCM compositions via provenance trio | P3 (ABG) | Graph capabilities reconciled (GAP-36 canonical names); `nucleus-deploy provenance` (was `provenance_pipeline.sh` in `deploy/legacy/`) exercises dag.*/spine.*/braid.* |
| LITHO-INT | lithoSpore workload integration | P4 (lithoSpore) | **EXCEEDED** — 7/7 modules PASS Tier 2 (75/75 checks, 117 tests), cross-tier parity 7/7 MATCH, Tier 3 wired (trio JSON-RPC), litho-core shared library, liveSpore tracking |
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

lithoSpore (`gardens/lithoSpore`) is a projectNUCLEUS subsystem — the
first Targeted GuideStone artifact.

- Workload TOMLs: `litho-validate-tier2.toml`, `litho-validate-tier3.toml`
- **Phase 2 COMPLETE**: 7/7 modules PASS Tier 2 (75/75 checks, 117 tests). Module 5 (biobricks) activated — all modules live
- **Cross-tier parity**: 7/7 modules MATCH (Python Tier 1 ↔ Rust Tier 2). `litho parity` is the ecosystem standard for cross-implementation validation
- **Tier 3 wired**: JSON-RPC trio provenance (rhizoCrypt + loamSpine + sweetGrass) with graceful degradation via `try_record_tier3()` / `primals_reached` tracking
- Shared `litho-core` library: `discovery`, `harness`, `stats`, `validation` modules — cross-module reuse pattern
- Integration point: NUCLEUS dispatches lithoSpore workloads via `composition.deploy(graph)`
- USB deployment: Tier 3 deploy graph (`graphs/ltee_guidestone.toml`) composes
  NUCLEUS primals around lithoSpore science modules
- **Ferment transcript ingestion**: wetSpring Barrick 2009 braids flow into lithoSpore `data.toml` as `upstream_braid` — computation-verified science chain
- Interstadial target: 2+ modules PASS at Tier 1 (Python) with real data — **EXCEEDED** (7/7 modules Tier 2 + parity)
- Stadial target: USB to Barrick Lab (Phase 5) — GuideStone Phase 1 DONE (architecture + queue seeding)

---

## Horizon 4: Transaction Infrastructure (May 2026+)

New horizon tracking economic, ceremony, and federation capabilities that
extend the sovereign organism from compute into transactions.

| ID | Gap | Status | Blocker | Relates to |
|----|-----|--------|---------|------------|
| **H4-01** | Tier 2 key ceremony — personal sovereignty | DESIGNED | BearDog ceremony orchestration mode | `specs/TIER2_CEREMONY_DESIGN.md` |
| **H4-02** | Tier 2 event ceremony — multi-device (stadium) | DESIGNED | Songbird ceremony relay mode | `specs/TIER2_CEREMONY_DESIGN.md` |
| **H4-03** | Steam save federation — cross-gate NestGate sync | DESIGNED | 10G cables ($50) | `STEAM_DATA_SERVICE_SPEC.md` |
| **H4-04** | Steam save federation — VPS off-site backup | DESIGNED | NestGate federation protocol activation | — |
| **H4-05** | Novel Ferment Transcript — full mint lifecycle | READY | esotericWebb session exercising provenance trio | `NOVEL_FERMENT_TRANSCRIPTS.md` |
| **H4-06** | Loam Certificate game key — sovereign ownership | READY | loamSpine `certificate.mint` exercised E2E | `LOAM_CERTIFICATES_LIVE.md` |
| **H4-07** | sunCloud attribution split — metabolic routing | DESIGNED | sweetGrass braid → value split implementation | `SUNCLOUD_ACTIVATION.md` |
| **H4-08** | Membrane cert verification — ionic TLS API | READY | BearDog ionic + Caddy endpoint wiring | — |
| **H4-09** | Ceremony attenuation scheduler | DESIGNED | loamSpine expiry + capability shrink logic | `specs/TIER2_CEREMONY_DESIGN.md` |
| **H4-10** | Fuzz evolution — darkforest membrane mode | DESIGNED | darkforest `--membrane` flag implementation | `specs/FUZZ_EVOLUTION.md` |
| **H4-11** | Artifact validation suite — benchScale scenario | READY | All primals running on ironGate | `specs/VALIDATION_PLAYBOOK.md` |
| **H4-12** | benchScale Docker topology — provenance trio lab | READY | Docker + plasmidBin binaries | `sort-after/benchScale/topologies/nucleus/` |
| **H4-13** | agentReagents gate template — VM-level provenance validation | READY | libvirt + base image + plasmidBin | `sort-after/agentReagents/templates/nucleus_gate.yaml` |
| **H4-14** | Cross-gate artifact federation — multi-gate NestGate replication | DESIGNED | 10G cables + second gate online | `specs/VALIDATION_PLAYBOOK.md` |
| **H4-15** | VPS ferment tracker — membrane events as NFT DAG | DESIGNED | H4-05 exercised + membrane_telemetry feeding DAG | `specs/VALIDATION_PLAYBOOK.md` |

### Priority sequence

1. **H4-11** (now): Run artifact validation suite against live NUCLEUS — immediate feedback
2. **H4-05 + H4-06** (now): Exercise ferment mint + Loam cert on ironGate (all primals present)
3. **H4-08** (now): Wire cert verification through VPS TLS surface
4. **H4-12** (next): Stand up Docker provenance trio lab for CI-friendly validation
5. **H4-01** (next): Personal sovereignty ceremony (all primitives exist, needs orchestration)
6. **H4-03 + H4-14** (after cables): Cross-gate save federation + artifact replication
7. **H4-15** (after H4-05): VPS membrane events feeding ferment DAG
8. **H4-07** (needs activation): sunCloud value routing
9. **H4-13** (needs libvirt): agentReagents VM gate for full OS-level isolation testing
10. **H4-02** (needs protocol): Multi-device stadium ceremony
11. **H4-10** (next sprint): darkforest membrane extension

---

## Wave 24: Shadow Run Matrix (Sovereignty Parity Proofs)

Deploy graph: `graphs/sovereignty_shadow.toml`
Protocol: calibrate → shadow → cutover (`wateringHole/SOVEREIGNTY_STANDARDS.md` §2)
Orchestrator: `infra/benchScale/scenarios/shadow_run_orchestrator.sh`
Telemetry: `nucleus-deploy telemetry` → `nucleus-deploy summary` → `validation/baselines/membrane_7day.toml` (was `deploy/membrane_telemetry.sh` → `deploy/membrane_summary.sh`, now in `deploy/legacy/`)

### Shadow Status

| # | Track | Sovereign | Commercial | Status | Parity Script | Metric |
|---|-------|-----------|------------|--------|---------------|--------|
| S1 | TLS termination | BearDog :8443 (rustls) | Cloudflare TLS | **LIVE** — 6-12ms vs 163ms (13-27×). VPS + ironGate deployed | `btsp_tls_parity.sh` | JSON-RPC p95, error rate |
| S2 | NAT traversal | Songbird TURN relay | cloudflared tunnel | **LIVE** — relay v0.2.1 on VPS, 100% reachable 3+ days | `songbird_nat_parity.sh` | TURN reachability, UDP latency |
| S3 | Content hosting | NestGate + petalTongue | GitHub Pages | **LIVE** — petalTongue web on VPS :8080. TTFB 67ms vs 111ms GH (40% faster) | `nestgate_content_parity.sh` | TTFB, content hash, 404 rate |
| S4 | Auth / JupyterHub | BearDog BTSP dual-auth | OAuth2 proxy | **SHADOW LIVE** — dual-auth shadow active, events accumulating; full cutover pending | (new — auth parity) | Auth latency (<50ms), session mgmt |

### Remaining per Track

**S1 — BearDog TLS (LIVE, shadow metrics accumulating)**
- [x] rustls X.509 termination live on :8443
- [x] Per-IP sliding-window rate limiter
- [x] `deploy_beardog_tls_shadow.sh` operational
- [x] bearDog ACME Phase 2: `beardog-acme` crate SHIPPED (Wave 106) — HTTP-01, cert storage, hot-reload via `Arc<ServerConfig>` swap, shadow metrics collector
- [x] `specs/ACME_TLS_INTEGRATION_PATH.md` exists (7.5KB design doc)
- [x] `deny.toml` ring wrappers reconciled: `["rustls", "rustls-webpki"]`
- [ ] bearDog ACME Phase 3: renewal daemon integration (12h check, 30-day-before-expiry)
- [ ] 7-day continuous p50/p95/p99 measurement via `nucleus-deploy telemetry` (was `membrane_telemetry.sh`)
- [ ] Cutover gate: sovereign p95 ≤ 1.5× commercial p95 for 7 consecutive days

**S2 — Songbird NAT (READY, relay deployment pending)**
- [x] `songbird-turn-client` crate (RFC 5766 TURN, Wave 205)
- [x] STUN wire-compliant (RFC 5389)
- [x] 5-tier ConnectionFallbackChain (direct → STUN → lineage → TURN → emergency)
- [x] `primal.announce` wired
- [x] Relay deployment guide shipped: `deployment/relay/README.md` (5-minute quick deploy)
- [x] TURN server improvements pulled (Wave 206+: session management, coordinator enhancements)
- [x] Deploy relay node on cellMembrane VPS — `songbird-relay.service` LIVE (v0.2.1, 3+ days, 100% reachable)
- [ ] Dual-path shadow routing (cloudflared + songbird in parallel)
- [ ] Cross-gate test: 2+ gates via songbird relay
- [ ] 7-day metric collection

**S3 — Content Hosting (READY, mirror pending)**
- [x] NestGate: 8 `content.*` methods on all 4 transports (Session 60)
- [x] petalTongue: `backend=nestgate` live (v1.6.6)
- [x] `content_pipeline_smoke.toml` graph
- [x] ACME routing rule in `routing_config.toml`
- [ ] Mirror GitHub Pages content to NestGate via `content.put`
- [ ] DNS staging subdomain → petalTongue :8080
- [ ] TTFB, cache hit, 404 rate metric collection
- [ ] Static asset parity (CSS/JS/images, MIME types, compression)

**S4 — Auth (READY, JupyterHub integration designed)**
- [x] Ed25519 ionic tokens with TTL/expiry (Wave 102)
- [x] BTSP Phase 3 AEAD on all 13 primals
- [x] FIDO2/CTAP2 IPC surface (Wave 103)
- [x] `deploy_btsp_auth_shadow.sh` operational
- [x] `specs/JUPYTERHUB_DUAL_AUTH_INTEGRATION.md` SHIPPED (bearDog Wave 106) — full architecture for dual-auth middleware (`BEARDOG_TLS_MODE=shadow`)
- [ ] Implement dual-auth middleware per bearDog spec (D4)
- [ ] Session management: bearDog token → JupyterHub session mapping
- [ ] Auth latency target: <50ms p95

### Cutover Criteria

Per `wateringHole/SOVEREIGNTY_STANDARDS.md`:

| Track | Metric | Threshold | Duration |
|-------|--------|-----------|----------|
| S1 TLS | `beardog_tls_p95` | ≤ 1.5× `cloudflare_ttfb_p95` | 7 consecutive days |
| S2 NAT | TURN reachability | 100% | 7 consecutive days |
| S3 Content | VPS TTFB | ≤ 110% GitHub Pages TTFB | 7 consecutive days |
| S4 Auth | Auth latency | < 50ms p95 | 7 consecutive days |

Cutover gate: `nucleus-deploy summary` (was `deploy/membrane_summary.sh`, now in `deploy/legacy/`) computes rolling 7-day window.
All 4 tracks must pass simultaneously before DNS switch.

### Upstream Blockers (for primal teams)

| Blocker | Owner | Impact | Status |
|---------|-------|--------|--------|
| ~~`beardog-acme` crate~~ | bearDog | ~~S1 cutover~~ | **RESOLVED** — Wave 106 shipped (10 source files, shadow_metrics.rs) |
| ~~`ACME_TLS_INTEGRATION_PATH.md` missing~~ | bearDog | ~~S1 design gap~~ | **RESOLVED** — exists on disk (7.5KB, Wave 105) |
| ~~`deny.toml` ring wrappers stale~~ | bearDog | ~~ring policy~~ | **RESOLVED** — `["rustls", "rustls-webpki"]` (Wave 105) |
| ~~JupyterHub dual-auth spec~~ | bearDog | ~~S4 design~~ | **RESOLVED** — `JUPYTERHUB_DUAL_AUTH_INTEGRATION.md` shipped (Wave 106) |
| ~~Songbird relay node deployment on cellMembrane~~ | songbird + projectNUCLEUS | ~~S2 shadow~~ | **RESOLVED** — `songbird-relay.service` LIVE on VPS (v0.2.1, 3+ days, 100% reachable) |
| ~~petalTongue static asset parity testing~~ | petalTongue + projectNUCLEUS | ~~S3 cutover~~ | **RESOLVED** — `petaltongue-web.service` deployed on VPS :8080. TTFB parity PASS (67ms vs 111ms). Content hash parity pending (mirror) |

---

## Wave 28-29: sporePrint + cellMembrane Nest Atomic

### Wave 28: sporePrint pappusCast (SP-4)

| # | Gap | Owner | Priority | Status |
|---|-----|-------|----------|--------|
| SP-4 | Sovereign publish: `publish_sporeprint.sh` → NestGate `content.put` | projectNUCLEUS | LOW | Script complete (123 LOC). **Blocked**: BearDog `auth.issue_session` returns fixed scopes without `content.*`. NestGate MethodGate rejects. Upstream ask: BearDog scope expansion for pipeline tokens. Natural after S3 content parity proves mirror. |

### Wave 29: cellMembrane Nest Atomic

Expand VPS from Tower Atomic (BearDog + Songbird + SkunkBat) to Nest Atomic
(+ NestGate + rhizoCrypt + loamSpine + sweetGrass). Memory budget: **1,597 MB
available**, current 8 services use 108 MB. Nest trio ~40-60 MB.

| # | Gap | Owner | Priority | Status |
|---|-----|-------|----------|--------|
| CM-1 | `deploy_membrane.sh --composition nest` | plasmidBin | ~~MEDIUM~~ | **RESOLVED** (Wave 29) — tooling shipped, Nest binaries in plasmidBin, deploy script supports `--composition nest` |
| CM-2 | `membrane_provenance.sh` post-deploy trio hook | projectNUCLEUS | ~~MEDIUM~~ | **RESOLVED** (Wave 29) — `membrane_provenance.sh` created with SSH dispatch + graceful degradation |
| CM-3 | Cross-gate `capability.call` testing | primalSpring + songbird | ~~LOW~~ | **RESOLVED** — CG-8 resolved in songbird Wave 211; primalSpring scenario `s_cross_gate_capability_call` shipped |
| CM-4 | darkforest MEM-14→MEM-17 (Nest health) | projectNUCLEUS | ~~LOW~~ | **RESOLVED** (Wave 29) — 17 PASS, 0 FAIL in `darkforest_membrane.sh`; Nest health checks SKIP when not deployed |

---

## Scoring

```
Horizon 1 (external security):    ██████████  COMPLETE — all resolved, darkforest v3.0 authoritative (inner + outer scope)
Horizon 2 (sovereignty):          █████████░  Nest Atomic LIVE (11 svc), Ch3 TLS LIVE (ACME), Forgejo primary (39 repos, diderm relay), shadow 6/0/0
Horizon 3 (primal-only):          ███░░░░░░░  H3-04 Forgejo ACTIVE, H3-07/H3-08 UNBLOCKED, H3-11 FlockGate DESIGNED
Horizon 4 (transactions):         ██░░░░░░░░  READY — playbook + benchScale topologies wired. H4-11/12/13 ready to run.
Shadow (Wave 38→59):              ████████░░  S1-S3 LIVE, S4 SHADOW LIVE, S5 DEPLOYED. Orchestrator: **6 PASS, 0 FAIL, 0 SKIP**.
Niche climate (Wave 59):          ████░░░░░░  NC-1 CODE COMPLETE (live gated on VPS v3.84). NC-2 IN PROGRESS. NC-3 CODE CONSUMED. NC-4 ADVANCING.
Upstream (waiting):               ██████████  ZERO OPEN — 13/13 primals, 8/8 springs at zero debt. 460 methods (Wave 56).
Interstadial exit:                █████████▌  EXIT GATE CLEARED — 9.5/10. Stadial entry: NC-1 + NC-2 + NC-4.
Dark Forest Glacial Gate:         ██████████  PASS — 33/33 checks, 5/5 pillars, all graphs hardened
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
| 2026-05-14 | **cellMembrane handoff — ironGate ownership transfer**: primalSpring deployed and hardened cellMembrane fieldMouse (157.230.3.183, DigitalOcean nyc1, ~$4/mo). Channel 2 (Songbird TURN relay :3478) LIVE. Hardening: fail2ban active, UFW 22+3478 only, exim4 purged, SSH key-only, journald persistent. Ownership transferred to ironGate/projectNUCLEUS. (1) H2-14 moved from OPS-READY → **LIVE**. (2) `nucleus_config.sh` updated with cellMembrane VPS config (`MEMBRANE_VPS_IP`, `SONGBIRD_TURN_SERVER`, `SONGBIRD_TURN_USERNAME`). (3) `songbird_nat_parity.sh` updated with TURN relay reachability probe + cellMembrane defaults. (4) `deploy_songbird_relay.sh` marked superseded by `plasmidBin/deploy_membrane.sh` for cellMembrane ops. (5) Interstadial exit advanced — 2 items remain (BTSP dual-auth, WCM through trio). Registry at **427 methods**. Private ops repo: `sporeGarden/cellMembrane`. Ops tooling: `plasmidBin/deploy_membrane.sh`. Architecture: `wateringHole/MEMBRANE_CHANNEL_ARCHITECTURE.md` + `CELLMEMBRANE_FIELDMOUSE_DEPLOYMENT.md`. |
| 2026-05-14 | **cellMembrane geo-delocalization — RustDesk + multi-gate SSH**: primalSpring shipped three updates to the cellMembrane VPS: (1) RustDesk sovereign relay LIVE (hbbs v1.1.15 on :21115-21116, hbbr on :21117) — sovereign remote desktop access through our own relay, all traffic e2e-encrypted. (2) Multi-gate SSH key management via `deploy_membrane.sh keys {list,add,revoke}` — audit-tagged, named keys for onboarding remote gates. (3) VPS hardening: `droplet-agent` purged (opaque DO monitoring), firewall updated to 22+3478+21115-21117. Escalation ladder advanced to Phase 0.5. `nucleus_config.sh` updated with `RUSTDESK_ID_SERVER` + `RUSTDESK_RELAY_SERVER`. SECURITY_VALIDATION.md Layer 6 updated: RustDesk threat model (ID enumeration LOW, relay impersonation mitigated by client key validation), darkforest MEM-11→MEM-15 gaps added. Full VPS verification: 3 services active, 14% disk, 103Mi RAM, no unexpected listeners. |
| 2026-05-14 | **Comprehensive sovereignty documentation**: SECURITY_VALIDATION.md Layer 6 (External Membrane) with full threat model, 15 darkforest MEM check IDs, Dark Forest provider analysis, escalation ladder. TUNNEL_EVOLUTION.md Step 3c updated (cellMembrane LIVE, what-was-built/what-remains), dependency tracker current, security posture table with cellMembrane column. GATE_PORTABILITY.md: external membrane layer in cell diagram, Channel 2+2b LIVE, failure modes for VPS down/compromised/provider outage. EVOLUTION_GAPS.md: 15 MEM security gaps, scoring updated. |
| 2026-05-14 | **13/13 primals LIVE + provenance pipeline validated**: (1) All 13 primals deployed on active gate — songbird and coralreef live for first time after manual binary recovery from plasmidBin v2026.05.12 (v2026.05.14 release shipped only 4/13 binaries). (2) BTSP dual-auth shadow ACTIVE on JupyterHub — BTSPAuthenticator plugin live with PAM fallback, auth events accumulating. (3) Provenance pipeline 9-phase run complete: 6/12 wetspring workloads PASS with full trio chain (BLAKE3 → rhizoCrypt DAG → loamSpine spine → sweetGrass braid). Merkle root + ed25519 witness braid operational. (4) Cross-primal auth (H3-07) identified as local deployment blocker — provenance primals temporarily run permissive for pipeline execution. (5) `SPRINGS_ROOT` fix: ToadStool workloads failed with "Executable file not found" — fixed by exporting `SPRINGS_ROOT` in `deploy_primal_start.sh`. (6) Songbird classified as optional in `provenance_pipeline.sh` — pipeline no longer blocks on its health check. **Upstream issues identified**: plasmidBin `fetch.sh --force` is destructive (deletes working binaries before verifying replacements), v2026.05.14 release incomplete (4/13 binaries shipped). BearDog TLS shadow restarted post-reboot: 2ms RPC latency vs 102ms Cloudflare (51x). |
| 2026-05-15 | **Dark Forest Glacial Gate PASS + deploy graph hardening + 427 methods**: (1) `validation/dark_forest_gate_local.sh` created — 33-check 5-pillar structural validation of local deploy graphs, mirroring primalSpring's `s_dark_forest_gate` scenario. All 5 pillars PASS. (2) All deploy graphs (`nucleus_complete`, `node_atomic_compute`, `ionic_capability_share`, `basement_hpc_covalent`, `friend_remote_covalent`) updated with `secure_by_default = true` in `[graph.metadata]` (DF-4 requirement). (3) Registry references updated from 418 to 427 methods across all docs (`README.md`, `PHASES.md`, `EVOLUTION_GAPS.md`, `LIVE_SCIENCE_API.md`, workload TOMLs). (4) Port realignment confirmed: `nucleus_config.sh` already matches Zero-Port Standard (NestGate 9500, Squirrel 9300). (5) `graphs/README.md` updated with Dark Forest compliance section. Interstadial scoring: 9/10 (Dark Forest gate cleared). |
| 2026-05-15 | **Interstadial exit gate CLEARED + cleanup**: (1) `composition.deploy(graph)` `--live` mode validated 12/12 primal nodes LIVE. (2) `darkforest_membrane.sh` created — 17 PASS, 0 FAIL, 1 SKIP against cellMembrane VPS (MEM-01→MEM-13). (3) 7-day Cloudflare baseline summary generated (9 days, 950 samples). (4) Songbird NAT shadow started (TURN 100% reachable). (5) plasmidBin checksums.toml synced to v5.4.0 (9 binaries). (6) 12/12 wetspring workloads wired with `toadstool-validate-v1` schema. (7) MEM-01→MEM-13 gaps updated from open to DONE/PASS. (8) Scoring updated to 9.5/10 — exit gate cleared. (9) Validation debris archived (5 timestamped docs + 1 provenance run → `validation/archive/`). (10) wateringHole handoff + exit criteria v1.7 written. |
| 2026-05-15 | **Sovereignty evolution — Forgejo primary + VPS Tower + Channel 3 shadow**: (1) Forgejo PRIMARY: 32 repos across 3 orgs (sporeGarden, ecoPrimals, syntheticChemistry) mirrored. SSH server enabled (:2222), `.netrc` credential caching, `forgejo_mirror.sh` for org/repo creation + dual-push. H3-04 ACTIVE. (2) VPS resized $4→$12 (512MB→2GB) via doctl. Tower composition deployed: BearDog crypto (:9100, UDS), SkunkBat audit (:9140, TCP --no-uds), 1.7GB free. Fixed: `skunkbat-membrane.service` for standalone mode, `tmpfiles.d/membrane.conf` for runtime dir persistence. (3) DO API token encrypted with BearDog AES-256-GCM (Argon2id KDF, `membrane-vault` key). MEM-15 DONE. (4) Channel 3 shadow LIVE: Caddy v2.11.3 on :80, health/status endpoints active, TLS config blocks ready for DNS grey-cloud. UFW updated: 443/tcp + 80/tcp. `caddy-tls.service` unit created. (5) Content-aware routing prototype: `routing_config.toml` defines rules (ACME→local, static→cache, git/API/auth→gate, large→P2P), trust model (covalent/ionic/metallic/weak→content scopes), cache policy (256MB, webhook invalidation). NestGate cache seeded on VPS. (6) `nucleus_config.sh` updated with Channel 3 + routing settings. (7) Second DO droplet (570909451, 159.223.173.73) discovered — old primalSpring instance, unreachable. |
| 2026-05-15 | **L3+L4 membrane telemetry — continuous sovereignty shadow**: (1) `deploy/membrane_telemetry.sh` — unified probe across external + internal membranes (Caddy, TURN, BearDog TLS, VPS SSH, Cloudflare tunnel, per-primal RPC, content TTFB, BTSP auth). Cron-ready (15-min). (2) `deploy/membrane_summary.sh` — rolling 7-day `membrane_7day.toml` with parity checks and cutover gates. (3) `nucleus_config.sh` telemetry settings (interval, dir, window, cutover days). (4) `shadow_run_orchestrator.sh` baseline path gap fixed — unified `membrane_7day.toml` preferred, results append to daily CSV. (5) `routing_config.toml [telemetry]` — `shadow_mode = "permanent"`, SkunkBat correlation. (6) Docs updated: `PRIMAL_VS_SOVEREIGNTY_GOALS.md` L3+L4 bridge, `SOVEREIGN_COMPOSITION_EVOLUTION.md` permanent shadow model. |
| 2026-05-15 | **Sovereignty targets executed — content sync + DNS grey-cloud + ACME TLS + HTTP parity**: (1) sporePrint content synced to VPS NestGate cache (19MB, `scp` — rsync unavailable on VPS). Caddy serving real content on :80. (2) `membrane.primals.eco` DNS A record created (DNS-only, not proxied, TTL 300). Safe test domain — no production traffic affected. (3) ACME cert automatically obtained (Let's Encrypt E8, valid to Aug 13 2026). Caddy TLS block added to `/etc/membrane/Caddyfile` — serves health, status, and sporePrint content. (4) BearDog TLS probe fixed: `probe_rpc()` rewritten to use `/dev/tcp` + `read -t 1` instead of `nc` (which waited for socket close). Latency: 3ms actual (was 3021ms). (5) BTSP auth telemetry: journald scan primary, logfile fallback. Semicolons for field separators in CSV extra column. (6) HTTP parity PASS: VPS 68ms TTFB vs GitHub Pages 89ms (10 samples). TLS parity via `membrane.primals.eco`: 130ms vs 96ms (within 35% — expected VPS vs CDN). Uptime 100% both channels. |
| 2026-05-15 | **Horizon 4 (transactions) created**: 10 gaps tracked across ceremony, federation, and economics. (1) Tier 2 key ceremony designed — personal sovereignty + event (stadium) types, full protocol spec in `specs/TIER2_CEREMONY_DESIGN.md`. (2) Steam save federation architecture — cross-gate via 10G backbone, VPS off-site backup, multi-household via Songbird. (3) Ferment token + Loam Certificate lifecycle ready to exercise (H4-05, H4-06). (4) sunCloud metabolic routing designed (H4-07). (5) Membrane cert verification wirable now (H4-08). (6) Unified architecture: `gen4/architecture/SOVEREIGN_TRANSACTION_MEMBRANE.md` ties all threads through gram-negative membrane model. (7) Fuzz evolution roadmap (H4-10) per `specs/FUZZ_EVOLUTION.md`. Scoring: 1/10 (designed, not exercised). |
| 2026-05-15 | **Artifact validation playbook + benchScale topologies**: (1) `specs/VALIDATION_PLAYBOOK.md` — practical "run this, validate that" mapping for 7 long-term artifacts (provenance trio, NFT, Loam cert, Tier 2 ceremony, Steam federation, sunCloud, BearDog genetics). Each artifact has smallest testable unit, smallest composition, 2-3 use cases, 2-3 science cases, and benchScale integration. (2) 4 Docker topologies for Rust benchScale: `provenance_trio`, `tower_membrane`, `ferment_lifecycle`, `full_nucleus` — lab-grade multi-primal composition testing. (3) `agentReagents/templates/nucleus_gate.yaml` — minimal VM gate provisioning provenance trio with systemd units. (4) `scenarios/artifact_validation.sh` — 7-section bash scenario exercising all artifacts against live NUCLEUS (TOML report output). (5) H4-11→H4-15 gaps added. Scoring: 2/10 (tooling ready, waiting for exercise). |
| 2026-05-16 | **Deep debt + portability**: 55 Rust tests (darkforest 34, tunnelKeeper 21), discovery module with 3-tier cascade, reqwest 0.13 (aws-lc-rs interim), capability-based primal resolution, hardcoded path elimination. |
| 2026-05-17 | **Reorg**: sporeGarden → gardens. cellMembrane moved to gardens/. Stale duplicates removed. Systemd units, forgejo mirror, docs updated. |
| 2026-05-17 | **Validation Gate Matrix + infrastructure review**: (1) Validation Gate Matrix section added — maps each of 11 validation systems to specific sovereignty phase transitions (H2-01→H2-20, H3-03→H3-04). Pre-transition, shadow, cutover, and post-cutover gates defined for each. (2) Validation cadence table added (continuous/daily/weekly/per-cutover). (3) Stale `gardens/sporeGarden/` clone removed (duplicate projectNUCLEUS). (4) `.env` audit: all sensitive files (squirrel API keys, JWT secrets) properly gitignored, no contamination risk. (5) `REPO_MEMBRANE_BOUNDARY.md` created in wateringHole — full repo classification (inner-only/dual-push/outer-only) with contamination risk matrix and Forgejo migration path. (6) cellMembrane decision documented: recommend Forgejo-only when operationally stable. |
| 2026-05-17 | **Wave 21 absorption**: (1) Registry updated 427→445 methods (Wave 20, stability-tier annotated). (2) lithoSpore refs updated 6/7→7/7 PASS (75/75 checks, 117 tests, cross-tier parity 7/7 MATCH, Tier 3 wired). (3) `primal.list` / `capability.list` canonical schemas marked SHIPPED — discovery module updated to prefer canonical `{ "primals": [...], "count": N }` envelope. (4) Ferment transcript dispatch route documented in `SCIENCE_DISPATCH_MAP.md` — wetSpring → trio → braid → lithoSpore pipeline. (5) Stability tier awareness added (stable/evolving/internal per capability_registry.toml). (6) cellMembrane degradation behavior documented in `EXECUTION_MODEL.md` — per-service fallback table. (7) Cross-tier parity reference added to `TIER2_CEREMONY_DESIGN.md`. (8) Partial provenance (trio transaction semantics) documented in dispatch map. |
| 2026-05-20 | **Wave 28-29 absorption + Nest Atomic preparation**: (1) Wave 28 SP-4 gap analyzed: `publish_sporeprint.sh` structurally complete but blocked on BTSP scope — BearDog `auth.issue_session` returns fixed scopes without `content.*`, NestGate MethodGate rejects. Upstream handback to bearDog team. (2) Wave 29 CM-2 `membrane_provenance.sh` created — post-deploy trio hook (DAG + spine + braid verification via SSH, graceful degradation). (3) CM-4 `darkforest_membrane.sh` MEM-14→MEM-17 added — NestGate, rhizoCrypt, loamSpine, sweetGrass health checks (SKIP when not deployed, PASS/FAIL when live). 17 PASS, 0 FAIL, 5 SKIP. (4) VPS memory budget confirmed: 1,597 MB available, 108 MB used by 8 services, Nest Atomic ~40-60 MB = budget OK. (5) All shadow services confirmed running 24h+ since deployment (BearDog TLS, petalTongue web, Songbird TURN). |
| 2026-05-19 | **Wave 24 absorption + phantom gap clearance**: (1) `sovereignty_shadow.toml` deploy graph created — 4-track (TLS/NAT/content/auth), 7 nodes, per-track cutover criteria. (2) Shadow config centralized in `nucleus_config.sh` (BTSP_SHADOW_*, BEARDOG_TLS_MODE, SONGBIRD_RELAY_URL). (3) Shadow matrix S1-S4 tracked in EVOLUTION_GAPS. (4) bearDog pulled (Waves 105-106): `beardog-acme` crate SHIPPED (10 source files, shadow_metrics.rs), `ACME_TLS_INTEGRATION_PATH.md` exists (7.5KB), `deny.toml` ring wrappers reconciled `["rustls", "rustls-webpki"]`, `JUPYTERHUB_DUAL_AUTH_INTEGRATION.md` shipped. (5) songbird pulled (Wave 206+): TURN server improvements, relay deployment guide, coordinator enhancements. (6) 4/5 upstream blockers cleared — remaining: relay deploy on cellMembrane + petalTongue asset parity test. |
| 2026-05-28 | **Wave 59 false readiness corrections**: (1) **NC-1 status**: WIRED → **CODE COMPLETE** — biomeOS v3.84 shipped `biomeos-pseudospore` + emit materialization. Live column U gated on VPS deploy. (2) **NC-3 status**: ADVANCING → **CODE CONSUMED** — sovereignty cutovers remain open (NS registrar, Forgejo releases, CI). (3) **NC-4 status**: ADVANCING — Wave 58 fully absorbed in PRIMAL_GAPS. (4) **biomeOS version**: v3.78 → v3.84 across all current-state sections. (5) **S4 auth**: Reconciled LIVE vs READY → **SHADOW LIVE** (dual-auth shadow active, events accumulating; full cutover pending). (6) **DNS sovereignty label**: S2 → S5 where DNS was mislabeled. knot-dns DEPLOYED, NS cutover pending registrar. (7) **H3-04 blocker text**: "Forgejo Actions working" → "CI still on GitHub Actions (H3-03 open)" — Wave 59 CI sovereignty gap acknowledged (glacial gate, not stadial blocker). (8) **Test counts**: 65/44/21 → 162/125/37 in SECURITY_VALIDATION.md, experiments/README.md (corrected from Wave 58 miscount of 166/41). Total validations 474→576+. (9) **Upstream absorbed**: primalSpring Wave 58b→59 — blake3 correctness (SHA-256→genuine BLAKE3 in nuclear lineage), PermissiveVerifier rename (no downstream impact), `primal_names::*` constants (zero hardcoded strings in orchestrator/routing), 21-file doc alignment, 11 handoffs archived (wateringHole 8 active, 355 archived). |
| 2026-06-01 | **Wave 67 glacial cutover debt sweep**: (1) **Large file splits**: `nucleus-deploy` `spore.rs` (857L) split into `spore/mod.rs` (690L) + `spore/trio.rs` (161L). `provenance.rs` (825L) split into `provenance/mod.rs` (745L) + `provenance/manifest.rs` (88L). All files now under 800L. (2) **Stale spec fixes**: `SECURITY_VALIDATION.md` updated bash invocations to `nucleus-deploy security` CLI. `PHASES.md` gets Wave 67 note about Rust evolution. `tunnelKeeper/README.md` updated: `serde_yaml` → `serde-saphyr`, SongbirdTransport v0.2 documented, `chrono` → `thiserror`. (3) **Wave 67 ACK**: Impulse acked to eastGate — S1 TLS ready, deep debt sweep active. (4) **Tests stable**: 234 tests across 3 crates (46 nucleus-deploy + 48 tunnelKeeper + 140 darkforest). Zero clippy warnings, zero fmt issues. |
| 2026-05-30 | **Wave 64 deep debt sweep**: (1) **Transport module ungated**: `transport.rs` was behind `#[cfg(feature = "songbird-transport")]` — 5 tests never ran. Feature gate removed from module declaration and test block; Cloudflare transport tests now run every `cargo test`. (2) **Last sync `load()` in async context fixed**: `health::run()` called `TunnelConfig::load()` (sync/blocking) on async executor — evolved to `TunnelConfig::load_async()`. (3) **Test coverage climb**: 164→184 tests (+20). darkforest 127→139 (+12 in pentest/compute, pentest/readonly), tunnelKeeper 36→45 (+9 transport, health). New tests: user isolation invariants, port uniqueness, constant sanity, gate_home resolution, CheckBuilder pattern validation. (4) **Dependency audit**: All deps pure Rust (zero C FFI). reqwest uses rustls (not native-tls). crypto deps (chacha20poly1305, ed25519-dalek) pure Rust. serde-saphyr pure Rust YAML. No evolution needed. (5) **Full audit confirmed clean**: Zero files >800L (max 599L health.rs). Zero unsafe code (`#![forbid(unsafe_code)]`). Zero production mocks. Zero production hardcoding. Zero `unwrap()` outside `#[cfg(test)]`. All `clone()` necessary (async ownership). Clippy pedantic+nursery clean. fmt clean. |
| 2026-05-28 | **Wave 58 deep debt execution**: (1) **Async/blocking fix**: tunnelKeeper `health.rs` blocking I/O (`std::process::Command`, `TcpStream`, `std::fs`) wrapped in `tokio::task::spawn_blocking`; credential reads migrated to `tokio::fs::read_to_string`. Async executor no longer stalled by health probes. (2) **Discovery transport evolution**: darkforest `discovery.rs` switched from `send_jsonrpc` (HTTP POST) to `send_jsonrpc_newline` (newline-delimited JSON-RPC) for biomeOS, liveness, and capability probes — matching actual primal wire format. `send_jsonrpc_newline` promoted from dead code to live. (3) **Silent error bugs fixed**: `serde_json::to_string_pretty().unwrap_or_default()` replaced with `?` propagation in config.rs/health.rs. `filter_map(.ok())` ingress rule drop replaced with `collect::<Result<_, _>>()?`. `Command::new("date")` replaced with pure `std::time::SystemTime`. (4) **Dependency reduction**: `chrono` crate removed (zero source refs). `tokio` `process` feature removed (unused). 175→173 transitive deps. (5) **Deploy hardcoding cleanup**: `membrane_provenance.sh` (5 port literals → config vars), `deploy_health_check.sh` (config sourcing), `switch_to_static_observer.sh` (8866 → `$OBSERVER_STATIC_PORT`), `membrane_telemetry.sh` (3478 → `$TURN_PORT`). (6) **Coverage climb**: 65→162 tests. darkforest 44→125 (+81), tunnelKeeper 21→37 (+16). Coverage: darkforest 40.77%, tunnelKeeper 52.67% (llvm-cov). New test modules in fuzz, crypto/protocol, pentest/external, pentest/compute, pentest/readonly, discovery, net. (7) **Hardcoding evolution**: timeout constants extracted, env-var-driven user functions, named port constants. (8) **DNS test fix**: 65s test split into fast variants + `#[ignore]`; test suite runtime: 1s (tunnelKeeper), 3s (darkforest). Build: darkforest 10s, tunnelKeeper 34s. Binaries: 1.1 MB, 6.5 MB. Zero files >600 LOC. Zero TODO/FIXME. Zero clippy warnings. |
| 2026-05-27 | **Wave 56 absorption — VPS deployment standard**: (1) `deploy.sh --uds-only` flag added — suppresses all TCP port arguments across 13 primals (VPS zero-port standard). (2) `deploy_primal_start.sh` refactored: each primal case conditionally builds CLI args, omitting `--port`/`--listen`/`--bind` when port=0. (3) `deploy_graph.sh` updated: `start_primal_from_graph()` respects UDS_ONLY override, replaces inline nohup with `args` array pattern. (4) `deploy_health_check.sh` evolved: `socket_health_check()` function for UDS-only mode — checks socket file existence instead of TCP probe. (5) `nucleus_config.sh` documented as desktop/debug TCP defaults, VPS uses `--uds-only`. (6) primalSpring cell graphs consumed: 6 VPS-ready (`spawn=false`, `transport = "uds_only"`), 3 desktop-only. `cells_manifest.toml` v1.1.0 with `vps_standard` field. (7) `primalspring checksums` + `primalspring registry` subcommands replace shell validation scripts — no shell-side changes needed. (8) VPS deployment flow: `deploy.sh --uds-only` → graph deploy → `biomeos deploy graphs/cells/{spring}_cell.toml` spring overlay. |
| 2026-05-27 | **Wave 55 deep debt sprint**: (1) `yaml_serde` (libyaml C) → `serde-saphyr` 0.0.26 (pure Rust, panic-free, no unsafe). Last C-binding YAML parser eliminated from codebase. `deny.toml` updated to ban `unsafe-libyaml`. (2) `net.rs` refactored — extracted `parse_status_code()` and `split_http_response()` shared helpers, reducing duplicated HTTP parsing across `http_get`/`http_post`/`http_method`/`send_jsonrpc`. (3) 10 new unit tests in `net.rs` (HTTP parsing, unreachable host graceful failure). darkforest now at **44 tests** (was 34). tunnelKeeper holds at 21. Total: **65 Rust tests, 0 failures**. (4) Full deep debt audit: zero files >800 LOC, zero unsafe blocks, zero TODO/FIXME, zero production mocks, `#![forbid(unsafe_code)]` enforced, all `.unwrap()`/`.expect()` confined to `#[cfg(test)]`. (5) Clippy pedantic+nursery: zero warnings across both crates after fixes. (6) Dependency analysis: darkforest has zero C dependencies. tunnelKeeper's remaining C/ASM: `ring` + `aws-lc-sys` (rustls TLS backend, monitored in deny.toml). All others pure Rust. |
| 2026-05-27 | **Wave 55 absorption — niche climate + 460 methods**: (1) Registry 458→460 (`nucleus.ingest_spore` + `nucleus.emit_spore`). primalSpring v0.9.30, 56 scenarios, 813 tests, biomeOS v3.78. (2) NC-1→NC-5 niche climate gaps tracked — NC-1 (spore gateway) BLOCKED on biomeOS, NC-2 (multi-gate mesh) BLOCKED on southGate 7/13, NC-3 (cellMembrane sovereignty) IN PROGRESS. (3) cellMembrane ops doc drift identified: VPS_STATE.md/GLACIAL_SHIFT say Tower-only despite Nest Atomic deployed May 22 (NC-3.1 sync needed). (4) `membrane.toml` → `composition = "nest"` required for K-Derm boundary (NC-3.2). (5) All 14 primals + 8 springs pulled to Wave 55 latest. (6) biomeOS refs updated v3.54→v3.78 across all docs. (7) `nest_ingest_spore` signal graph awareness (biomeOS/primalSpring canonical). (8) Wave 54 southGate redeploy fixes ready (plasmidBin bf5c96b→b310310): petalTongue CLI, barraCuda --no-gpu-probe, ToadStool early health. (9) Stadial entry requires: NC-1 (2+ springs ingest), NC-2 (3+ gates meshed), NC-4 (4 gates healthy). |
| 2026-05-23 | **Wave 46 absorption — primalSpring v0.9.27 zero gate debt**: (1) Registry 445→458 methods (typed `DispatchError`/`IonicProtocolError`/`PhasedIpcError`, env_keys centralization). (2) All 12 deploy graphs now have `secure_by_default = true` — 5 fragments (tower_atomic, node_atomic, nest_atomic, nucleus, rootpulse_commit) hardened. (3) H3-11 (FlockGate cross-WAN) tracked — gate manifest + covalent graph exist, TURN relay LIVE, deployment pending. (4) All 14 primals pulled to latest (Wave 46 deep debt). (5) `serde_yaml` → `yaml_serde 0.10` (eliminated `unsafe-libyaml`; later evolved to `serde-saphyr` Wave 55). (6) `deny.toml` created for darkforest + tunnelKeeper (ban openssl/native-tls). (7) darkforest discovery suite wired (`DISC-01` capability-based resolution). (8) 55 Rust tests PASS (34+21). Upstream: primalSpring at zero gate debt, 49 scenarios, 44-cell deployment matrix. TEAM_OWNERSHIP_MATRIX v1.1: projectNUCLEUS retains deploy/gates/darkforest/Forgejo/genomeBin; cellMembrane owns VPS provisioning. Not yet available upstream: FlockGate cross-WAN, NeuralBridge BTSP compositions, game/nautilus/ml method depth, Thread 1 WCM RPC (0/24). |
| 2026-07-10 | **Wave 136a — Hardening delivered**: (1) **EXP-01 PATCHED**: `(security_headers)` snippet deployed on all 5 Caddy server blocks — HSTS preload (max-age=63072000, includeSubDomains, preload), X-Frame-Options DENY, X-Content-Type-Options nosniff, Permissions-Policy (camera/mic/geo denied), Server header suppressed. (2) **EXP-02 PATCHED**: 404 catch-all removed — Zola `404.html` served via `handle_errors`, `try_files` fallback removed from all blocks. No more content confusion or SEO poisoning. (3) **EXP-03 SELF-RESOLVING**: ACME cert auto-renewal confirmed operational — `primals.eco` renewed Jul 9→Oct 7, `membrane` ~Jul 14, `git` ~Jul 27. Caddy handles lifecycle autonomously. (4) **EXP-04 PATCHED**: fail2ban active on Forgejo SSH port 2222 — maxretry=3, bantime=3600s, findtime=600s. (5) **HTTP/3 confirmed**: `alt-svc: h3=":443"` announced on all public domains. gzip encoding enabled. (6) **Criterion 8 progress**: 3/5 sub-criteria met (headers, 404, fail2ban). Remaining: CSP header, depot rate-limiting, cascade output signing. (7) **Sprint scoped**: 136b (hardening: depot rate-limit, CSP, WG rotation, signing), 136c (resilience: cert drill, WG failover, Forgejo recovery, darkforest outer execution), 136d (monitoring: skunkBat outer mode, overlay IDS, audit trail, cert alerting). |
| 2026-07-10 | **Wave 136 — Security & Exposure Sprint**: (1) **darkforest v3.0** shipped: `--scope outer` flag + `--target` for outer membrane domain. 6 new modules: `outer/tls.rs` (OTR-01→06: TLS reachability, HSTS, cert, protocol downgrade), `outer/http.rs` (OHT-01→06: security headers, 404 behavior, verb fuzzing, path traversal, directory listing, clickjacking), `outer/depot.rs` (ODP-01→04: write rejection, checksums, enumeration), `outer/forge.rs` (OFG-01→04: SSH auth, repo enumeration), `outer/dns.rs` (ODN-01→03: AXFR, DNSSEC, NXDOMAIN), `outer/mesh.rs` (OMS-01→03: WireGuard probes, invalid handshake, mesh surface). 26 new checks total. Tests: 140→146 (all PASS). (2) **Triage matrix absorbed**: 3 CRITICAL (EXP-01 no security headers, EXP-02 404 catch-all, EXP-03 cert expiry), 4 HIGH (EXP-04 Forgejo SSH, EXP-05 depot, EXP-06 JupyterHub, EXP-07 WireGuard), 4 MEDIUM (EXP-08→11), 3 LOW (EXP-12→14). (3) **Glacial Criterion 8** tracked: outer membrane hardened for public exposure. (4) **Failover playbook** RF-01→RF-10 defined: single gate death, VPS death, Caddy crash, depot corruption, DNS poisoning, cert expiry, forge compromise, mesh partition, power loss, build authority compromise. Critical gap: RF-07+RF-10 — no code/content signing on cascade output. (5) **Upstream cascade**: primalSpring manifest fix (loamSpine to nest, rhizoCrypt to westGate), 1101 tests PASS. wateringHole: DNS cutover AAR, sovereign validator AAR, site rebuild fix, provision updates, security sprint handoff. |
| 2026-05-22 | **Wave 38 absorption — Nest Atomic LIVE on VPS**: (1) CM-1 executed: `deploy_membrane.sh --composition nest` — NestGate v2.1.0, rhizoCrypt v0.14.0, loamSpine v0.9.16, sweetGrass v0.7.34 deployed to VPS. Fixed CLI interface mismatches (each primal uses `server` subcommand, not `--socket`), NestGate JWT secret provisioned. (2) CM-2 provenance trio verified E2E: 10/10 PASS — DAG session created, events appended, spine created, braid created, Tower cross-check healthy. (3) CM-4 darkforest: 21 PASS, 0 FAIL, 1 SKIP — NestGate (:9500 HTTP REST), rhizoCrypt (:9602 JSON-RPC), loamSpine (:9700 HTTP), sweetGrass (:9850 TCP) all healthy. MEM-10 updated with Nest ephemeral ports. (4) Shadow orchestrator: 5 PASS, 0 FAIL, 1 SKIP — S1 TLS 13ms vs CF 163ms, S2 TURN reachable, S3 content live, DNS pending. (5) Registry confirmed at 445 (was 452 at Wave 20). (6) VPS running 11 services, 7 primals, 1.6 GB free. Composition advanced from Tower to Nest Atomic. |
