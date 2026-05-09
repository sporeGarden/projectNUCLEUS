# Live Science API — JSON-RPC Method Specifications

Target methods for Tier 2 (notebook-direct) and Tier 3 (petalTongue dashboard)
live science access. These methods do not exist yet — this spec is the contract
that primal teams evolve toward.

**Current state**: Tier 0/1 (CLI binaries + notebook parsing)
**Target**: Tier 2 (JSON-RPC from notebooks) → Tier 3 (petalTongue live dashboard)

---

## toadstool.validate

Dispatch a workload and return structured results as JSON instead of CLI text.

**Owner**: ToadStool team
**Transport**: HTTP JSON-RPC on port 9400

```json
{
  "jsonrpc": "2.0",
  "method": "toadstool.validate",
  "params": {
    "workload": "wetspring-16s-rust-validation",
    "format": "json"
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "workload": "wetspring-16s-rust-validation",
    "status": "PASS",
    "duration_ms": 450,
    "sections": [
      {
        "name": "DADA2 Denoising",
        "checks": [
          {"name": "distinct ASVs", "actual": 2, "expected": 2, "status": "OK"},
          {"name": "input uniques", "actual": 2, "expected": 2, "status": "OK"}
        ]
      }
    ],
    "summary": {"total": 37, "ok": 37, "fail": 0},
    "output_hash": "5cee126110521afb..."
  }
}
```

**Why this matters**: Eliminates CLI output parsing. Notebooks get typed data
directly. petalTongue can render live dashboards without subprocess execution.

---

## toadstool.list_workloads

List available workload TOMLs, optionally filtered by spring.

**Owner**: ToadStool team

```json
{
  "jsonrpc": "2.0",
  "method": "toadstool.list_workloads",
  "params": {"spring": "wetspring"},
  "id": 1
}
```

**Response**:

```json
{
  "result": [
    {
      "name": "wetspring-16s-rust-validation",
      "description": "Full 16S pipeline validation",
      "spring": "wetspring",
      "checks": 37
    }
  ]
}
```

---

## barracuda.compute

Submit a GPU compute request — invoke a barraCuda primitive by name.

**Owner**: barraCuda team
**Transport**: Newline-delimited JSON-RPC on port 9740

```json
{
  "jsonrpc": "2.0",
  "method": "barracuda.compute",
  "params": {
    "primitive": "shannon_entropy_f64",
    "input": [0.25, 0.25, 0.25, 0.25],
    "device": "gpu"
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "output": 1.3862943611198906,
    "device": "RTX 4070",
    "duration_us": 42,
    "precision": "f64"
  }
}
```

**Why this matters**: Notebooks can invoke GPU math directly without workload
TOML indirection. Enables interactive exploration of barraCuda's 79+ primitives.

---

## biomeos.spring_status

Return which springs have validation binaries available on this gate.

**Owner**: biomeOS team
**Transport**: Newline-delimited JSON-RPC on port 9800

```json
{
  "jsonrpc": "2.0",
  "method": "biomeos.spring_status",
  "params": {},
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "springs": [
      {
        "name": "wetspring",
        "binaries": 8,
        "workloads": 11,
        "last_validated": "2026-05-06T18:30:00Z",
        "checks_passing": 235
      },
      {
        "name": "hotspring",
        "binaries": 4,
        "workloads": 1,
        "last_validated": null,
        "checks_passing": 0
      }
    ],
    "gate": "<active-gate>",
    "composition": "full",
    "primals_healthy": 13
  }
}
```

**Why this matters**: The notebook template auto-discovers which springs are
ready for validation instead of hardcoding paths. petalTongue can render a
"which springs are validated" dashboard.

---

## nestgate.artifact_query

Query provenance for a specific content-addressed artifact.

**Owner**: NestGate team
**Transport**: Newline-delimited JSON-RPC on port 9500

```json
{
  "jsonrpc": "2.0",
  "method": "nestgate.artifact_query",
  "params": {
    "hash": "b106aa1d1bb45430d00d605626e10488119f9e4f9f315a738939049a6da9ceec"
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "hash": "b106aa1d...",
    "type": "merkle_root",
    "created": "2026-05-06T18:30:00Z",
    "dag_session": "019dfe5d-c17f-7a93-889e-01bf813ee7f8",
    "ledger_index": 32,
    "braid_urn": "urn:braid:b106aa1d...",
    "witness": {
      "algorithm": "ed25519",
      "agent": "did:key:z6MkL98JJs...",
      "verified": true
    }
  }
}
```

**Why this matters**: Notebooks can verify provenance chains inline. External
reviewers can trace any result to its cryptographic witness.

---

## rhizocrypt.dag_summary

Get a summary of a DAG session — event count, Merkle root, artifact list.

**Owner**: rhizoCrypt team
**Transport**: Newline-delimited JSON-RPC on port 9602

```json
{
  "jsonrpc": "2.0",
  "method": "rhizocrypt.dag_summary",
  "params": {
    "session_id": "019dfe5d-c17f-7a93-889e-01bf813ee7f8"
  },
  "id": 1
}
```

**Response**:

```json
{
  "result": {
    "session_id": "019dfe5d-...",
    "event_count": 26,
    "merkle_root": "b106aa1d...",
    "artifacts": [
      {"key": "ncbi:SRR7760408:R1", "hash": "6250f200...", "size_bytes": 2200000000}
    ]
  }
}
```

---

## Evolution Path

```
Tier 0 (now):     CLI binary → stdout [OK]/[FAIL]
Tier 1 (now):     + notebook parses CLI output → matplotlib
Tier 2 (target):  + JSON-RPC methods above → notebooks call primals directly
Tier 3 (future):  + petalTongue renders live dashboard from primal APIs
Standalone:       + NestGate serves content → sporePrint self-hosted on NUCLEUS
```

Each tier adds capability without removing previous tiers. A CLI binary that
works at Tier 0 continues to work when Tier 2 JSON-RPC APIs exist.

---

## Implementation Priority

| Method | Owner | Priority | Unlocks |
|--------|-------|----------|---------|
| `toadstool.validate` | ToadStool | P0 | Tier 2 notebooks, petalTongue dashboards |
| `toadstool.list_workloads` | ToadStool | P0 | Auto-discovery in notebooks |
| `biomeos.spring_status` | biomeOS | P1 | Spring status dashboard |
| `nestgate.artifact_query` | NestGate | P1 | Inline provenance verification |
| `barracuda.compute` | barraCuda | P2 | Interactive GPU exploration |
| `rhizocrypt.dag_summary` | rhizoCrypt | P2 | Full provenance chain inspection |

P0 methods enable Tier 2. P1 methods complete the notebook experience.
P2 methods enable interactive exploration and Tier 3 dashboards.
