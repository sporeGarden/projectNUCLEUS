# Invisibility Standard — projectNUCLEUS

**Generation**: gen4 — composition and deployment
**Upstream**: primalSpring `CAPABILITY_ROUTING_TRACE.md`, gen4 `COMPOSITION_PATTERNS.md`
**Scope**: Primals are invisible to products. Capability-first. No identity coupling.

---

## Principle

A product built on NUCLEUS must not know which primals exist, how many
there are, which binary provides a capability, or what port a primal
listens on. Products consume **capabilities** through the **Neural API**
or through **`capability.call`** — never through primal-specific imports,
ports, or addresses.

This is the **gen4 consumer contract**: primals are infrastructure, not API.

## Rules

### R1: No primal names in product code

Product code must not reference `beardog`, `songbird`, `rhizocrypt`, or
any other primal by name. Use capability domains instead:

| Instead of | Use |
|------------|-----|
| `connect("beardog:9100")` | `capability.call("security.encrypt", ...)` |
| `curl localhost:9200/ipc.resolve` | `capability.discover("compute")` |
| `import rhizocrypt_client` | `capability.call("dag.anchor", ...)` |

### R2: No hardcoded ports in product code

Ports are a Tier 5 fallback for containerized environments. Products use
discovery tiers 1-4 (Songbird, Neural API, UDS, socket registry).

Port knowledge belongs exclusively to:
- `deploy.sh` (substrate launcher)
- `gates/*.toml` (per-gate overrides)
- Deploy graph `tcp_fallback_port` fields

### R3: No transport assumptions in product code

Products must not assume UDS, TCP, or HTTP. The substrate negotiates
transport. Wire Standard L3 ensures every primal's `capabilities.list`
reports its `protocol` and `transport` — the Neural API uses this to
route appropriately.

### R4: Self-knowledge only

Each primal knows:
- Its own capability domain
- Its own methods
- Its own health status

It discovers other primals through the 5-tier escalation hierarchy at
runtime. No primal hardcodes another primal's address, port, or name
in its source code.

### R5: Deploy graphs declare needs, not providers

Prefer `by_capability` over `name` in graph nodes. The composition layer
resolves which primal satisfies a capability at deployment time:

```toml
# Correct: declare need
by_capability = "tensor"

# Incorrect: declare provider
name = "barracuda"
```

Exception: `depends_on` edges use names for topological ordering because
ordering is structural, not semantic.

### R6: Application graphs do not contain substrate nodes

An application graph (game, pipeline, visualization) must not declare
BearDog, Songbird, ToadStool, or any NUCLEUS primal as a node. The
substrate is already running. Application graphs reference substrate
capabilities via `by_capability`.

## Degradation Model

When a capability has no provider (primal not running, not in composition),
the product must handle the absence:

```
capability.call("tensor.infer", input)
  → if available: result
  → if unavailable: Err(CapabilityUnavailable("tensor"))
```

Products implement `call_or_default`, `check_skip`, or explicit
degradation per the gen4 resilience contract:

| Strategy | When |
|----------|------|
| `call_or_default(capability, fallback)` | Non-critical enrichment |
| `check_skip(capability)` | Optional pipeline stage |
| `require(capability)` | Hard dependency — fail if missing |

## Mocks

Mocks of primal behavior are **isolated to testing**. Any mock that
appears in production code must be evolved to a complete implementation
or replaced with the real capability call.

Test mocks use the same `capability.call` interface with a mock transport
that returns fixtures. This validates the product's capability contract
without the real primal.

## Validation

### How to check for identity coupling

Search product code for:
- Primal names as string literals (`"beardog"`, `"songbird"`, etc.)
- Hardcoded ports (`9100`, `9200`, `9300`, etc.)
- Direct socket paths (`/run/biomeos/beardog-*.sock`)
- Binary names (`beardog_primal`, `songbird_primal`, etc.)

Any match is a violation. Convert to capability-based routing.

### Exceptions

Internal projectNUCLEUS infrastructure (`deploy.sh`, gate TOMLs, deploy
graphs) necessarily references primal names — this is the substrate layer
that implements invisibility for everything above it. The boundary is:

```
├── Infrastructure (names OK)      ← deploy.sh, graphs, gates
│   └── Substrate layer
├── Product code (names NEVER)     ← apps, pipelines, UIs
│   └── Capability calls only
```

## References

- gen3: `ECOSYSTEM_ARCHITECTURE.md` §5 (Neural API and Dark Forest)
- gen4: `architecture/COMPOSITION_PATTERNS.md` (PrimalBridge pattern)
- gen4: `architecture/CREATIVE_SURFACE_ARCHITECTURE.md` (product layer)
- primalSpring: `specs/CAPABILITY_ROUTING_TRACE.md` (debt trace)
- primalSpring: `specs/GEN4_COMPOSITION_AUDIT.md` (drift detection)
- wateringHole: `CAPABILITY_DISCOVERY_STANDARD.md` (routing contract)
