# Forgejo CI Activation — Runbook

**Wave 74** | Owner: ironGate

## Status

| Repo | Workflow | Runner | Status |
|------|----------|--------|--------|
| projectNUCLEUS | `.forgejo/workflows/ci.yml` | ubuntu-latest | **ACTIVE** since Wave 69 |
| primalSpring | `.forgejo/workflows/ci.yml` | ubuntu-latest | **DEPLOYED** Wave 74 |
| bearDog | `.forgejo/workflows/ci.yml` | ubuntu-latest | **DEPLOYED** Wave 74 |

## Step 1: Register Self-Hosted Runner

Run on ironGate (or any gate with Rust toolchain):

```bash
# Get token from: https://git.primals.eco/-/admin/runners
# (Site Admin → Actions → Runners → Create new Runner)

cd projectNUCLEUS
./infra/ci/provision-runner.sh --token <RUNNER_TOKEN>
```

This:
1. Downloads `forgejo-runner` v6.3.1
2. Registers with git.primals.eco (labels: `self-hosted,linux,x86_64,rust`)
3. Creates + starts `forgejo-runner.service` (systemd user unit)

## Step 2: Verify Runner Registration

```bash
# Check local service
systemctl --user status forgejo-runner

# Check Forgejo UI
# https://git.primals.eco/-/admin/runners
# Runner should show as "Online" with green indicator
```

## Step 3: Test End-to-End

Push a trivial change to primalSpring and verify the CI triggers:

```bash
cd springs/primalSpring
echo "" >> README.md
git add README.md && git commit -m "ci: trigger Forgejo Actions test"
git push forgejo main
```

Then check: `https://git.primals.eco/sporeGarden/primalSpring/actions`

## Step 4: Switch to Self-Hosted Runner

Once the runner is verified, update workflows:

```yaml
# Change:
runs-on: ubuntu-latest
# To:
runs-on: [self-hosted, linux, x86_64, rust]
```

## Step 5: Expand Coverage

Repos queued for Forgejo CI activation (priority order):

1. primalSpring — **DONE** (Wave 74)
2. bearDog — **DONE** (Wave 74)
3. biomeOS — workspace, needs `cargo test -p biomeos-core`
4. songBird — workspace, needs `cargo test --workspace`
5. cellMembrane — workspace with multiple crates
6. Remaining primals — one workflow per primal

## Artifact Storage

Currently validate-only (no artifacts stored). Evolution:

| Phase | Artifacts | Storage |
|-------|-----------|---------|
| Now | None | — |
| Next | Test reports, coverage | Forgejo upload-artifact |
| Future | Release binaries | NestGate CAS |

## Secrets

| Secret | Repo | Purpose | Migration |
|--------|------|---------|-----------|
| `SPOREPRINT_DISPATCH_TOKEN` | projectNUCLEUS (GitHub) | sporePrint notify | → Forgejo webhook |

No secrets needed for basic CI. Future bearDog BTSP signing requires
self-hosted runner with UDS access to bearDog.
