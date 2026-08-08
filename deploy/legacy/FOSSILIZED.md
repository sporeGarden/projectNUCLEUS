# deploy/legacy/ — FOSSILIZED

**Status**: FOSSILIZED — bash deploy pipeline superseded by nucleus-deploy
**Filed**: 2026-08-08 Wave 157a role refinement
**Superseded by**: `nucleus-deploy` (Rust, 9 subcommands, 49 tests)

## Context

These 16 bash scripts were the original NUCLEUS deployment pipeline (pre-Wave 64).
All functionality has been absorbed into `nucleus-deploy`:

| Legacy Script | Replacement |
|---------------|-------------|
| `deploy.sh` | `nucleus-deploy deploy` |
| `security_validation.sh` | `nucleus-deploy security` |
| `provenance_pipeline.sh` | `nucleus-deploy provenance` |
| `membrane_telemetry.sh` | `nucleus-deploy telemetry` |
| `membrane_summary.sh` | `nucleus-deploy summary` |
| `deploy_graph.sh` | `nucleus-deploy deploy --graph-deploy` |
| `deploy_primal_start.sh` | `nucleus-deploy deploy` (subprocess) |
| `deploy_health_check.sh` | `nucleus-deploy verify` |
| `deploy_knot_dns.sh` | `nucleus-deploy dns` |
| `gate_provision_cloudflared.sh` | Fossilized (tunnelKeeper + cellMembrane) |
| `deploy_songbird_relay.sh` | cellMembrane membrane deploy |
| `cloudflared_config-*.yml` | tunnelKeeper config management |
| `cloudflare_access_setup.sh` | tunnelKeeper CF API |
| `membrane_provenance.sh` | `nucleus-deploy provenance` |

## Action

Move to `fossilRecord/projectNUCLEUS-legacy-deploy/` for archaeological
reference. No code here is active or maintained.
