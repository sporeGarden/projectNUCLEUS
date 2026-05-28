# projectNUCLEUS Wave 55 Absorption — May 27, 2026

> **Note**: Wave 56 absorption handoff is the current canonical handoff.
> This document records the Wave 55 niche climate absorption.

**Source**: primalSpring spring river delta — Wave 55 niche climate audit
**Scope**: Registry 458→460, NC-1→NC-5 tracked, biomeOS v3.78, southGate NC-2, cellMembrane NC-3

---

## What Changed

### Registry: 458 → 460 methods (Wave 55)

Two new methods registered:
- `nucleus.ingest_spore` — NUCLEUS spore ingest gateway (biomeOS NC-1.1)
- `nucleus.emit_spore` — NUCLEUS spore emit gateway (biomeOS NC-1.2)

primalSpring v0.9.30, 56 scenarios, 813 tests (797 pass, 16 live-tier, 2 ignored).

### Docs updated (458→460, biomeOS v3.54→v3.78)

| File | Changes |
|------|---------|
| `README.md` | Banner: 460 methods Wave 55, ACTIVE 2026-05-27, biomeOS v3.78 |
| `PHASES.md` | Status banner, registry lineage, primalSpring reference |
| `specs/EVOLUTION_GAPS.md` | Header, Tier 2 block, scoring bars, Wave 55 changelog entry |
| `specs/LIVE_SCIENCE_API.md` | Registry line |
| `whitePaper/baseCamp/README.md` | Method counts (445→460, 458→460) |
| `experiments/README.md` | Wave synced to v0.9.30, niche climate goals added |
| `graphs/README.md` | Signal graph section for `nest_ingest_spore`, date bump |

### Niche Climate Gaps Tracked (NC-1→NC-5)

Added to `EVOLUTION_GAPS.md` — new "Niche Climate Gaps" section:

| NC | Description | Owner | Status |
|----|-------------|-------|--------|
| NC-1 | postPrimordial spore gateway | biomeOS + lithoSpore | BLOCKED — scaffolded v3.77, NC-1.3 done, NC-1.4 open |
| NC-2.1 | southGate 13/13 health | Songbird + wetSpring | BLOCKED — 7/13 responding |
| NC-2.3 | Cross-gate capability via cellMembrane | projectNUCLEUS + cellMembrane | OPEN |
| NC-3.1 | cellMembrane Nest docs sync | cellMembrane | **DONE** — VPS_STATE, GLACIAL_SHIFT, membrane.toml all updated |
| NC-3.2 | `membrane.toml` → `composition = "nest"` | cellMembrane | **DONE** — published with signal channel enabled |
| NC-3.3 | knot-dns NS cutover | cellMembrane + registrar | OPEN — knot-dns running, NS record pending |
| NC-3.4 | Forgejo Releases | cellMembrane + plasmidBin | OPEN |
| NC-3.5 | sporePrint living content | cellMembrane + bearDog | BLOCKED — needs `content.*` scope |

### cellMembrane NC-3 Status

cellMembrane ops docs are **synced** to Nest Atomic reality (updated May 27):
- `VPS_STATE.md`: "Deployed composition: Nest Atomic (Wave 38)"
- `membrane.toml`: `composition = "nest"`, signal channel enabled, K-Derm diderm topology
- `GLACIAL_SHIFT_TRACKER.md`: Criterion #3 (Nest expansion) marked RESOLVED

Remaining NC-3 items: NS cutover (NC-3.3), Forgejo releases (NC-3.4), sporePrint (NC-3.5 — blocked on bearDog).

### Signal Graph Awareness

`nest_ingest_spore.toml` documented in `graphs/README.md`. Canonical source is biomeOS
(`primals/biomeOS/graphs/signals/`). Six sequential nodes composing existing primal
capabilities: NestGate → rhizoCrypt → loamSpine → sweetGrass → BearDog.

ironGate VPS is a deployment target for column U when biomeOS gateway completes.

---

## Pull Status

| Repo | Status | Notes |
|------|--------|-------|
| primalSpring | PULLED | v0.9.30, exp115 landed, NICHE_CLIMATE_EVOLUTION spec |
| wateringHole | PULLED | Wave 55 handoff + southGate redeploy + wetSpring issues |
| airSpring | PULLED | Wave 48 covalent mesh handoff |
| groundSpring | PULLED | Gate deployment handoff |
| hotSpring | **DIVERGED** | Needs rebase (local divergence from main) |
| songBird | PULLED | Neural announce + remote dispatch modules |
| biomeOS | PULLED | v3.78, `nest_ingest_spore.toml` |
| coralReef | PULLED | 1317 insertions, PTX ray query codegen |
| sourDough | PULLED | notify-plasmidbin workflow |
| lithoSpore | PULLED | CHASSIS.md + PSEUDOSPORE_STANDARD.md specs |
| toadStool | **FAILED** | master/main branch mismatch (persistent) |
| All others | PULLED | Up to date |

---

## Upstream Gaps (for primalSpring / team awareness)

| Gap | Owner | Notes |
|-----|-------|-------|
| biomeOS NC-1.4: use `pseudospore-core` instead of inline validation | biomeOS | primalSpring flagged — push biomeOS to swap |
| biomeOS signal graph divergence | biomeOS / primalSpring | Two copies (biomeOS vs primalSpring) differ on store step and bonding policy |
| southGate 7/13 health | Songbird + wetSpring | Wave 54 redeploy fixes ready but may not be applied yet |
| hotSpring divergence | hotSpring | Local repo diverged from remote main — needs rebase |
| toadStool master/main | toadStool / primalSpring | Config expects master, remote only has main |
| DOWNSTREAM_PATTERN_GUIDE still says 445 | primalSpring | Upstream doc — should be 460 |

---

## Local Open Gaps (projectNUCLEUS)

| Gap | Priority | Blocks |
|-----|----------|--------|
| `biomeos nucleus ingest` on ironGate VPS | HIGH | Column U — blocked on biomeOS NC-1.1 |
| southGate stabilization (NC-2.1) | HIGH | Multi-gate mesh — blocked on southGate 7/13 |
| biomeGate elevation (NC-2.6) | MEDIUM | Needs HBM2 hardware + hotSpring coordination |
| flockGate WAN provisioning (H3-11) | MEDIUM | Cross-family ionic contract E2E |
| Forgejo binary releases (NC-3.4 / H3-02) | LOW | Coordinate with plasmidBin |

---

## Stadial Entry Requirements

From `NICHE_CLIMATE_EVOLUTION.md`:

| Requirement | Current | Target |
|-------------|---------|--------|
| NC-1: Column U for 2+ springs | 0 springs | 2+ |
| NC-2: 3+ gates meshed | 1 gate healthy (ironGate) | 3+ |
| NC-4: All 4 gates healthy | ironGate healthy, southGate 7/13, biomeGate 9/13 | 4/4 |

**projectNUCLEUS next actions when unblocked:**
1. Deploy biomeOS v3.77+ to ironGate VPS when NC-1.1 ships
2. Run `biomeos nucleus ingest` against hotSpring pseudoSpore v1.6.1
3. Verify `receipts/nucleus_ingest.toml` closes column U
4. Coordinate with primalSpring for live `s_covalent_mesh` when southGate reaches 13/13
