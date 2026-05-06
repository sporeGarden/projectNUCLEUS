# Composition Contract — projectNUCLEUS

**Generation**: gen4 — composition and deployment
**Upstream**: primalSpring `DESKTOP_SESSION_MODEL.md`, `MIXED_COMPOSITION_PATTERNS.md`
**Scope**: How deploy graphs compose, how applications layer over the substrate

---

## Principle

The deploy graph is the source of truth for what a composition needs.
Graphs declare **capabilities**, not implementations. The substrate
resolves providers at runtime. Applications are graphs that attach to an
already-running substrate — they never re-declare NUCLEUS primals.

## Graph Structure

Every deploy graph follows the `DeployGraph` schema from primalSpring:

```toml
[graph]
name = "composition_name"
version = "2.0.0"
coordination = "sequential"      # or "parallel"
bonding_policy = "btsp_required" # graph-level BTSP enforcement

[graph.metadata]
security_model = "btsp_enforced"
transport = "uds_preferred"      # uds_only | uds_preferred | tcp_only
composition_model = "nucleated"  # nucleated | cellular | pure
fragments = ["tower_atomic", "node_atomic"]

[graph.bonding_policy]
tower_internal = "covalent"
tower_to_nucleus = "metallic"
encryption_tiers.tower = "full"
encryption_tiers.nucleus = "hmac_plain"

[[graph.nodes]]
name = "primal_name"
binary = "primal_binary"
order = 1
required = true
depends_on = ["other_primal"]
health_method = "health.liveness"
by_capability = "domain"           # capability-first discovery
security_model = "btsp"
tcp_fallback_port = 9100           # Tier 5 discovery
capabilities = ["domain.method1", "domain.method2"]
```

## Node Rules

### Required Fields

Every primal-spawn node must have:
- `by_capability` — the capability domain this primal provides
- `health_method` — must be `"health.liveness"` (Wire Standard L3)
- `security_model` — `"btsp"` for all domain primals (Phase 59)
- `tcp_fallback_port` — canonical Tier 5 port from the port table

### Operation Nodes

Nodes that perform RPC operations (mesh init, NAT traversal, health checks)
rather than spawning primals use the `[graph.nodes.operation]` sub-table
pattern. These are exempt from `by_capability` requirements in guidestone
validation.

### Capability-Based vs Identity-Based

**Prefer `by_capability`** over `name` for provider resolution:

```toml
# Good: portable across gates
by_capability = "tensor"

# Acceptable: sequencing dependency (topological ordering)
depends_on = ["beardog"]

# Avoid: identity-based provider hardcoding
# (use only when multi-family graphs need disambiguation)
name = "barracuda"
```

## Substrate vs Application

The substrate is the always-on NUCLEUS — primals running as shared
infrastructure. Applications are deploy graphs that consume the substrate
without redeclaring its primals.

### Substrate Graphs

Declare `[[graph.nodes]]` for each primal. Used by `deploy.sh` and
biomeOS to bring up the capability plane.

### Application Graphs

Declare only application-specific nodes. Reference substrate capabilities
via `by_capability`. Add `[graph.metadata.application]` to mark as an app:

```toml
[graph]
name = "my_app"
coordination = "sequential"

[graph.metadata]
application = true   # this is an app, not substrate
substrate = "nucleus_complete"

[[graph.nodes]]
name = "app_logic"
by_capability = "compute"
# ... app-specific dispatch
```

## Fragment Composition

Fragments are reusable building blocks with `[fragment]` instead of
`[graph]`. They declare `includes` for nesting:

| Fragment | Particle | Includes |
|----------|----------|----------|
| `tower_atomic` | Electron | — |
| `node_atomic` | Proton | `tower_atomic` |
| `nest_atomic` | Neutron | `tower_atomic`, `provenance_trio` |
| `nucleus` | Full atom | `tower_atomic`, `node_atomic`, `nest_atomic` |

Graphs reference fragments via `[graph.metadata] fragments = [...]`.
primalSpring's `resolve_fragments` merges fragment nodes into the graph
when `resolve = true`.

## Graph Merge

`merge_graphs(base, overlay)` composes a base graph with an overlay.
Use for Nest Atomic + extension patterns (e.g., adding compute to a
storage gate).

## Discovery — 5-Tier Escalation

Compositions use `CompositionContext::discover()` with five tiers:

| Tier | Mechanism | Transport | When |
|------|-----------|-----------|------|
| 1 | Songbird `ipc.resolve` | Any | Full NUCLEUS, cross-gate |
| 2 | biomeOS `capability.discover` | UDS | Local orchestration |
| 3 | UDS filesystem convention | UDS | Local, no orchestrator |
| 4 | Socket registry / manifests | UDS/TCP | Self-registered |
| 5 | TCP probing on well-known ports | TCP | Containers, no UDS |

Graph `transport` metadata drives which tiers are attempted:
- `uds_only` → tiers 1-4
- `uds_preferred` → tiers 1-5 (TCP as fallback)
- `tcp_only` → tier 5 only

## Validation

### Structural (guidestone)

`primalspring_guidestone validate --graph <path>` checks:
- Graph parses as valid TOML
- Nodes with `security_model = "btsp"` have `bonding_policy` at graph level
- UDS transport graphs have `by_capability` on primal nodes
- Fragment references resolve
- Checksums match (source integrity)

### Deployment Readiness

`validate_deployment_readiness()` adds runtime checks:
- Binaries discoverable in plasmidBin
- Required env vars set
- Bonding policy consistency

## References

- gen3: `ECOSYSTEM_ARCHITECTURE.md` §3 (atomics), §4 (bonding)
- gen4: `architecture/COMPOSITION_PATTERNS.md` (PrimalBridge, degradation)
- primalSpring: `specs/DESKTOP_SESSION_MODEL.md` (app lifecycle)
- primalSpring: `specs/MIXED_COMPOSITION_PATTERNS.md` (L0-L3 validation)
- primalSpring: `deploy/mod.rs` (DeployGraph, GraphNode, merge_graphs)
