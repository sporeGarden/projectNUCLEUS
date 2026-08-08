# Workloads — Handoff to Spring Repos

**Status**: PENDING MIGRATION
**Filed**: 2026-08-08 Wave 157a role refinement
**Owner**: overwatch (primalSpring / spring repo maintainers)

## Context

These 43 toadStool dispatch specs (8 spring categories) originated in
projectNUCLEUS during the atomics evolution phase when composition validation
was being proven. Now that:

- primalSpring owns 197+ composition scenarios
- Each spring has its own repo with tests and dispatch patterns
- toadStool dispatch is stable and gate-agnostic

...these workload TOMLs belong in their respective spring repos or in
primalSpring as canonical workload patterns.

## Migration Targets

| Directory | Target Repo | Org |
|-----------|-------------|-----|
| `wetspring/` (15 TOMLs) | `springs/wetSpring` | syntheticChemistry |
| `healthspring/` (6 TOMLs) | `springs/healthSpring` | syntheticChemistry |
| `hotspring/` (6 TOMLs) | `springs/hotSpring` | syntheticChemistry |
| `neuralspring/` (5 TOMLs) | `springs/neuralSpring` | syntheticChemistry |
| `airspring/` (6 TOMLs) | `springs/airSpring` | syntheticChemistry |
| `ludospring/` (2 TOMLs) | `springs/ludoSpring` | syntheticChemistry |
| `groundspring/` (1 TOML) | `springs/groundSpring` | syntheticChemistry |
| `templates/` (3 TOMLs) | `springs/primalSpring` | syntheticChemistry |

## Action Required

1. Each spring repo maintainer copies their workload TOMLs from this directory
2. Once all TOMLs are absorbed upstream, this directory moves to fossilRecord
3. projectNUCLEUS removes `workloads/` from active scope

## Note

All workload TOMLs are gate-agnostic (use `$SPRINGS_ROOT` / `$ECOPRIMALS_ROOT`).
No projectNUCLEUS-specific configuration — they are pure toadStool dispatch specs.
