#!/usr/bin/env bash
set -euo pipefail

# Captures Cloudflare tunnel baseline metrics for lab.primals.eco.
# Runs a configurable number of samples, measures latency/throughput/uptime,
# and writes results to the baselines/ directory.
#
# Usage:
#   ./cloudflare_tunnel_baseline.sh [--samples N] [--url URL] [--output DIR]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../deploy/nucleus_config.sh"

BASELINES_DIR="$SCRIPT_DIR/../baselines"
SAMPLES=10
TARGET_URL="${LAB_URL}/hub/login"
DELAY_BETWEEN=2

while [[ $# -gt 0 ]]; do
    case "$1" in
        --samples)  SAMPLES="$2"; shift 2 ;;
        --url)      TARGET_URL="$2"; shift 2 ;;
        --output)   BASELINES_DIR="$2"; shift 2 ;;
        *)          echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$BASELINES_DIR"

RUN_ID="$(date -u +%Y%m%d-%H%M%S)"
CSV="$BASELINES_DIR/tunnel_baseline_${RUN_ID}.csv"

echo "timestamp_utc,dns_ms,tcp_ms,tls_ms,ttfb_ms,total_ms,http_code,speed_bps,size_bytes" > "$CSV"

CURL_FMT='%{time_namelookup},%{time_connect},%{time_appconnect},%{time_starttransfer},%{time_total},%{http_code},%{speed_download},%{size_download}'

echo "benchScale: Cloudflare tunnel baseline"
echo "  Target:  $TARGET_URL"
echo "  Samples: $SAMPLES"
echo "  Output:  $CSV"
echo ""

PASS=0
for i in $(seq 1 "$SAMPLES"); do
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    raw=$(curl -sS -o /dev/null -w "$CURL_FMT" --max-time 30 "$TARGET_URL" 2>/dev/null) || raw="0,0,0,0,0,0,0,0"

    IFS=',' read -r dns tcp tls ttfb total code speed size <<< "$raw"

    dns_ms=$(echo "$dns * 1000" | bc)
    tcp_ms=$(echo "$tcp * 1000" | bc)
    tls_ms=$(echo "$tls * 1000" | bc)
    ttfb_ms=$(echo "$ttfb * 1000" | bc)
    total_ms=$(echo "$total * 1000" | bc)

    echo "${ts},${dns_ms},${tcp_ms},${tls_ms},${ttfb_ms},${total_ms},${code},${speed},${size}" >> "$CSV"

    if [[ "$code" == "200" ]]; then
        PASS=$((PASS + 1))
    fi

    printf "  [%d/%d] HTTP %s  ttfb=%sms  tls=%sms  total=%sms\n" "$i" "$SAMPLES" "$code" "$ttfb_ms" "$tls_ms" "$total_ms"
    sleep "$DELAY_BETWEEN"
done

echo ""
echo "Uptime: ${PASS}/${SAMPLES} requests succeeded"
echo "Results: $CSV"

# Generate summary TOML
SUMMARY="$BASELINES_DIR/tunnel_baseline_${RUN_ID}.toml"

percentile() {
    local col=$1 pct=$2
    tail -n +2 "$CSV" | awk -F, "{print \$$col}" | sort -n | awk -v p="$pct" '
        {a[NR]=$1}
        END {
            idx = int(p/100 * NR + 0.5)
            if (idx < 1) idx = 1
            if (idx > NR) idx = NR
            printf "%.2f", a[idx]
        }'
}

cat > "$SUMMARY" << EOF
# Cloudflare Tunnel Baseline — $RUN_ID
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

[metadata]
target = "$TARGET_URL"
samples = $SAMPLES
uptime_pct = $(echo "scale=1; $PASS * 100 / $SAMPLES" | bc)
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[latency_ms]
ttfb_p50 = $(percentile 5 50)
ttfb_p95 = $(percentile 5 95)
ttfb_p99 = $(percentile 5 99)
tls_p50  = $(percentile 4 50)
tls_p95  = $(percentile 4 95)
tls_p99  = $(percentile 4 99)
total_p50 = $(percentile 6 50)
total_p95 = $(percentile 6 95)
total_p99 = $(percentile 6 99)

[throughput]
avg_bytes_per_sec = $(tail -n +2 "$CSV" | awk -F, '{sum+=$8; n++} END {if(n>0) printf "%.0f", sum/n; else print 0}')
EOF

echo "Summary: $SUMMARY"
