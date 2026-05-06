# Notebook Elevation — Validation to Visualization

How springs prepare their validation suites for interactive notebook execution
and visual exploration by ABG members on the JupyterHub compute resource.

---

## Principle: Validate Natively, Visualize Interactively

The math runs in Rust, natively, inside the primals. Notebooks don't recompute
— they dispatch, parse, and render. This separation matters:

- **Primals** own the computation (correctness, performance, provenance)
- **Notebooks** own the presentation (visualization, exploration, comparison)
- **ToadStool** bridges the two (workload dispatch, structured output)

An ABG member runs a notebook cell. That cell invokes `toadstool execute`
against a workload TOML. The output comes back as structured text with `[OK]`
and `[FAIL]` markers, numeric values, expected/tolerance pairs, and section
headers. The notebook parses this output and renders charts, tables, and
provenance links.

Over time, the math itself may be exposed as primal API calls (JSON-RPC)
that notebooks invoke directly. This is the long-term evolution path.

---

## Elevation Tiers

### Tier 0: CLI Validation (current)

The spring provides a compiled Rust binary (`validate_*`) that runs
independently and prints structured output to stdout.

```
$ validate_16s_pipeline
═══════════════════════════════════════════════════════════
  wetSpring 16S Pipeline Validation
═══════════════════════════════════════════════════════════

DADA2 Denoising
  [OK]  DADA2 distinct → 2 ASVs: 2 (expected 2)
  [OK]  DADA2 input uniques: 2 (expected 2)
...
  wetSpring 16S Pipeline Validation: 37/37 checks passed
  RESULT: PASS
```

**What the spring provides**: Binary + workload TOML.
**What projectNUCLEUS does**: Wraps in provenance pipeline, deploys on JupyterHub.

### Tier 1: Notebook Visualization (current target)

The spring's validation output is parsed and rendered interactively.
The notebook runs the binary via ToadStool, extracts `[OK]/[FAIL]` lines,
and creates matplotlib/pandas visualizations.

**What the spring provides**: Same binary + TOML + optional structured JSON output.
**What projectNUCLEUS does**: Provides the `wetspring-validation-viz.ipynb` notebook.

### Tier 2: JSON-RPC Validation APIs (evolution target)

The spring exposes validation as primal JSON-RPC methods. The notebook
calls these directly, gets structured JSON responses, and renders them.

```python
resp = rpc(TOADSTOOL_PORT, "compute.validate_16s", {
    "data_path": "ncbi:SRR7760408",
    "parameters": {"min_abundance": 2}
})
df = pd.DataFrame(resp["result"]["checks"])
```

**What the spring provides**: JSON-RPC methods on ToadStool or a custom primal.
**What projectNUCLEUS does**: Notebooks call APIs directly, richer interactivity.

### Tier 3: Live Compute Dashboard (long-term)

petalTongue renders a web dashboard that non-notebook users can access.
Results flow from primals → petalTongue in real time. Notebooks become
one of many clients.

**What the spring provides**: Primal APIs.
**What projectNUCLEUS does**: petalTongue + NestGate serve the dashboard.

---

## Structured Output Contract

For Tier 0/1 to work, spring validation binaries should emit structured output
following this contract:

### Section Headers

```
Section Name
  [OK]  ...
  [FAIL] ...
```

Or delimited:

```
── Section Name ──
  [OK]  ...
```

### Check Lines

Numeric checks with expected value and tolerance:

```
  [OK]  Check name: 1.386294 (expected 1.386294, tol 0.000000)
  [FAIL] Check name: 0.500000 (expected 1.000000, tol 0.100000)
```

Integer checks:

```
  [OK]  Check name: 37 (expected 37)
```

### Summary Line

```
  Spring Name Validation: 37/37 checks passed
  RESULT: PASS
```

### Optional: JSON Mode

For Tier 2 readiness, binaries can also emit structured JSON when invoked
with `--json`:

```json
{
  "validation": "16s_pipeline",
  "sections": [
    {
      "name": "DADA2 Denoising",
      "checks": [
        {"name": "distinct ASVs", "actual": 2, "expected": 2, "status": "OK"}
      ]
    }
  ],
  "summary": {"total": 37, "ok": 37, "fail": 0, "result": "PASS"}
}
```

---

## Workload TOML Contract

Each validation needs a workload TOML that ToadStool can dispatch:

```toml
[metadata]
name = "spring-validation-name"
description = "What this validates"
version = "0.1.0"

[execution]
type = "native"
command = "/path/to/validate_binary"
working_dir = "/path/to/spring"

[resources]
max_memory_bytes = 2147483648
max_cpu_percent = 80.0

[security]
isolation_level = "None"
```

The `command` path should be absolute and point to a built binary in the
spring's `target/release/` directory. projectNUCLEUS's `deploy.sh` and
the provenance pipeline will handle discovery and wrapping.

---

## ABG Tier Access

Notebooks respect the user's ABG tier:

| Tier | Can Execute | Can Visualize | Can Modify |
|------|-------------|---------------|------------|
| observer | No | Yes (cached results) | No |
| compute | Yes (via ToadStool) | Yes | No |
| admin | Yes | Yes | Yes |

Observer-tier users see pre-computed results from the latest provenance
pipeline run. Compute-tier users can re-run validations and see live results.

---

## What Springs Need to Prepare

For any spring wanting to elevate validations into notebooks:

1. **Structured output** — follow the `[OK]/[FAIL]` contract above
2. **Workload TOML** — one per validation binary, in `workloads/<spring>/`
3. **Data references** — NCBI accessions, BLAKE3 hashes, file paths
4. **Section grouping** — organize checks into logical domains
5. **Optional: JSON mode** — `--json` flag for structured output

projectNUCLEUS handles the rest: notebook generation, provenance wrapping,
JupyterHub deployment, tier-scoped access, and visualization.

---

## Current Notebooks

| Notebook | Purpose | Tier Required |
|----------|---------|--------------|
| `abg-wetspring-validation.ipynb` | Run workloads, inspect provenance | compute |
| `wetspring-validation-viz.ipynb` | Full visualization dashboard | compute |

---

## Evolution Path

```
Now:      CLI binaries → ToadStool dispatch → notebook parse → matplotlib
Near:     + JSON output mode → richer notebook interactivity
Medium:   + JSON-RPC primal APIs → notebooks call primals directly
Long:     + petalTongue dashboards → web-native visualization
Eventual: + sweetGrass attribution → every visualization is provenance-tracked
```

Each step adds capability without removing previous tiers. A CLI binary
that works today will still work when JSON-RPC APIs exist — the notebook
just gets smarter about how it gets data.
