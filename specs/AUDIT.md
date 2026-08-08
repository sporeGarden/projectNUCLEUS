# Specs Audit — Wave 157a Role Refinement

**Filed**: 2026-08-08
**Purpose**: Classify 23 specs as KEEP, MIGRATE (to wateringHole), or FOSSILIZE

## Classification

### KEEP (15) — NUCLEUS-specific operational docs

| Spec | Reason |
|------|--------|
| `EVOLUTION_GAPS.md` | Living NUCLEUS gap tracker with validation baselines |
| `SECURITY_VALIDATION.md` | Active NUCLEUS security posture and pen test methodology |
| `TIERED_ACCESS_ARCHITECTURE.md` | NUCLEUS Tier 1-4 remote access deployment |
| `LIVE_SCIENCE_API.md` | Local implementation-status mirror for JSON-RPC methods |
| `CI_EVALUATION.md` | Active NUCLEUS CI operational evaluation |
| `FUZZ_EVOLUTION.md` | NUCLEUS-specific security evolution roadmap |
| `SOVEREIGNTY_VALIDATION_PROTOCOL.md` | Master validate-then-replace protocol |
| `GATE_PORTABILITY.md` | Operational gate-migration baseline |
| `TUNNEL_EVOLUTION.md` | Active CF → primal replacement roadmap |
| `SCIENCE_DISPATCH_MAP.md` | Spring-to-gate hardware mapping |
| `SHARED_WORKSPACE.md` | ABG lab operational policy |
| `VALIDATION_PLAYBOOK.md` | Operational validation companion |
| `VALIDATION_RESULTS.md` | Recorded Phase 1-2a validation evidence |
| `TIER2_CEREMONY_DESIGN.md` | Tiered-access ceremony spec |
| `NOTEBOOK_ELEVATION.md` | Spring validation → notebook pattern |

### MIGRATE (5) — Normative ecosystem standards → wateringHole

| Spec | Target | Reason |
|------|--------|--------|
| `TRANSPORT_MATRIX.md` | `wateringHole/specs/` | Per-primal wire/transport spec — ecosystem-wide normative |
| `COMPOSITION_CONTRACT.md` | `wateringHole/specs/` | Gen4 composability rules — ecosystem law |
| `EXECUTION_MODEL.md` | `wateringHole/specs/` | Substrate lifecycle spec — any gate, not just NUCLEUS |
| `PROVENANCE_CONTRACT.md` | `wateringHole/specs/` | Trio contract — ecosystem-wide provenance |
| `INVISIBILITY_STANDARD.md` | `wateringHole/specs/` | Capability-first product standard — ecosystem law |

### FOSSILIZE (3) — Completed snapshots

| Spec | Reason |
|------|--------|
| `DARKFOREST_OUTER_MEMBRANE_REPORT.md` | Wave 136b scan snapshot; living state in `SECURITY_VALIDATION.md` |
| `COMPLETE_DEPENDENCY_INVENTORY.md` | Static Wave 56 dependency snapshot; living state in `EVOLUTION_GAPS.md` |
| `README.md` | Stays but should be updated to reflect this audit |

## Action Required

1. **Overwatch**: Copy 5 MIGRATE specs to `wateringHole/specs/` as normative standards
2. After absorption, replace originals with reference stubs pointing to wateringHole
3. Move 2 FOSSILIZE specs to `validation/archive/`
4. Update `specs/README.md` to reflect reduced scope
