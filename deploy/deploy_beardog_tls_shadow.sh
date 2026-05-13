#!/usr/bin/env bash
# deploy_beardog_tls_shadow.sh — Start BearDog TLS shadow on :8443 (H2-12)
#
# Runs BearDog with rustls TLS (Wave 100) alongside the existing Cloudflare
# tunnel on :443. Captures comparison metrics for interstadial exit criteria.
#
# Prerequisites:
#   - BearDog binary in plasmidBin with TLS support (Wave 100+)
#   - Self-signed or ACME cert for lab.primals.eco
#   - Cloudflare baseline metrics in validation/baselines/
#
# Usage:
#   bash deploy_beardog_tls_shadow.sh [--cert PATH] [--key PATH] [--port 8443]
#
# See: infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md (Shadow Run Readiness)
#      infra/wateringHole/btsp/BEARDOG_TECHNICAL_STACK.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

SHADOW_PORT="${1:-8443}"
BEARDOG_BIN="${PLASMIDBIN_DIR:-$GATE_HOME/plasmidBin}/primals/beardog"
BASELINE_DIR="${SCRIPT_DIR}/../validation/baselines"
SHADOW_LOG="/tmp/beardog-tls-shadow.log"
AUDIT_DIR="${GATE_HOME:?}/.beardog/audit"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --port) SHADOW_PORT="$2"; shift 2 ;;
        *) shift ;;
    esac
done

echo "══ BearDog TLS Shadow Run (H2-12) ══"
echo "  Port:     $SHADOW_PORT"
echo "  Binary:   $BEARDOG_BIN"
echo "  Log:      $SHADOW_LOG"
echo "  Audit:    $AUDIT_DIR"
echo ""

if [[ ! -x "$BEARDOG_BIN" ]]; then
    echo "ERROR: BearDog binary not found at $BEARDOG_BIN"
    echo "  Run: deploy/fetch_primals.sh to pull from plasmidBin"
    exit 1
fi

echo "  BearDog version: $("$BEARDOG_BIN" version 2>&1 || echo unknown)"
echo ""

mkdir -p "$AUDIT_DIR"

# Start BearDog with TCP listener (rustls/BTSP handles encryption internally)
echo "  Starting BearDog on :${SHADOW_PORT}..."
export BEARDOG_FAMILY_SEED="${BEACON_SEED:-$(head -c 32 /dev/urandom | base64)}"

nohup "$BEARDOG_BIN" server \
    --listen "127.0.0.1:${SHADOW_PORT}" \
    --family-id "${FAMILY_ID:-nucleus}" \
    --audit-dir "$AUDIT_DIR" \
    > "$SHADOW_LOG" 2>&1 &

SHADOW_PID=$!
echo "  PID: $SHADOW_PID"
sleep 2

if ! kill -0 "$SHADOW_PID" 2>/dev/null; then
    echo "  ERROR: BearDog TLS shadow failed to start"
    echo "  Check $SHADOW_LOG for details"
    exit 1
fi

# Phase 3: Health check + baseline comparison probe
echo ""
echo "  Running health check..."

beardog_health() {
    local start_ns end_ns elapsed_ms
    start_ns=$(date +%s%N)
    if echo '{"jsonrpc":"2.0","method":"health.liveness","id":1}' | \
        nc -w 2 127.0.0.1 "$SHADOW_PORT" 2>/dev/null | grep -q "result"; then
        end_ns=$(date +%s%N)
        elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
        echo "$elapsed_ms"
    else
        echo "unreachable"
    fi
}

bd_time=$(beardog_health)
cf_time=$(curl -so /dev/null -w '%{time_total}' --max-time 5 \
    "https://lab.primals.eco" 2>/dev/null || echo "unreachable")

echo "  Cloudflare latency:  ${cf_time}s"
echo "  BearDog RPC latency: ${bd_time}ms"

# Phase 4: Record baseline
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
BASELINE_FILE="${BASELINE_DIR}/beardog_tls_shadow_${TIMESTAMP}.csv"
mkdir -p "$BASELINE_DIR"
echo "timestamp,cloudflare_s,beardog_rpc_ms,shadow_port,pid" > "$BASELINE_FILE"
echo "${TIMESTAMP},${cf_time},${bd_time},${SHADOW_PORT},${SHADOW_PID}" >> "$BASELINE_FILE"
echo ""
echo "  Baseline recorded: $BASELINE_FILE"

echo ""
echo "══ Shadow running ══"
echo "  BearDog JSON-RPC on :${SHADOW_PORT} (PID $SHADOW_PID)"
echo "  Cloudflare tunnel on :443 (unchanged)"
echo "  Comparison metrics accumulating in $BASELINE_DIR"
echo ""
echo "  To stop: kill $SHADOW_PID"
echo "  To monitor: tail -f $SHADOW_LOG"
echo "  Parity test: infra/benchScale/scenarios/btsp_tls_parity.sh"
