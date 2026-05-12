# Complete External Dependency Inventory

**Date**: 2026-05-07
**From**: projectNUCLEUS
**Purpose**: Map every external dependency in the ecoPrimals ecosystem, classify
by phase (build/run/dev), identify primal replacements (existing/planned/gap),
and note calibration instruments where applicable.

This inventory grounds the sovereignty evolution path. Each dependency is a
measurement target: we capture its baseline behavior, and primal replacements
must prove parity before the external is removed.

---

## Cluster 1 — Cloudflare (DNS / TLS / CDN / Tunnel)

**Status**: Fully mapped in `TUNNEL_EVOLUTION.md`. Baselines capturing. benchScale ready.

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| Cloudflare DNS (`primals.eco` NS) | Runtime | Domain registrar config | Self-hosted authoritative DNS + BTSP DoH | Specified (Step 4) |
| Cloudflare TLS (edge termination) | Runtime | Tunnel ingress for `lab.primals.eco` | BearDog BTSP ChaCha20-Poly1305 + ACME | Specified (Step 3b) |
| Cloudflare CDN (proxy caching) | Runtime | Orange-cloud proxy on `primals.eco` | NestGate content-addressing | Specified (Step 3a) |
| `cloudflared` binary | Runtime | Systemd user service on the active gate | Songbird NAT traversal | Specified (Step 3c) |
| Cloudflare API | Ops | Dynamic DNS updates (future) | Sovereign DNS update mechanism | Planned |
| Cloudflare dashboard | Ops | Tunnel config, DNS management | CLI / RPC-based management | N/A (ops tooling) |

**Calibration**: `infra/benchScale/scenarios/cloudflare_tunnel_baseline.sh` captures hourly metrics. 7-day summary via `validation/baselines/summarize_baselines.sh`.

---

## Cluster 2 — GitHub (Repos / CI / Pages / Releases / Webhooks)

**Status**: Largest and least mapped cluster. 74 workflow files across ecosystem.

### 2a. Git Hosting (Repos)

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| GitHub repos (clone URLs) | Dev | All READMEs, `Cargo.toml` `repository =`, bootstrap scripts | Self-hosted git (Forgejo calibration instrument) | Calibration planned |
| GitHub SSH keys | Dev | `git@github.com:*` clone URLs | BearDog key management | Gap |
| GitHub branch protection | Dev | Repo settings | Forgejo equivalent / RootPulse policies | Gap |

### 2b. CI/CD (GitHub Actions)

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| GitHub Actions runners | CI | 74 `.github/workflows/*.yml` files | Self-hosted runners on the active gate (or Forgejo Actions) | Gap |
| `actions/checkout@v4` | CI | Nearly all workflows | Local clone (self-hosted runner) | Trivial with self-hosted runner |
| `actions/cache` | CI | Rust build caching | Local filesystem cache | Trivial |
| `dtolnay/rust-toolchain` | CI | Rust workflows | Pre-installed toolchain on runner | Trivial |
| `Swatinem/rust-cache` | CI | Rust workflows | Local sccache or cargo cache | Trivial |
| `EmbarkStudios/cargo-deny-action` | CI | Security audit workflows | Local `cargo deny` | Trivial |
| `codecov/codecov-action` | CI | Coverage upload (ionChannel, benchScale, nestGate) | Self-hosted coverage HTML | Low priority |
| `peter-evans/repository-dispatch` | CI | sporePrint cross-repo auto-refresh (26 repos) | Internal webhook / Squirrel RPC | Gap |
| `shalzz/zola-deploy-action` | CI | sporePrint deploy | Local Zola build + NestGate publish | **UNBLOCKED** — NestGate `content.put` shipped (Session 60). `publish_sporeprint.sh` ready. |
| `actions/deploy-pages` | CI | sporePrint to GitHub Pages | petalTongue web serving from NestGate | **UNBLOCKED** — petalTongue `backend=nestgate` + SPA shipped. Wire for production. |

### 2c. GitHub Pages

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| GitHub Pages hosting | Runtime | `primals.eco` static site (sporePrint Zola output) | NestGate + petalTongue web | **UNBLOCKED** — NestGate content pipeline + petalTongue `backend=nestgate` both shipped. Shadow run (H2-08) next. |

### 2d. GitHub Releases & API

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| GitHub Releases (binary hosting) | Runtime | `plasmidBin/fetch.sh`, `primalSpring/tools/fetch_primals.sh` | NestGate blob storage + manifest | **UNBLOCKED** — NestGate `content.put` + `content.resolve` shipped. Wire fetch scripts. |
| GitHub REST API (release metadata) | Dev/CI | Same fetch scripts | NestGate manifest query | **UNBLOCKED** — NestGate `content.collections` + `content.resolve` shipped. |
| `SPOREPRINT_DISPATCH_TOKEN` | CI | GitHub secret store | Local secret management | Gap |
| `SPOREPRINT_REFRESH_PAT` | CI | GitHub secret store | Local secret management | Gap |
| RustSec advisory DB | Dev | `deny.toml` in 32 workspaces | Vendored advisory DB or internal feed | Low priority |

**Calibration instrument**: Forgejo on the active gate. Captures git operation baselines that RootPulse must match.

---

## Cluster 3 — Package Registries (crates.io / PyPI / Conda)

**Status**: Highest inertia, lowest sovereignty urgency. Vendoring is the escape hatch.

| Dependency | Phase | Scale | Where Referenced | Primal Replacement | Status |
|-----------|-------|-------|-----------------|-------------------|--------|
| crates.io (Cargo default registry) | Build | 664 `Cargo.toml` files | All Rust workspaces | Vendored deps or private registry | Low priority |
| PyPI (`pip install`) | Runtime | 16 `requirements.txt` | Springs, JupyterHub, science stacks | ToadStool registry config / wheelhouse | Low priority |
| Conda-Forge channel | Runtime | JupyterHub envs, bioinfo tools | `setup-jupyterhub.sh`, ABG conda envs | Static envs, source builds | Low priority |
| Bioconda channel | Runtime | Bioinformatics tools | ABG conda envs | Source builds, containerized tools | Low priority |
| Miniforge installer | Setup | One-time | `setup-jupyterhub.sh` (GitHub Releases URL) | Cached local copy | Trivial |
| Zola binary | Build | One-time | sporePrint CI, `deploy.yml` | Cached local copy or cargo build | Trivial |
| rustup (`sh.rustup.rs`) | Dev | One-time | Provisioning scripts | Offline rustup distro | Trivial |

**Mitigation**: Cargo vendor, pip download, conda pack. These are build-time dependencies with well-understood offline modes. Full sovereignty here means building everything from source — high effort, low return until all other clusters are sovereign.

---

## Cluster 4 — Container Registries (Docker Hub / ghcr.io / quay.io)

**Status**: Used by ToadStool, biomeOS, wetSpring. Not on the critical path.

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| Docker Hub (`docker.io`) | Runtime | ToadStool defaults, benchScale docker backend | NestGate-stored OCI blobs | Gap |
| GitHub Container Registry (`ghcr.io`) | Runtime | ToadStool tests, biomeOS examples, fossilRecord deploys | NestGate-stored OCI blobs | Gap |
| Quay.io | Runtime | wetSpring Galaxy images | Self-built or cached locally | Low priority |

**Mitigation**: ToadStool already abstracts the registry URL via config. Switching to a local registry (or NestGate-backed OCI store) is a config change once the backend exists.

---

## Cluster 5 — AI/ML APIs (Anthropic / OpenAI)

**Status**: Optional cloud inference. Local fallback exists.

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| Anthropic API (`api.anthropic.com`) | Runtime (optional) | biomeOS nucleus mode, Squirrel AI tools, BearDog BTSP examples | Ollama + local models via Songbird HTTP | Partial (Ollama works) |
| OpenAI API (`api.openai.com`) | Runtime (optional) | biomeOS, Squirrel | Ollama + local models | Partial |
| Hugging Face model cache | Dev | biomeOS model import | Local model registry | Low priority |

**Mitigation**: biomeOS already supports Ollama as a local provider. barraCuda WGSL compute is the long-term sovereign inference path. Cloud APIs are convenience, not dependency.

---

## Cluster 6 — Science Data APIs (NCBI / UniProt / KEGG)

**Status**: Irreplaceable external data sources. Mitigation is caching + provenance.

| Dependency | Phase | Where Referenced | Primal Replacement | Status |
|-----------|-------|-----------------|-------------------|--------|
| NCBI E-utilities / datasets | Data ingest | `foundation/deploy/fetch_sources.sh`, wetSpring experiments | Local mirror + `abg_data.sh` registry | Partial (registry exists) |
| UniProt REST API | Data ingest | `foundation/deploy/fetch_sources.sh` | Local mirror + provenance tracking | Partial |
| KEGG REST API | Data ingest | `foundation/deploy/fetch_sources.sh` | License-conscious mirror | Gap (license constraints) |

**Mitigation**: These are data sources, not services. Once fetched, data is local forever. The `abg_data.sh` registry tracks provenance (BLAKE3 checksums, source URL, download date). sweetGrass braids witness the data lineage. NestGate `storage.fetch_external` already hashes external downloads.

---

## Cluster Summary

| Cluster | Dependencies | Critical Path? | Sovereignty Priority | Calibration Instrument |
|---------|-------------|---------------|---------------------|----------------------|
| 1. Cloudflare | 6 | Yes (membrane layer) | High — Steps 2b-4 | benchScale baselines (capturing) |
| 2. GitHub | ~15 distinct | Yes (extracellular layer) | High — Step 3a, Forgejo | Forgejo (planned) |
| 3. Package Registries | 4 channels | No | Low — vendor when needed | N/A |
| 4. Container Registries | 3 registries | No | Low — config swap | N/A |
| 5. AI/ML APIs | 3 providers | No (optional) | Low — Ollama works | N/A |
| 6. Science Data APIs | 3 sources | No (data, not service) | Low — cache + provenance | `abg_data.sh` registry |
| 7. Internal Primal Gaps | 6 gaps (1 critical) | **No longer blocking** | **Resolved** — JH-0→JH-5 all resolved/enforced | JupyterHub deployment |

---

## Sovereignty Progress by Cluster

```
Cluster 1 (Cloudflare):  ████████░░ ~80% mapped, baselines capturing
Cluster 2 (GitHub):      ███░░░░░░░ ~30% mapped, Forgejo calibration next
Cluster 3 (Registries):  ██░░░░░░░░ ~20% mapped, vendor escape hatch known
Cluster 4 (Containers):  ██░░░░░░░░ ~20% mapped, config swap path known
Cluster 5 (AI APIs):     ██████░░░░ ~60% mapped, Ollama fallback working
Cluster 6 (Science):     █████░░░░░ ~50% mapped, data registry operational
Cluster 7 (Internal):    ██████████ 100% resolved, JH-0→JH-5+JH-11 adopted/enforced, MethodGate 13/13
```

---

## Cluster 7 — Internal Primal Gaps (Discovered via JupyterHub Deployment)

**Status (2026-05-11)**: All 6 gaps resolved. MethodGate enforced on 13/13 primals.
Ionic tokens live, resource envelopes enforced, audit log operational,
cross-primal token federation resolved (JH-11).
See `validation/REVALIDATION_PHASE60_MAY08_2026.md` and `specs/EVOLUTION_GAPS.md` for current state.

> Original gap discovery documented below. These were found by deploying
> JupyterHub as a real multi-user service and observing where the primal
> stack could not enforce what external tooling (PAM, systemd, Cloudflare)
> currently handles.

| Gap | Severity | Owner | Blocks | Description |
|-----|----------|-------|--------|-------------|
| JH-0: RPC dispatcher capability check | **Critical** | All primal teams | Secure multi-user compositions | **ENFORCED** — MethodGate on 13/13 primals, all confirmed enforced via TCP. `-32001 PERMISSION_DENIED` on unauthenticated calls. |
| JH-1: BearDog identity management | High | BearDog | Step 2b (BTSP auth) | **RESOLVED** — `identity.create`, `auth.issue_ionic`, `auth.verify_ionic` live. Ed25519-signed scoped tokens. |
| JH-2: Token-carried resource envelope | High | biomeOS + ToadStool | neuralAPI enforcement | **RESOLVED** — biomeOS v3.48 enforces `timeout_ms`, ToadStool S232 enforces `mem_mb`, `cpu_cores`. |
| JH-3: Composition hot-reload | Medium | biomeOS | Rolling primal updates | **RESOLVED** — `composition.reload` ships in biomeOS v3.47. |
| JH-4: Token issuance UX | Medium | BearDog + primalSpring | Non-technical user onboarding | **RESOLVED** — `auth.issue_session` with purpose-based presets (`jupyterhub`, `desktop`, `admin`). |
| JH-5: Log aggregation + provenance | Medium | skunkBat | Unified security monitoring | **Phase 2 COMPLETE** — `security.audit_log` ring buffer live. Cross-primal forwarding (JH-5 Phase 3) wired but not exercised end-to-end. |
| JH-11: Cross-primal token federation | Medium | biomeOS/BearDog | Cross-gate auth | **RESOLVED** — BearDog `auth.public_key` + biomeOS `BearDogVerifier`. |

**Sovereignty Progress**:

```
Cluster 7 (Internal):   ██████████ 100% — all gaps resolved, MethodGate 13/13 enforced
```

**Relationship to other clusters**: Cluster 7 is the prerequisite for Cluster 1
(Cloudflare) and Cluster 2 (GitHub) sovereignty. RPC capability enforcement
(JH-0) is now enforced on 13/13 primals — complete for Step 2b. Identity
management (JH-1) is resolved — BTSP auth (Step 2b) can replace PAM.
All internal primal gaps are resolved.

---

## Cross-Cutting Dependencies (Not Cluster-Specific)

| Dependency | Type | Note |
|-----------|------|------|
| Domain registrar (`primals.eco`) | Administrative | Irreducible — someone must hold the domain |
| Linux kernel / systemd | OS | Foundation — not a sovereignty target |
| NVIDIA drivers | Hardware | Vendor lock for GPU compute — unavoidable |
| Let's Encrypt / ACME | Runtime (future) | Needed for BearDog TLS (Step 3b) browser compatibility |
| VPS for STUN/TURN relay | Infrastructure | ~$5/mo for Songbird NAT (Step 3c) |

---

## Document History

| Date | Change |
|------|--------|
| 2026-05-07 | Initial inventory. 6 clusters, ~35 distinct dependencies mapped. |
| 2026-05-07 | Added Cluster 7 — Internal Primal Gaps. 5 gaps (1 critical) from JupyterHub deployment patterns. |
| 2026-05-11 | Reconciled Cluster 7: all gaps resolved, MethodGate corrected to 11/13, JH-11 added, progress bars updated. Cluster summary updated — internal gaps no longer blocking. |
| 2026-05-11 | MethodGate updated to 13/13 — toadStool + squirrel resolved upstream. Cluster 7 at 100%. |
| 2026-05-11 | NestGate Session 60: `content.*` transport parity shipped. GitHub Pages, Releases, REST API dependencies all UNBLOCKED (NestGate `content.put/get/resolve/collections`). petalTongue `backend=nestgate` + SPA UNBLOCKED. All L1 primal debt resolved. |
