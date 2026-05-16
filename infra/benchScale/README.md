# benchScale

Load generation, pen testing, and parity validation framework for NUCLEUS.

benchScale is the tool that validates every external dependency replacement on
the path to full sovereignty. It captures baselines for current external
services, runs parity comparisons against primal replacements, and produces
structured reports.

## Usage

```bash
# Capture Cloudflare tunnel baseline (runs 5 samples)
./scenarios/cloudflare_tunnel_baseline.sh

# Compare BTSP TLS against Cloudflare baseline
./scenarios/btsp_tls_parity.sh --baseline baselines/tunnel_baseline_YYYYMMDD-HHMMSS.toml

# Run full-stack synthetic load at 2x peak
./scenarios/full_stack_load.sh --multiplier 2

# Run three-layer security scan
./pentest/three_layer_scan.sh --tunnel-url https://lab.primals.eco

# Fuzz primal JSON-RPC APIs
./pentest/fuzz_jsonrpc.py --target 127.0.0.1 --ports 9100,9400,9500

# External attack surface probe
./pentest/tunnel_probe.sh --url https://lab.primals.eco
```

## Structure

```
benchScale/
├── scenarios/          # Load and parity validation scenarios
│   ├── cloudflare_tunnel_baseline.sh
│   ├── btsp_tls_parity.sh
│   ├── nestgate_content_parity.sh
│   ├── songbird_nat_parity.sh
│   ├── dot_sovereign_parity.sh
│   ├── full_stack_load.sh
│   ├── shadow_run_orchestrator.sh
│   └── artifact_validation.sh    # 7-artifact validation suite
├── pentest/            # Security testing tools
│   ├── three_layer_scan.sh
│   ├── fuzz_jsonrpc.py
│   └── tunnel_probe.sh
├── baselines/          # Captured baseline CSVs and TOMLs
└── reports/            # Generated comparison reports
```

## Validation Flow

1. **Baseline** — capture metrics for the current external service (e.g. Cloudflare tunnel)
2. **Shadow Run** — run the primal replacement alongside the external service for 7 days
3. **Parity Check** — compare replacement metrics against baseline thresholds
4. **Cutover** — if parity met, switch traffic; if not, iterate

Every scenario produces a structured TOML report in `reports/` that can be
consumed by skunkBat for security metric tracking.

## Artifact Validation

`scenarios/artifact_validation.sh` exercises the 7 long-term sovereign artifacts
against a running NUCLEUS deployment:

```bash
# Run all 7 artifact sections
./scenarios/artifact_validation.sh

# Run a single section (1-7)
./scenarios/artifact_validation.sh --section 2   # Novel Ferment Transcript only
```

Sections:
1. Provenance Trio Pipeline (nest_store signal graph)
2. Novel Ferment Transcript (time-accumulated DAG → cert)
3. Loam Certificate (ownership proof minting)
4. Tier 2 Key Ceremony (protocol validation with synthetic entropy)
5. Steam Data Federation (NestGate cross-gate content replication)
6. sunCloud Metabolic Routing (multi-contributor attribution braids)
7. BearDog Genetic Authority (key derivation + trust ceiling)

See `specs/VALIDATION_PLAYBOOK.md` for the full artifact mapping including
use cases, science cases, and integration with Rust benchScale topologies
at `sort-after/benchScale/topologies/nucleus/`.
