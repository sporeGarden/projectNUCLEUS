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
./scenarios/btsp_tls_parity.sh --baseline baselines/cloudflare_tunnel_7day.toml

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
│   └── full_stack_load.sh
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
