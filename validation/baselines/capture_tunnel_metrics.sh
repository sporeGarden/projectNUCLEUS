#!/usr/bin/env bash
set -euo pipefail

# Captures Cloudflare tunnel performance metrics for lab.primals.eco.
# Designed to run via cron every hour, appending to a daily CSV.

BASELINES_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$BASELINES_DIR/../../deploy/nucleus_config.sh" 2>/dev/null || true
TARGET_URL="${LAB_URL:-https://lab.primals.eco}/hub/login"
CSV_DIR="${BASELINES_DIR}/daily"
mkdir -p "$CSV_DIR"

TODAY="$(date -u +%Y-%m-%d)"
CSV_FILE="${CSV_DIR}/tunnel_metrics_${TODAY}.csv"

if [ ! -f "$CSV_FILE" ]; then
    echo "timestamp_utc,dns_lookup_ms,tcp_connect_ms,tls_handshake_ms,ttfb_ms,total_ms,http_code,throughput_bytes_per_sec,content_length_bytes" > "$CSV_FILE"
fi

CURL_FORMAT='%{time_namelookup},%{time_connect},%{time_appconnect},%{time_starttransfer},%{time_total},%{http_code},%{speed_download},%{size_download}'

SAMPLES=5
for _ in $(seq 1 $SAMPLES); do
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    result=$(curl -sS -o /dev/null -w "$CURL_FORMAT" --max-time 30 "$TARGET_URL" 2>/dev/null || echo "0,0,0,0,0,0,0,0")

    IFS=',' read -r dns tcp tls ttfb total code speed size <<< "$result"

    dns_ms=$(echo "$dns * 1000" | bc)
    tcp_ms=$(echo "$tcp * 1000" | bc)
    tls_ms=$(echo "$tls * 1000" | bc)
    ttfb_ms=$(echo "$ttfb * 1000" | bc)
    total_ms=$(echo "$total * 1000" | bc)

    echo "${ts},${dns_ms},${tcp_ms},${tls_ms},${ttfb_ms},${total_ms},${code},${speed},${size}" >> "$CSV_FILE"
    sleep 2
done

# Quick uptime check (3 pings in sequence)
UPTIME_PASS=0
UPTIME_TOTAL=3
for _ in $(seq 1 $UPTIME_TOTAL); do
    code=$(curl -sS -o /dev/null -w "%{http_code}" --max-time 10 "$TARGET_URL" 2>/dev/null || echo "0")
    if [ "$code" = "200" ]; then
        UPTIME_PASS=$((UPTIME_PASS + 1))
    fi
    sleep 1
done

UPTIME_PCT=$(echo "scale=1; $UPTIME_PASS * 100 / $UPTIME_TOTAL" | bc)
echo "$(date -u +%Y-%m-%dT%H:%M:%SZ) uptime_check: ${UPTIME_PASS}/${UPTIME_TOTAL} (${UPTIME_PCT}%)" >> "${CSV_DIR}/uptime_log_${TODAY}.txt"

echo "[$(date -u +%H:%M:%S)] Captured ${SAMPLES} samples, uptime ${UPTIME_PCT}% -> ${CSV_FILE}"
