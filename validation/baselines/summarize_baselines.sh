#!/usr/bin/env bash
set -euo pipefail

# Summarizes daily CSVs into a consolidated baseline TOML.
# Run after 7 days of hourly capture to produce the reference baseline.

BASELINES_DIR="$(cd "$(dirname "$0")" && pwd)"
CSV_DIR="${BASELINES_DIR}/daily"
OUTPUT="${BASELINES_DIR}/cloudflare_tunnel_7day.toml"

if [ ! -d "$CSV_DIR" ]; then
    echo "No daily/ directory found. Run capture_tunnel_metrics.sh first."
    exit 1
fi

ALL_CSV=$(find "$CSV_DIR" -name 'tunnel_metrics_*.csv' -type f | sort)
COUNT=$(echo "$ALL_CSV" | wc -l)

if [ "$COUNT" -lt 1 ]; then
    echo "No CSV files found."
    exit 1
fi

echo "Summarizing $COUNT days of data..."

COMBINED=$(mktemp)
for f in $ALL_CSV; do
    tail -n +2 "$f" >> "$COMBINED"
done

TOTAL_SAMPLES=$(wc -l < "$COMBINED")

percentile() {
    local col=$1 pct=$2
    awk -F, "{print \$$col}" "$COMBINED" | sort -n | awk -v p="$pct" '
        {a[NR]=$1}
        END {
            idx = int(p/100 * NR + 0.5)
            if (idx < 1) idx = 1
            if (idx > NR) idx = NR
            printf "%.2f", a[idx]
        }'
}

UPTIME_CHECKS=0
UPTIME_PASS=0
for f in "$CSV_DIR"/uptime_log_*.txt; do
    if [ -f "$f" ]; then
        while IFS= read -r line; do
            pass=$(echo "$line" | grep -oP '\d+(?=/)')
            total=$(echo "$line" | grep -oP '(?<=/)\d+')
            UPTIME_PASS=$((UPTIME_PASS + pass))
            UPTIME_CHECKS=$((UPTIME_CHECKS + total))
        done < "$f"
    fi
done

if [ "$UPTIME_CHECKS" -gt 0 ]; then
    UPTIME_PCT=$(echo "scale=4; $UPTIME_PASS * 100 / $UPTIME_CHECKS" | bc)
else
    UPTIME_PCT="0"
fi

cat > "$OUTPUT" << EOF
# Cloudflare Tunnel Baseline — lab.primals.eco
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)
# Source: ${COUNT} days of hourly captures (${TOTAL_SAMPLES} total samples)

[metadata]
target = "https://lab.primals.eco/hub/login"
capture_days = ${COUNT}
total_samples = ${TOTAL_SAMPLES}
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[latency_ms]
# DNS lookup
dns_p50 = $(percentile 2 50)
dns_p95 = $(percentile 2 95)
dns_p99 = $(percentile 2 99)

# TCP connect
tcp_p50 = $(percentile 3 50)
tcp_p95 = $(percentile 3 95)
tcp_p99 = $(percentile 3 99)

# TLS handshake (time_appconnect)
tls_p50 = $(percentile 4 50)
tls_p95 = $(percentile 4 95)
tls_p99 = $(percentile 4 99)

# Time to first byte
ttfb_p50 = $(percentile 5 50)
ttfb_p95 = $(percentile 5 95)
ttfb_p99 = $(percentile 5 99)

# Total request time
total_p50 = $(percentile 6 50)
total_p95 = $(percentile 6 95)
total_p99 = $(percentile 6 99)

[uptime]
checks_total = ${UPTIME_CHECKS}
checks_passed = ${UPTIME_PASS}
uptime_pct = ${UPTIME_PCT}

[throughput]
# Average bytes/sec across all samples
avg_bytes_per_sec = $(awk -F, '{sum+=$8; n++} END {if(n>0) printf "%.0f", sum/n; else print 0}' "$COMBINED")

[thresholds]
# Replacement must meet or beat these to pass parity
max_ttfb_p95_ms = $(percentile 5 95)
max_tls_p95_ms = $(percentile 4 95)
min_uptime_pct = ${UPTIME_PCT}
EOF

rm -f "$COMBINED"
echo "Baseline written to $OUTPUT"
