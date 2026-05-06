# Execution Model — projectNUCLEUS

**Generation**: gen4 — composition and deployment
**Upstream**: primalSpring `DESKTOP_NUCLEUS_DEPLOYMENT.md`, `DESKTOP_SESSION_MODEL.md`
**Scope**: How primals are spawned, ordered, monitored, and torn down on a gate

---

## Principle

projectNUCLEUS owns the **substrate lifecycle**. Primals are infrastructure —
they start before applications, run continuously, and shut down after
applications exit. No application should start, manage, or directly reference
a primal process. The substrate is invisible.

## Startup Ordering

Primals launch in **topological waves** derived from `depends_on` edges in
the deploy graph. primalSpring's `topological_waves()` (Kahn's algorithm)
computes these; `deploy.sh` follows the same ordering.

### Phase 0: Orchestration Substrate

biomeOS Neural API is the coordination surface. It is expected to be running
before NUCLEUS primals spawn (marked `spawn = false` in graphs). If not
present, primals self-register via Songbird or UDS convention.

### Phase 1: Tower Atomic (Electron)

**BearDog** (security) → **Songbird** (discovery)

BearDog requires `BEARDOG_FAMILY_SEED` or a `.beacon.seed` file. It must
be the first primal to start — all other primals depend on it for BTSP
handshake and identity.

Songbird requires a live BearDog socket for its security provider. Once
running, it provides the Tier 1 discovery surface (`ipc.resolve`).

### Phase 2: Domain Primals

Order within this phase is flexible; all depend on Tower.

**Node Atomic (Proton)**: ToadStool → barraCuda, coralReef (parallel)
**Nest Atomic (Neutron)**: NestGate → rhizoCrypt → loamSpine → sweetGrass

### Phase 3: Meta Tier

**Squirrel** (AI) depends on Tower + ToadStool + NestGate.
**petalTongue** (visualization) depends on biomeOS Neural API.

### Phase 4: Coordination

**primalSpring** (coordination validator) depends on the full substrate.
Marked `spawn = false` — runs as validation tool, not continuous process.

## Health Model

### Process Health

Every primal exposes `health.liveness` via JSON-RPC. The substrate
verifies health in two passes:

1. **Post-startup**: PID check + `health.liveness` RPC on TCP fallback port
2. **Continuous**: Periodic liveness probes (when biomeOS is orchestrating)

### Deployment Readiness

Before launching, `validate_deployment_readiness()` checks four categories:

| Category | What | Blocking |
|----------|------|----------|
| **Structure** | Graph TOML exists and parses | Yes |
| **BinaryMissing** | Primal binary not found in plasmidBin | Yes |
| **EnvMissing** | Required env var not set | Yes (BEARDOG_FAMILY_SEED) |
| **BondingInconsistent** | BTSP nodes without BearDog in composition | Yes |

Override with `NUCLEUS_SKIP_READINESS=1` for development.

### Graceful Degradation

Optional primals (barraCuda, coralReef, provenance trio, petalTongue) may
fail to start without blocking the composition. The substrate reports them
as degraded, not failed. Required primals (BearDog, Songbird, ToadStool,
NestGate, Squirrel) block the composition.

## Shutdown

Reverse topological order. Meta tier → domain primals → Tower. Socket
cleanup after process termination. Stale UDS sockets from previous runs
are cleaned before startup.

## Seed Lifecycle

Family seeds are managed by plasmidBin's `seed_workflow.sh`:

1. **Init**: Creates family seed + beacon seed if not present
2. **Add node**: Creates per-node lineage seed from family seed
3. **Persistence**: `${XDG_CONFIG_HOME}/biomeos/family/`

The family ID is an 8-hex-char truncation of the genesis seed. It scopes
all UDS socket paths: `${RUNTIME_DIR}/biomeos/<primal>-${FAMILY_ID}.sock`.

## Gate Configuration

Gate TOMLs (`gates/<name>.toml`) declare hardware, composition, and port
overrides. When `--gate <name>` is passed to `deploy.sh`, the gate TOML
overrides default ports.

## Composition Profiles

| Composition | Primals | Graph | Use Case |
|-------------|---------|-------|----------|
| `tower` | BearDog, Songbird | `tower_atomic.toml` | Tunnel endpoint, intake node |
| `node` | Tower + ToadStool, barraCuda, coralReef | `node_atomic_compute.toml` | Compute workstation |
| `nest` | Tower + NestGate, rhizoCrypt, loamSpine, sweetGrass | `nest_atomic.toml` | Storage + provenance |
| `full` | All 10 domain + Squirrel | `nucleus_complete.toml` | Full NUCLEUS gate |

## References

- gen3: `ECOSYSTEM_ARCHITECTURE.md` §3 (NUCLEUS Atomic Composition)
- primalSpring: `specs/DESKTOP_NUCLEUS_DEPLOYMENT.md` (operational lifecycle)
- primalSpring: `deploy/validation.rs` (DeploymentReadiness struct)
- primalSpring: `deploy/profiles.rs` (PrimalDeployProfile generation)
