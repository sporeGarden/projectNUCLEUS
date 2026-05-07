#!/usr/bin/env bash
set -euo pipefail

# Compares Songbird NAT traversal against cloudflared tunnel.
#
# Measures connection reliability, latency, and throughput to determine
# if Songbird can replace cloudflared as the tunnel mechanism.
#
# Prerequisites:
#   - Songbird NAT traversal running and reachable
#   - cloudflared tunnel still active for comparison
#
# Usage:
#   ./songbird_nat_parity.sh \
#     --cf-url https://lab.primals.eco/hub/login \
#     --songbird-url http://<songbird-endpoint>/hub/login \
#     [--samples 20] [--duration 300]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPORTS_DIR="$SCRIPT_DIR/../reports"
CF_URL="https://lab.primals.eco/hub/login"
SONGBIRD_URL=""
SAMPLES=20
DURATION=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cf-url)        CF_URL="$2"; shift 2 ;;
        --songbird-url)  SONGBIRD_URL="$2"; shift 2 ;;
        --samples)       SAMPLES="$2"; shift 2 ;;
        --duration)      DURATION="$2"; shift 2 ;;
        *)               echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

if [[ -z "$SONGBIRD_URL" ]]; then
    echo "Usage: $0 --songbird-url <URL> [--cf-url URL] [--samples N]" >&2
    echo ""
    echo "Songbird NAT traversal must be running and reachable."
    echo "This scenario is a stub until Songbird NAT is implemented (Step 3c)."
    exit 1
fi

mkdir -p "$REPORTS_DIR"
RUN_ID="$(date -u +%Y%m%d-%H%M%S)"
REPORT="$REPORTS_DIR/songbird_nat_parity_${RUN_ID}.toml"

echo "benchScale: Songbird NAT Parity"
echo "  Cloudflare: $CF_URL"
echo "  Songbird:   $SONGBIRD_URL"
echo "  Samples:    $SAMPLES"
echo ""

CURL_FMT='%{time_starttransfer},%{time_total},%{http_code}'

run_samples() {
    local url="$1" label="$2"
    local pass=0 ttfb_sum=0 total_sum=0

    for i in $(seq 1 "$SAMPLES"); do
        raw=$(curl -sS -o /dev/null -w "$CURL_FMT" --max-time 30 "$url" 2>/dev/null) || raw="0,0,0"
        IFS=',' read -r ttfb total code <<< "$raw"

        ttfb_ms=$(echo "$ttfb * 1000" | bc)
        total_ms=$(echo "$total * 1000" | bc)

        ttfb_sum=$(echo "$ttfb_sum + $ttfb_ms" | bc)
        total_sum=$(echo "$total_sum + $total_ms" | bc)

        [[ "$code" == "200" ]] && pass=$((pass + 1))

        printf "  [%s %d/%d] HTTP %s  ttfb=%sms\n" "$label" "$i" "$SAMPLES" "$code" "$ttfb_ms"
        sleep 1
    done

    local avg_ttfb=$(echo "scale=2; $ttfb_sum / $SAMPLES" | bc)
    local avg_total=$(echo "scale=2; $total_sum / $SAMPLES" | bc)
    local uptime=$(echo "scale=1; $pass * 100 / $SAMPLES" | bc)

    echo "$avg_ttfb,$avg_total,$uptime"
}

echo "--- Cloudflare ---"
cf_result=$(run_samples "$CF_URL" "CF")
cf_line=$(echo "$cf_result" | tail -1)
CF_TTFB=$(echo "$cf_line" | cut -d, -f1)
CF_TOTAL=$(echo "$cf_line" | cut -d, -f2)
CF_UPTIME=$(echo "$cf_line" | cut -d, -f3)

echo ""
echo "--- Songbird ---"
sb_result=$(run_samples "$SONGBIRD_URL" "SB")
sb_line=$(echo "$sb_result" | tail -1)
SB_TTFB=$(echo "$sb_line" | cut -d, -f1)
SB_TOTAL=$(echo "$sb_line" | cut -d, -f2)
SB_UPTIME=$(echo "$sb_line" | cut -d, -f3)

TTFB_PARITY=$(echo "$SB_TTFB <= $CF_TTFB * 1.1" | bc -l 2>/dev/null || echo "0")
UPTIME_PARITY=$(echo "$SB_UPTIME >= $CF_UPTIME" | bc -l 2>/dev/null || echo "0")

echo ""
echo "Parity:"
echo "  TTFB:   SB=${SB_TTFB}ms  CF=${CF_TTFB}ms  $([ "$TTFB_PARITY" = "1" ] && echo "PASS" || echo "FAIL")"
echo "  Uptime: SB=${SB_UPTIME}%  CF=${CF_UPTIME}%  $([ "$UPTIME_PARITY" = "1" ] && echo "PASS" || echo "FAIL")"

cat > "$REPORT" << EOF
# Songbird NAT Parity Report — $RUN_ID
# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)

[metadata]
cf_url = "$CF_URL"
songbird_url = "$SONGBIRD_URL"
samples = $SAMPLES
generated_at = "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

[cloudflare]
avg_ttfb_ms = $CF_TTFB
avg_total_ms = $CF_TOTAL
uptime_pct = $CF_UPTIME

[songbird]
avg_ttfb_ms = $SB_TTFB
avg_total_ms = $SB_TOTAL
uptime_pct = $SB_UPTIME

[parity]
ttfb_parity = $([ "$TTFB_PARITY" = "1" ] && echo "true" || echo "false")
uptime_parity = $([ "$UPTIME_PARITY" = "1" ] && echo "true" || echo "false")
overall = $([ "$TTFB_PARITY" = "1" ] && [ "$UPTIME_PARITY" = "1" ] && echo "true" || echo "false")
EOF

echo "Report: $REPORT"
