# Provenance Contract — projectNUCLEUS

**Generation**: gen4 — composition and deployment
**Upstream**: wateringHole `SWEETGRASS_SPRING_BRAID_PATTERNS.md`, `PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md`
**Scope**: Trio operations, key namespace, hash encoding, braid lifecycle

---

## Principle

The provenance trio (rhizoCrypt, loamSpine, sweetGrass) provides an
immutable, verifiable record of all computation performed through NUCLEUS.
Every workload dispatched through ToadStool produces a provenance chain:
hash → DAG → ledger → braid. The trio operates as **Nest Atomic
infrastructure** — invisible to products, present on every gate that
runs workloads.

## Trio Roles

| Primal | Capability | Role | Primary Methods |
|--------|-----------|------|-----------------|
| **rhizoCrypt** | `dag` | Merkle DAG anchoring | `dag.anchor`, `dag.verify`, `dag.query` |
| **loamSpine** | `ledger` | Append-only ledger | `ledger.append`, `ledger.query`, `ledger.verify` |
| **sweetGrass** | `braid` | PROV-O braid verification | `braid.weave`, `braid.verify`, `braid.query` |

## Provenance Pipeline

### Phase 1: Workload Execution

ToadStool dispatches a workload via `sandbox.execute`. The workload
produces output files.

### Phase 2: Hash Anchoring

BLAKE3 hash of each output file → `dag.anchor` on rhizoCrypt.
Returns a DAG node ID (hex string).

### Phase 3: Ledger Append

DAG node IDs + metadata → `ledger.append` on loamSpine.
Returns a ledger entry ID.

### Phase 4: Braid Verification

Ledger entries → `braid.weave` on sweetGrass.
Returns a Merkle root for the braid.

### Phase 5: Storage (Optional)

Provenance artifacts → `storage.store` on NestGate.
Key format: `provenance:{workload_id}:{artifact_type}`

## Hash Encoding

All BLAKE3 hashes at the JSON-RPC boundary use **lowercase hex strings**
(64 characters for 32-byte hashes).

```json
{
  "jsonrpc": "2.0",
  "method": "dag.anchor",
  "params": {
    "hash": "a1b2c3d4e5f6...64chars...",
    "metadata": {
      "workload": "abg_phylogeny",
      "timestamp": "2026-05-06T16:45:00Z"
    }
  },
  "id": 1
}
```

**Conversion rule**: Primals may use `[u8; 32]` internally. At the
JSON-RPC boundary, convert to/from hex. loamSpine historically returned
byte arrays — the operational handoff confirmed hex normalization is the
correct integration pattern.

## NestGate Key Namespace

Storage keys follow a three-part convention:

```
{domain}:{id}:{qualifier}
```

| Domain | Example Key | Content |
|--------|-------------|---------|
| `provenance` | `provenance:workload_123:dag` | DAG anchor result |
| `provenance` | `provenance:workload_123:ledger` | Ledger entry |
| `provenance` | `provenance:workload_123:braid` | Braid merkle root |
| `workload` | `workload:abg_phylo:output` | Workload output blob |
| `config` | `config:gate:<active-gate>` | Gate configuration snapshot |

**NestGate transport note**: NestGate uses newline-delimited TCP JSON-RPC,
not HTTP. Use `nc` or raw TCP — not `curl`.

## PROV-O Braid Model

sweetGrass braids follow the W3C PROV-O ontology adapted for NUCLEUS:

| PROV-O Concept | NUCLEUS Mapping |
|----------------|-----------------|
| **Entity** | Workload output, DAG node, ledger entry |
| **Activity** | Workload execution, hash computation, braid weave |
| **Agent** | Primal that performed the activity |
| **wasGeneratedBy** | Output → workload execution |
| **wasDerivedFrom** | Braid → ledger entries → DAG nodes |
| **wasAttributedTo** | Result → responsible primal |

## Transport Details

| Primal | Port | Framing | BTSP | Notes |
|--------|------|---------|------|-------|
| rhizoCrypt | 9601 | Newline-delimited TCP | Required | Use `nc` for health |
| loamSpine | 9700 | HTTP JSON-RPC | Required | Use `curl` for health |
| sweetGrass | 9850 | BTSP-encrypted TCP | Strict | Must complete BTSP handshake before any call |

**sweetGrass strict BTSP**: Unlike other primals where BTSP is negotiated
on first meaningful call, sweetGrass requires BTSP handshake completion
before accepting any method call on TCP. Use UDS to bypass this requirement
in development.

## Workload-Level Provenance

Currently, provenance is applied via the `provenance_pipeline.sh` wrapper
script. The target evolution is a `[provenance]` section in workload TOMLs:

```toml
# Future — not yet implemented in ToadStool
[provenance]
enabled = true
dag_anchor = true
ledger_append = true
braid_weave = true
storage_key_prefix = "provenance"
```

Until ToadStool supports native `[provenance]`, the pipeline wrapper
remains the authoritative implementation.

## Validation

### Trio Health

All three primals must pass `health.liveness` for provenance to be
operational. The pipeline script checks all three and reports degradation
if any fail.

### Chain Integrity

A valid provenance chain requires:
1. BLAKE3 hash matches file content
2. DAG node references valid hash
3. Ledger entry references valid DAG node
4. Braid merkle root covers all ledger entries for the batch

### Missing Trio

If the trio is not in the composition (e.g., `tower` or `node` without
`nest`), provenance is unavailable. Workloads execute without provenance
tracking. This is the expected degradation — not a failure.

## References

- wateringHole: `SWEETGRASS_SPRING_BRAID_PATTERNS.md` (PROV-O braids)
- wateringHole: `PROVENANCE_TRIO_OPERATIONAL_HANDOFF_MAY2026.md` (operational)
- wateringHole: `NESTGATE_STORAGE_PATTERNS_HANDOFF_MAY2026.md` (key namespace)
- gen4: `architecture/ANCHORING_PIPELINE.md` (DAG→sign→dehydrate→anchor)
- gen4: `foundations/ABG_WHOLE_CELL_REBUILD.md` (provenance-heavy deployment)
