# Live Science API — JSON-RPC Method Specifications

Tier 2 (notebook-direct) and Tier 3 (petalTongue dashboard) live science
access. Derived from `springs/primalSpring/docs/LIVE_SCIENCE_API.md` (v1.0.0,
May 12, 2026) — the canonical wire contract.

**Current state**: Tier 2 ACTIVE — `toadstool.validate` **IMPLEMENTED** (S250),
`toadstool.list_workloads` **WIRED** (S245+), `barracuda.precision.route` **IMPLEMENTED** (649 tests),
`biomeos.spring_status` **IMPLEMENTED** (v3.54)

**Canonical source**: `springs/primalSpring/docs/LIVE_SCIENCE_API.md`
**Registry**: 415 methods (13 primals, 8 springs)

---

## toadstool.validate

Pre-flight validate a workload against the current compute environment without
executing it. Returns compatibility report including GPU availability, precision
tier, and estimated dispatch time.

**Owner**: ToadStool (S250)
**Transport**: HTTP JSON-RPC on port 9400

```json
{
  "jsonrpc": "2.0",
  "method": "toadstool.validate",
  "params": {
    "workload_path": "/path/to/workload.toml",
    "dry_run": true
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "valid": true,
    "gpu_available": true,
    "precision_tier": "DF64",
    "estimated_dispatch_time_ms": 1200,
    "warnings": [],
    "required_capabilities": ["compute", "shader"],
    "dry_run": true
  }
}
```

**Why this matters**: Notebooks can pre-flight workloads before dispatch.
petalTongue can render live dashboards. lithoSpore modules can validate
against running primals.

---

## toadstool.list_workloads

List registered workload TOMLs and their status.

**Owner**: ToadStool (S245+)

```json
{
  "jsonrpc": "2.0",
  "method": "toadstool.list_workloads",
  "params": {"filter": "active"},
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "workloads": [
      {
        "id": "yukawa_md_force",
        "path": "workloads/yukawa_md.toml",
        "status": "ready",
        "last_run": "2026-05-12T14:30:00Z",
        "precision_tier": "F32"
      }
    ],
    "total": 1
  }
}
```

---

## barracuda.precision.route

Query precision routing advice for a physics domain and hardware profile.

**Owner**: barraCuda (649 tests)
**Transport**: Newline-delimited JSON-RPC on port 9740

```json
{
  "jsonrpc": "2.0",
  "method": "barracuda.precision.route",
  "params": {
    "domain": "lattice_qcd",
    "hardware_hint": "compute"
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "recommended_tier": "DF64",
    "fma_safe": false,
    "requires_compiler": true,
    "hardware_hint": "compute"
  }
}
```

---

## compute.dispatch.submit

Submit a workload for execution (compute trio IPC contract, Wave 8).

**Owner**: ToadStool + coralReef + barraCuda
**Transport**: HTTP JSON-RPC on port 9400

```json
{
  "jsonrpc": "2.0",
  "method": "compute.dispatch.submit",
  "params": {
    "binary_b64": "<base64 shader binary>",
    "dispatch_dims": [256, 1, 1],
    "buffers": [
      { "data_b64": "<base64>", "size": 1024, "binding": 0 }
    ],
    "timeout_ms": 30000
  },
  "id": 1
}
```

---

## Resolved / Implemented — Method Status (May 13, 2026)

| Method | Status | Notes |
|--------|--------|-------|
| `biomeos.spring_status` | **IMPLEMENTED** (v3.54) | Binary discovery + workload counts for Tier 2 notebooks. Also available via `capabilities.list` per primal |
| `nestgate.artifact_query` | **Resolved** | `content.get` (by BLAKE3 hash) and `content.resolve` (by collection+path) — both SHIPPED in Session 60. Provenance chain via `dag.session.get` on rhizoCrypt |
| `rhizocrypt.dag_summary` | **Resolved** | `dag.session.get` (returns full session state) and `dag.session.list` (enumeration) — both SHIPPED. No separate summary method needed |

---

## Wire Notes (May 13 audit)

**bearDog signing**: `auth.sign` expects base64-encoded `message` param, not raw bytes.

**skunkBat audit pipeline**: Use `security.audit_log` (not `defense.audit`). Full chain:
`skunkBat → rhizoCrypt` (DAG provenance) `→ sweetGrass` (braid attribution). JH-5 Phase 3
operational. Wire into deploy graphs via H3-08.

**NestGate domain separation**: `content.*` (CAS: put/get/exists/list/publish/resolve/promote/collections)
and `storage.*` (blob store) are **separate domains** — intentional, not a bug. CAS operates on
content-addressed hashes; storage operates on opaque blob keys.

**BTSP auth pipeline**: 13/13 primals ship `auth.public_key` → Ed25519 → cross-primal verify via
bearDog federation. Wire through `BearDogVerifier` in biomeOS `CompositionContext`.

**composition.deploy.shadow**: biomeOS v3.53 — dry-run graph validation before live deploy.
Local implementation in `deploy_graph.sh` `shadow_deploy()` function.

---

## Evolution Path

```
Tier 0 (operational): CLI binary → stdout [OK]/[FAIL]
Tier 1 (operational): + notebook parses CLI output → matplotlib
Tier 2 (ACTIVE):      + JSON-RPC methods above → notebooks call primals directly
Tier 3 (future):      + petalTongue renders live dashboard from primal APIs
Standalone:           + NestGate serves content → sporePrint self-hosted on NUCLEUS
```

Each tier adds capability without removing previous tiers.

---

## Implementation Status (May 13, 2026)

| Method | Owner | Status | Notes |
|--------|-------|--------|-------|
| `toadstool.validate` | toadStool | **IMPLEMENTED** (S250) | Pre-flight workload validation |
| `toadstool.list_workloads` | toadStool | **WIRED** (S245+) | Workload discovery |
| `compute.dispatch.submit` | toadStool | **WIRED** (Wave 8) | Full dispatch pipeline |
| `barracuda.precision.route` | barraCuda | **IMPLEMENTED** (649 tests) | Precision routing |
| `shader.compile.wgsl` | coralReef | **WIRED** (Sprint 7) | WGSL→PTX/SPIR-V |
| `content.put/get` | nestGate | **SHIPPED** (Session 60) | Content-addressed storage |
| `dag.session.create` | rhizoCrypt | **SHIPPED** | Provenance pipeline |
| `biomeos.spring_status` | biomeOS | **IMPLEMENTED** (v3.54) | Binary discovery + workload counts |
| `nestgate.artifact_query` | nestGate | **Resolved** | Use `content.get`/`content.resolve` (Session 60) |
| `rhizocrypt.dag_summary` | rhizoCrypt | **Resolved** | Use `dag.session.get`/`dag.session.list` |

## Adoption Path

1. ~~**toadStool implements `toadstool.validate`**~~ — **DONE** (S250)
2. ~~**primalSpring adds gate tests**~~ — Wave 8 closure
3. **projectNUCLEUS wires Tier 2** notebooks to use JSON-RPC instead of CLI
4. **lithoSpore modules** gain live validation via `toadstool.validate`
5. **Foundation threads** consume live provenance chains
