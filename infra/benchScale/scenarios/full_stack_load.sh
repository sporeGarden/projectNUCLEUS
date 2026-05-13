#!/usr/bin/env bash
set -euo pipefail

# Synthetic load test at configurable multiples of peak traffic.
#
# Generates concurrent requests against the full stack:
#   - JupyterHub (via tunnel or direct)
#   - Primal APIs (health endpoints)
#   - Content endpoints (when NestGate/petalTongue active)
#
# Uses curl in parallel (background subshells) for load generation.
# For heavier loads, consider wrapping with `hey` or `wrk` if available.
#
# Usage:
#   ./full_stack_load.sh [--multiplier 2] [--duration 60] [--target hub|primals|all]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../../deploy/nucleus_config.sh"

REPORTS_DIR="$SCRIPT_DIR/../reports"
MULTIPLIER=2
DURATION=60
TARGET="all"
HUB_URL="http://${NUCLEUS_BIND_ADDRESS}:${JUPYTERHUB_PORT}/hub/login"

# Estimated peak: 5 req/s across hub + primals
PEAK_RPS=5

while [[ $# -gt 0 ]]; do
    case "$1" in
        --multiplier)  MULTIPLIER="$2"; shift 2 ;;
        --duration)    DURATION="$2"; shift 2 ;;
        --target)      TARGET="$2"; shift 2 ;;
        --hub-url)     HUB_URL="$2"; shift 2 ;;
        *)             echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$REPORTS_DIR"
RUN_ID="$(date -u +%Y%m%d-%H%M%S)"
REPORT="$REPORTS_DIR/full_stack_load_${RUN_ID}.toml"
LOG="$REPORTS_DIR/full_stack_load_${RUN_ID}.log"

TARGET_RPS=$((PEAK_RPS * MULTIPLIER))
SLEEP_INTERVAL=$(echo "scale=3; 1 / $TARGET_RPS" | bc)

echo "benchScale: Full Stack Load Test"
echo "  Multiplier: ${MULTIPLIER}x peak"
echo "  Target RPS: $TARGET_RPS"
echo "  Duration:   ${DURATION}s"
echo "  Targets:    $TARGET"
echo ""

PRIMAL_ENDPOINTS=(
    "${NUCLEUS_BIND_ADDRESS}:${BEARDOG_PORT}"
    "${NUCLEUS_BIND_ADDRESS}:${TOADSTOOL_PORT}"
    "${NUCLEUS_BIND_ADDRESS}:${NESTGATE_PORT}"
    "${NUCLEUS_BIND_ADDRESS}:${LOAMSPINE_PORT}"
    "${NUCLEUS_BIND_ADDRESS}:${SKUNKBAT_PORT}"
)

TOTAL_REQS=0
TOTAL_OK=0
TOTAL_FAIL=0
LATENCY_SUM=0

fire_hub() {
    local result
    result=$(curl -sS -o /dev/null -w "%{time_total},%{http_code}" --max-time 10 "$HUB_URL" 2>/dev/null) || result="10,0"
    echo "hub,$result"
}

fire_primal() {
    local ep="$1"
    local result
    result=$(printf '{"jsonrpc":"2.0","method":"health.liveness","id":1}\n' | \
        nc -w 3 "${ep%%:*}" "${ep#*:}" 2>/dev/null | head -1)
    if echo "$result" | grep -q '"result"' 2>/dev/null; then
        echo "primal:${ep},0.001,200"
    else
        echo "primal:${ep},0,0"
    fi
}

START_TIME=$(date +%s)
END_TIME=$((START_TIME + DURATION))

echo "Load test running for ${DURATION}s..."

while [[ $(date +%s) -lt $END_TIME ]]; do
    # Fire requests in parallel
    if [[ "$TARGET" == "all" || "$TARGET" == "hub" ]]; then
        fire_hub >> "$LOG" &
    fi

    if [[ "$TARGET" == "all" || "$TARGET" == "primals" ]]; then
        ep="${PRIMAL_ENDPOINTS[$((RANDOM % ${#PRIMAL_ENDPOINTS[@]}))]}"
        fire_primal "$ep" >> "$LOG" &
    fi

    TOTAL_REQS=$((TOTAL_REQS + 1))

    # Throttle to target RPS (allow background jobs to accumulate)
    sleep "$SLEEP_INTERVAL" 2>/dev/null || sleep 1
done

wait

ELAPSED=$(($(date +%s) - START_TIME))
ACTUAL_RPS=$(echo "scale=1; $TOTAL_REQS / $ELAPSED" | bc 2>/dev/null || echo "?")

if [[ -f "$LOG" ]]; then
    TOTAL_OK=$(grep -c ",200" "$LOG" 2>/dev/null || echo "0")
    TOTAL_FAIL=$((TOTAL_REQS - TOTAL_OK))
fi

echo ""
echo "Load test complete."
echo "  Duration:     ${ELAPSED}s"
echo "  Total reqs:   $TOTAL_REQS"
echo "  Actual RPS:   $ACTUAL_RPS"
echo "  Successes:    $TOTAL_OK"
echo "  Failures:     $TOTAL_FAIL"

cat > "$REPORT" << EOF
# Full Stack Load Report — $RUN_ID
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

[metadata]
multiplier = $MULTIPLIER
peak_rps = $PEAK_RPS
target_rps = $TARGET_RPS
actual_rps = $ACTUAL_RPS
duration_s = $ELAPSED
target_scope = "$TARGET"
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[results]
total_requests = $TOTAL_REQS
successes = $TOTAL_OK
failures = $TOTAL_FAIL
success_rate_pct = $(echo "scale=1; $TOTAL_OK * 100 / $TOTAL_REQS" | bc 2>/dev/null || echo "0")

[thresholds]
min_success_rate_pct = 99.0
max_p95_latency_ms = 500
EOF

echo "Report: $REPORT"
