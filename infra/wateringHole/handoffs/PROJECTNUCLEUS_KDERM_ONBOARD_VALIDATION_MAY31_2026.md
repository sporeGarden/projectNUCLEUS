# projectNUCLEUS K-Derm Diderm Onboarding — Validation from ironGate

**Date**: 2026-05-31
**From**: projectNUCLEUS (ironGate, covalent LAN)
**To**: primalSpring, cellMembrane, eastGate
**Context**: Complement to flockGate WAN validation

---

## Validation Results

| Step | Status | Detail |
|------|--------|--------|
| Pull wateringHole from forgejo | **PASS** | 57 files, 4,476 insertions — K-Derm standards, impulse system, context braids, Forgejo hooks |
| Pull cellMembrane from forgejo | **PASS** | 11 files, 2,616 insertions — bridge, context, impulse, signal modules |
| Build membrane binary | **PASS** | `cargo build --release` 22s, 7.2 MB binary, all deps pure Rust |
| `membrane manifest.info` | **PASS** | Topology: `diderm: golgiBody → peptidoglycan → golgiBody-ext`, 39 repos, temporal source |
| Temporal sync (cascade-pull) | **21/22 PASS** | 1 FAIL: toadStool DIVERGE (pre-diderm master/main mismatch, see below) |
| Push relay test | **FAIL** | Push mirrors not deployed on golgiBody Forgejo (see gap below) |
| Stale dual-push doc cleanup | **PASS** | 5 docs updated: README, PHASES, EVOLUTION_GAPS, COMPLETE_DEPENDENCY_INVENTORY, baseCamp |

---

## toadStool Divergence (Known, Pre-Diderm)

Local and forgejo are at parity (`844dada31`). Origin (GitHub) has a different commit history — the old master/main mismatch from Wave 60 that was converted on Forgejo but not on GitHub.

```
local    = forgejo/main (844dada31)
origin/main              (8c025107d) — different history, 3 commits ahead
```

**Resolution needed**: One-time force-push from golgiBody-ext (trans face) to align GitHub's toadStool to Forgejo's canonical history. This is a cellMembrane ops action.

---

## Gap: Push Mirrors Not Deployed on golgiBody

**Finding**: Zero push mirrors configured on any repo across all three Forgejo orgs. The diderm relay chain hooks exist in `wateringHole/hooks/forgejo/` but are not installed on golgiBody.

```
$ membrane mirror.push-list sporeGarden/projectNUCLEUS
0 push mirror(s) for sporeGarden/projectNUCLEUS

$ membrane mirror.push-list ecoPrimals/wateringHole
0 push mirror(s) for ecoPrimals/wateringHole
```

**What's missing on golgiBody VPS**:
1. `membrane` binary not installed (not in PATH, not in /usr/local/bin/)
2. `setup-push-mirrors.sh` not run — no Forgejo push mirrors exist
3. `ext-github-push.sh` / `pepti-sync-relay.sh` hooks not installed
4. GitHub SSH key on golgiBody-ext for the trans face push

**Impact**: Gates must still dual-push (`git push forgejo && git push origin`) until push mirrors are deployed. ironGate used direct origin push as fallback for this commit.

**Recommended fix** (cellMembrane/eastGate):
1. Cross-compile membrane binary for VPS (x86_64-unknown-linux-gnu) and deploy to `/usr/local/bin/membrane`
2. Run `setup-push-mirrors.sh` on golgiBody to configure Forgejo push mirrors for all 39 repos
3. Install the relay hooks (pepti-sync-relay, ext-github-push) on peptidoglycan and golgiBody-ext
4. Verify SSH key from golgiBody-ext can push to GitHub

---

## ironGate State Post-Onboarding

| Check | Result |
|-------|--------|
| membrane binary | `~/.local/bin/membrane` (7.2 MB, release build) |
| Manifest version | v2.2.0, wave 63, 39 repos |
| ironGate repo assignment | 22 repos (manifest.repos ironGate) |
| Temporal position | 21/22 parity (toadStool flagged) |
| Push workflow | Forgejo-only intended, origin fallback active until relay deployed |
| Dual-push refs cleaned | 5 docs updated, `forgejo_mirror.sh` description updated |
| Ecosystem norms synced | K-Derm standards, impulse/potential, context braids — all pulled |
| Hardware (for future ABG compute) | i9-14900K, RTX 5070, 96GB RAM, 1G LAN (10G-ready) |

---

*ironGate onboarded to K-Derm diderm topology. Temporal sync operational. Push relay awaiting VPS deployment.*
