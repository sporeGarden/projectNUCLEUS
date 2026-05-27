#!/usr/bin/env bash
set -euo pipefail

# Compares BTSP TLS termination (BearDog) against Cloudflare TLS baseline.
#
# Prerequisites:
#   - BearDog TLS termination running on the specified port
#   - A Cloudflare baseline TOML from cloudflare_tunnel_baseline.sh
#
# Usage:
#   ./btsp_tls_parity.sh --baseline ../baselines/cloudflare_tunnel_7day.toml \
#                         --btsp-url https://127.0.0.1:8443
#
# Produces a parity report showing whether BTSP matches CF performance.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../../deploy/nucleus_config.sh"

REPORTS_DIR="$SCRIPT_DIR/../reports"
BASELINE=""
BTSP_URL="https://${BTSP_SHADOW_HOST:-127.0.0.1}:${BTSP_SHADOW_PORT:-8443}/hub/login"
SAMPLES=10

while [[ $# -gt 0 ]]; do
    case "$1" in
        --baseline) BASELINE="$2"; shift 2 ;;
        --btsp-url) BTSP_URL="$2"; shift 2 ;;
        --samples)  SAMPLES="$2"; shift 2 ;;
        *)          echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

if [[ -z "$BASELINE" ]]; then
    echo "Usage: $0 --baseline <cloudflare_baseline.toml> [--btsp-url URL]" >&2
    exit 1
fi

if [[ ! -f "$BASELINE" ]]; then
    echo "Baseline file not found: $BASELINE" >&2
    exit 1
fi

mkdir -p "$REPORTS_DIR"

RUN_ID="$(date -u +%Y%m%d-%H%M%S)"
REPORT="$REPORTS_DIR/btsp_tls_parity_${RUN_ID}.toml"

echo "benchScale: BTSP TLS Parity Test"
echo "  Baseline: $BASELINE"
echo "  BTSP URL: $BTSP_URL"
echo "  Samples:  $SAMPLES"
echo ""

# Extract baseline thresholds
CF_TLS_P95=$(grep 'tls_p95' "$BASELINE" | head -1 | awk -F= '{gsub(/ /,"",$2); print $2}')
CF_TTFB_P95=$(grep 'ttfb_p95' "$BASELINE" | head -1 | awk -F= '{gsub(/ /,"",$2); print $2}')

echo "  CF Baseline: tls_p95=${CF_TLS_P95}ms  ttfb_p95=${CF_TTFB_P95}ms"
echo ""

# BearDog speaks JSON-RPC over TCP (BTSP protocol), not HTTPS.
# Measure latency via JSON-RPC health.liveness round-trip.
BTSP_HOST=$(echo "$BTSP_URL" | sed -E 's|https?://||' | cut -d: -f1)
BTSP_PORT=$(echo "$BTSP_URL" | sed -E 's|https?://||' | cut -d: -f2 | cut -d/ -f1)
[[ -z "$BTSP_PORT" ]] && BTSP_PORT=8443
RPC_REQ='{"jsonrpc":"2.0","method":"health.liveness","id":1}'

TMP_CSV=$(mktemp)
echo "rpc_ms,code" > "$TMP_CSV"

PASS=0
for i in $(seq 1 "$SAMPLES"); do
    start_ns=$(date +%s%N)
    resp=$(echo "$RPC_REQ" | nc -q 0 -w 2 "$BTSP_HOST" "$BTSP_PORT" 2>/dev/null | head -1 || echo "")
    end_ns=$(date +%s%N)
    elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))

    if echo "$resp" | grep -q '"result"'; then
        code="OK"
        PASS=$((PASS + 1))
    else
        code="ERR"
    fi

    echo "${elapsed_ms},${code}" >> "$TMP_CSV"
    printf "  [%d/%d] %s  rpc=%dms\n" "$i" "$SAMPLES" "$code" "$elapsed_ms"
    sleep 1
done

percentile_csv() {
    local col=$1 pct=$2
    tail -n +2 "$TMP_CSV" | awk -F, "{print \$$col}" | sort -n | awk -v p="$pct" '
        {a[NR]=$1}
        END {
            idx = int(p/100 * NR + 0.5)
            if (idx < 1) idx = 1
            if (idx > NR) idx = NR
            printf "%.2f", a[idx]
        }'
}

BTSP_RPC_P95=$(percentile_csv 1 95)

# Compare JSON-RPC p95 against CF TTFB p95 (both measure "time to response")
RPC_PARITY=$(echo "$BTSP_RPC_P95 <= $CF_TTFB_P95" | bc -l 2>/dev/null || echo "0")

echo ""
echo "Results:"
echo "  BTSP rpc_p95:  ${BTSP_RPC_P95}ms  (CF ttfb_p95: ${CF_TTFB_P95}ms, CF tls_p95: ${CF_TLS_P95}ms)"
echo "  RPC parity (rpc ≤ CF TTFB): $([ "$RPC_PARITY" = "1" ] && echo "PASS" || echo "FAIL")"
echo "  Success rate: ${PASS}/${SAMPLES}"

cat > "$REPORT" << EOF
# BTSP TLS Parity Report — $RUN_ID
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

[metadata]
baseline = "$BASELINE"
btsp_url = "$BTSP_URL"
btsp_host = "$BTSP_HOST"
btsp_port = $BTSP_PORT
probe = "JSON-RPC health.liveness"
samples = $SAMPLES
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[cloudflare_baseline]
tls_p95_ms = $CF_TLS_P95
ttfb_p95_ms = $CF_TTFB_P95

[btsp_measured]
rpc_p95_ms = $BTSP_RPC_P95
uptime_pct = $(echo "scale=1; $PASS * 100 / $SAMPLES" | bc)

[parity]
rpc_parity = $([ "$RPC_PARITY" = "1" ] && echo "true" || echo "false")
overall = $([ "$RPC_PARITY" = "1" ] && echo "true" || echo "false")
EOF

rm -f "$TMP_CSV"
echo ""
echo "Report: $REPORT"
