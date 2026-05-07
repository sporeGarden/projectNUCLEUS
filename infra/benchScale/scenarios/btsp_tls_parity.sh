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
source "$SCRIPT_DIR/../../deploy/nucleus_config.sh"

REPORTS_DIR="$SCRIPT_DIR/../reports"
BASELINE=""
BTSP_URL="https://127.0.0.1:8443/hub/login"
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

CURL_FMT='%{time_namelookup},%{time_connect},%{time_appconnect},%{time_starttransfer},%{time_total},%{http_code}'
TMP_CSV=$(mktemp)
echo "tls_ms,ttfb_ms,total_ms,code" > "$TMP_CSV"

PASS=0
for i in $(seq 1 "$SAMPLES"); do
    raw=$(curl -sS -o /dev/null -w "$CURL_FMT" --max-time 30 -k "$BTSP_URL" 2>/dev/null) || raw="0,0,0,0,0,0"
    IFS=',' read -r dns tcp tls ttfb total code <<< "$raw"

    tls_ms=$(echo "$tls * 1000" | bc)
    ttfb_ms=$(echo "$ttfb * 1000" | bc)
    total_ms=$(echo "$total * 1000" | bc)

    echo "${tls_ms},${ttfb_ms},${total_ms},${code}" >> "$TMP_CSV"

    if [[ "$code" == "200" ]]; then
        PASS=$((PASS + 1))
    fi

    printf "  [%d/%d] HTTP %s  tls=%sms  ttfb=%sms\n" "$i" "$SAMPLES" "$code" "$tls_ms" "$ttfb_ms"
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

BTSP_TLS_P95=$(percentile_csv 1 95)
BTSP_TTFB_P95=$(percentile_csv 2 95)

TLS_PARITY=$(echo "$BTSP_TLS_P95 <= $CF_TLS_P95" | bc -l 2>/dev/null || echo "0")
TTFB_PARITY=$(echo "$BTSP_TTFB_P95 <= $CF_TTFB_P95" | bc -l 2>/dev/null || echo "0")

echo ""
echo "Results:"
echo "  BTSP tls_p95:  ${BTSP_TLS_P95}ms  (CF: ${CF_TLS_P95}ms) — $([ "$TLS_PARITY" = "1" ] && echo "PASS" || echo "FAIL")"
echo "  BTSP ttfb_p95: ${BTSP_TTFB_P95}ms (CF: ${CF_TTFB_P95}ms) — $([ "$TTFB_PARITY" = "1" ] && echo "PASS" || echo "FAIL")"

cat > "$REPORT" << EOF
# BTSP TLS Parity Report — $RUN_ID
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

[metadata]
baseline = "$BASELINE"
btsp_url = "$BTSP_URL"
samples = $SAMPLES
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[cloudflare_baseline]
tls_p95_ms = $CF_TLS_P95
ttfb_p95_ms = $CF_TTFB_P95

[btsp_measured]
tls_p95_ms = $BTSP_TLS_P95
ttfb_p95_ms = $BTSP_TTFB_P95
uptime_pct = $(echo "scale=1; $PASS * 100 / $SAMPLES" | bc)

[parity]
tls_parity = $([ "$TLS_PARITY" = "1" ] && echo "true" || echo "false")
ttfb_parity = $([ "$TTFB_PARITY" = "1" ] && echo "true" || echo "false")
overall = $([ "$TLS_PARITY" = "1" ] && [ "$TTFB_PARITY" = "1" ] && echo "true" || echo "false")
EOF

rm -f "$TMP_CSV"
echo ""
echo "Report: $REPORT"
