# projectNUCLEUS Specifications

Local specs for execution, composability, and invisibility of the primal
substrate. These codify patterns from upstream (primalSpring specs,
wateringHole standards, gen3 architecture, gen4 composition) into
projectNUCLEUS's own operational terms.

## Documents

| Spec | Scope |
|------|-------|
| [EXECUTION_MODEL.md](EXECUTION_MODEL.md) | Substrate lifecycle: startup ordering, health probes, shutdown, seed management, gate configuration |
| [COMPOSITION_CONTRACT.md](COMPOSITION_CONTRACT.md) | Deploy graph structure, capability-based discovery, substrate vs application graphs, fragment composition |
| [INVISIBILITY_STANDARD.md](INVISIBILITY_STANDARD.md) | Products never see primals — capability-first routing, no identity coupling, degradation model |
| [TRANSPORT_MATRIX.md](TRANSPORT_MATRIX.md) | Per-primal wire protocol, JSON-RPC framing, BTSP requirements, hash encoding, port table |
| [PROVENANCE_CONTRACT.md](PROVENANCE_CONTRACT.md) | Trio operations (hash → DAG → ledger → braid), key namespace, PROV-O mapping |
| [TUNNEL_EVOLUTION.md](TUNNEL_EVOLUTION.md) | Systematic Cloudflare → primal replacement: where we are, where we're going, validation protocol |
| [VALIDATION_RESULTS.md](VALIDATION_RESULTS.md) | Phase 1 pipeline results, braid analysis, rigor/reproducibility/security assessment, bugs found |
| [NOTEBOOK_ELEVATION.md](NOTEBOOK_ELEVATION.md) | How springs elevate CLI validations into visual notebooks — output contract, tier access, evolution path |
| [SECURITY_VALIDATION.md](SECURITY_VALIDATION.md) | Three-layer pen testing (below/at/above primals), skunkBat integration, baseline results |
| [SHARED_WORKSPACE.md](SHARED_WORKSPACE.md) | ABG shared workspace visibility model, access tiers, reviewer access, sporePrint integration |
| [LIVE_SCIENCE_API.md](LIVE_SCIENCE_API.md) | JSON-RPC method specs for Tier 2/3 live science — toadstool.validate, barracuda.compute, etc. |

## Architecture Lineage

```
gen3 ECOSYSTEM_ARCHITECTURE.md          (what primals are, how they compose)
  └── Paper 23: mass-energy-information  (particle metaphor: Tower/Node/Nest = e/p/n)
       └── MIXED_COMPOSITION_PATTERNS    (L0-L3 validation layers, bonding sketches)

gen4 architecture/                       (who uses it — products, guidestone, artifacts)
  ├── COMPOSITION_PATTERNS.md            (PrimalBridge, degradation, zero-import)
  ├── GUIDESTONE.md                      (5 properties: deterministic, traceable, self-verifying)
  └── CREATIVE_SURFACE_ARCHITECTURE.md   (sporeGarden product layer)

primalSpring specs/                      (runtime contracts)
  ├── DESKTOP_NUCLEUS_DEPLOYMENT.md      (substrate lifecycle — Phase 0-4)
  ├── DESKTOP_SESSION_MODEL.md           (app graphs on substrate)
  ├── LIVE_GUI_COMPOSITION_PATTERN.md    (petalTongue visualization)
  └── NUCLEUS_VALIDATION_MATRIX.md       (columns A-P acceptance)

wateringHole/                            (ecosystem standards)
  ├── PRIMAL_IPC_PROTOCOL.md             (JSON-RPC contract)
  ├── BTSP_PROTOCOL_STANDARD.md          (Phase 3 AEAD)
  ├── CAPABILITY_DISCOVERY_STANDARD.md   (5-tier escalation)
  └── ECOSYSTEM_EVOLUTION_CYCLE.md       (water cycle: mountain → spring → delta)
```

## Principles

1. **Execution**: projectNUCLEUS owns the substrate lifecycle. Primals
   start before apps, run continuously, shut down after apps exit.

2. **Composability**: Deploy graphs are the source of truth. Graphs
   declare capabilities, not implementations. Fragments compose into
   atomics, atomics compose into NUCLEUS.

3. **Invisibility**: Products built on NUCLEUS never reference primals
   by name, port, or binary. They consume capabilities through the
   Neural API. The substrate is invisible infrastructure.

## Upstream Sources

These specs are **derived from** upstream, not copies of upstream. When
upstream evolves (new primalSpring phase, new wateringHole standard),
review and update these specs to reflect the change in projectNUCLEUS's
operational terms.

| Source | Location | What to watch |
|--------|----------|---------------|
| primalSpring specs | `springs/primalSpring/specs/` | Runtime contracts, validation matrix |
| wateringHole | `infra/wateringHole/` | IPC, BTSP, capability standards |
| gen3 architecture | `infra/whitePaper/gen3/` | Ecosystem paper, particle model |
| gen4 architecture | `infra/whitePaper/gen4/` | Composition, guidestone, products |
| wateringHole handoffs | `infra/wateringHole/handoffs/` | Per-primal operational findings |
