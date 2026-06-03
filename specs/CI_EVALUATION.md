# Forgejo Actions CI Evaluation — projectNUCLEUS

**Wave 72 (2026-06-03)** | Owner: ironGate | Status: ACTIVE

## Current State

Forgejo Actions is **PRIMARY CI** for projectNUCLEUS since Wave 69.
GitHub Actions retains only `notify-sporeprint.yml` (postprimordial shadow).

### What runs today

| Step | Scope | Tests |
|------|-------|-------|
| darkforest — fmt + clippy + test | validation/darkforest | 140 |
| tunnelKeeper — fmt + clippy + test | validation/tunnelKeeper | 48 |
| nucleus-primals — fmt + clippy + test | deploy/nucleus-primals | 7 |
| nucleus-deploy — fmt + clippy + test | deploy/nucleus-deploy | 47 |
| Shell syntax check | deploy/*.sh | ~10 scripts |
| **Total Rust tests** | **4 crates** | **242** |

Toolchain: pinned via `rust-toolchain.toml` (channel 1.96.0).
Runner: `ubuntu-latest` (Forgejo shared runner).
Trigger: push to `main`, all pull requests.

## Key Questions & Answers

### 1. Runner Provisioning

**Current**: `ubuntu-latest` — Forgejo's built-in shared runner.

**Self-hosted path**: Register a runner on ironGate (or VPS) via:
```
forgejo-runner register --instance https://git.primals.eco \
  --token <runner-token> --labels "self-hosted,linux,x86_64,rust"
```

**Recommendation**: Stay on `ubuntu-latest` for now. Self-hosted runner
adds value when:
- Builds need GPU (coralReef, barraCuda validation)
- Release builds need signing (bearDog BTSP)
- genomeBin harvest needs all 14 primal source trees (not in this repo)

**Fallback pattern** (already in ci.yml header):
```yaml
runs-on: [self-hosted, linux, x86_64, rust]
# If no self-hosted, Forgejo queues but does not fall back automatically.
# Manual: change to ubuntu-latest if self-hosted is down.
```

### 2. Artifact Storage

**Current**: No artifacts stored — CI is validate-only.

**Options for future artifact needs** (genomeBin binaries, coverage reports):

| Option | Pros | Cons |
|--------|------|------|
| Forgejo Actions `upload-artifact` | Native, no infra | 30-day retention, Forgejo quota |
| NestGate CAS (content-addressable) | Sovereign, permanent, primal-native | Needs NestGate on runner or SSH push |
| Local runner filesystem | Zero config | Single-machine, no replication |
| plasmidBin GitHub Release | Existing infra | Cage bar (GitHub dependency) |

**Recommendation**: Short term — Forgejo `upload-artifact` for CI reports.
Medium term — NestGate CAS for genomeBin release binaries (sovereign).

### 3. Secret Management

**Current**: One GitHub secret (`SPOREPRINT_DISPATCH_TOKEN`) used by the
shadow `notify-sporeprint.yml` workflow. No Forgejo secrets configured.

**Migration path**:
1. Create `SPOREPRINT_WEBHOOK_SECRET` in Forgejo repo settings
2. Replace GitHub repository-dispatch with Forgejo webhook to sporePrint
3. Archive `notify-sporeprint.yml` on GitHub

**Future (bearDog BTSP)**: When primal signing is needed in CI:
- Runner authenticates to bearDog via BTSP `secrets.seal`/`secrets.unseal`
- No secrets stored in Forgejo settings — all mediated through primal IPC
- Requires self-hosted runner with bearDog access (UDS or localhost TCP)

### 4. GitHub → Forgejo Workflow Migration

**Compatibility**: Forgejo Actions is ~95% compatible with GitHub Actions.

| Feature | GitHub | Forgejo | Notes |
|---------|--------|---------|-------|
| `actions/checkout@v4` | native | works | Forgejo mirrors github.com actions |
| `runs-on: ubuntu-latest` | native | works | Forgejo provides base images |
| `upload-artifact` | native | works | `forgejo/upload-artifact` also exists |
| `repository-dispatch` | native | **not available** | Use Forgejo webhooks instead |
| Matrix builds | native | works | Same syntax |
| Reusable workflows | native | works | Same syntax |
| OIDC tokens | native | partial | Forgejo support varies |

**Migration for ecosystem (73 repos, H3-03)**:
- projectNUCLEUS: **DONE** (primary on Forgejo since Wave 69)
- sporePrint: needs Forgejo webhook (replaces `notify-sporeprint.yml`)
- Other repos: evaluate per-repo — most are build/test only, direct port

### 5. What's NOT in CI Yet

| Gap | Priority | Blocker |
|-----|----------|---------|
| `cargo deny` (darkforest, tunnelKeeper) | Medium | None — add step |
| Coverage reporting (llvm-cov) | Low | Upload target needed |
| Dark Forest structural gate (33 checks) | Low | Needs VPS or self-hosted runner |
| genomeBin harvest validation | Low | Needs ecoPrimals source tree |
| `notify-sporeprint` on Forgejo | Medium | Webhook setup on VPS |

## Implementation (Wave 73)

Runner provisioning script: `infra/ci/provision-runner.sh`
- Downloads forgejo-runner, registers with git.primals.eco
- Creates systemd user service for always-on operation
- Labels: `self-hosted, linux, x86_64, rust`

Activation runbook: `infra/ci/ACTIVATION.md`
- Step-by-step runner registration, testing, and expansion guide

Deployed workflows (Wave 74):
- projectNUCLEUS: `.forgejo/workflows/ci.yml` — **ACTIVE** since Wave 69
- primalSpring: `.forgejo/workflows/ci.yml` — **DEPLOYED** Wave 74
- bearDog: `.forgejo/workflows/ci.yml` — **DEPLOYED** Wave 74

## Evolution Path

1. **Now**: Forgejo Actions primary, 4 crates, 242 tests
2. **Next**: Provision runner on ironGate, activate primalSpring CI
3. **Then**: Add `cargo deny` step, port sporePrint notification to webhook
4. **Future**: Self-hosted runner for release signing + genomeBin harvest
5. **Stadial**: bearDog BTSP-mediated secrets, NestGate artifact storage
