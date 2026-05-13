#!/usr/bin/env bash
set -euo pipefail

# Compares NestGate/petalTongue self-hosted content serving against GitHub Pages.
#
# Measures:
#   - TTFB (Time to First Byte)
#   - LCP proxy (total page load time)
#   - Content hash parity (same content served by both)
#
# Prerequisites:
#   - petalTongue web mode serving sporePrint content
#   - GitHub Pages still live at primals.eco
#
# Usage:
#   ./nestgate_content_parity.sh \
#     --ghpages-url https://primals.eco \
#     --nestgate-url http://127.0.0.1:9900 \
#     [--samples 10]

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../../deploy/nucleus_config.sh"

REPORTS_DIR="$SCRIPT_DIR/../reports"
GHPAGES_URL="https://primals.eco"
NESTGATE_URL="http://${NUCLEUS_BIND_ADDRESS}:${PETALTONGUE_PORT}"
SAMPLES=10
PATHS=("/" "/lab/compute-access/" "/about/")

while [[ $# -gt 0 ]]; do
    case "$1" in
        --ghpages-url)  GHPAGES_URL="$2"; shift 2 ;;
        --nestgate-url) NESTGATE_URL="$2"; shift 2 ;;
        --samples)      SAMPLES="$2"; shift 2 ;;
        *)              echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$REPORTS_DIR"
RUN_ID="$(date -u +%Y%m%d-%H%M%S)"
REPORT="$REPORTS_DIR/nestgate_content_parity_${RUN_ID}.toml"

echo "benchScale: NestGate Content Parity"
echo "  GitHub Pages: $GHPAGES_URL"
echo "  NestGate:     $NESTGATE_URL"
echo "  Samples:      $SAMPLES per path"
echo ""

CURL_FMT='%{time_starttransfer},%{time_total},%{http_code},%{size_download}'

measure_path() {
    local url="$1" path="$2" label="$3"
    local ttfb_sum=0 total_sum=0 count=0

    for _ in $(seq 1 "$SAMPLES"); do
        raw=$(curl -sS -o /dev/null -w "$CURL_FMT" --max-time 15 "${url}${path}" 2>/dev/null) || raw="0,0,0,0"
        IFS=',' read -r ttfb total code size <<< "$raw"

        ttfb_ms=$(echo "$ttfb * 1000" | bc)
        total_ms=$(echo "$total * 1000" | bc)

        ttfb_sum=$(echo "$ttfb_sum + $ttfb_ms" | bc)
        total_sum=$(echo "$total_sum + $total_ms" | bc)
        count=$((count + 1))

        sleep 1
    done

    local avg_ttfb=$(echo "scale=2; $ttfb_sum / $count" | bc)
    local avg_total=$(echo "scale=2; $total_sum / $count" | bc)
    printf "  %-12s %s  avg_ttfb=%sms  avg_total=%sms\n" "$label" "$path" "$avg_ttfb" "$avg_total"
    echo "$avg_ttfb,$avg_total"
}

content_hash() {
    local url="$1" path="$2"
    curl -sS --max-time 15 "${url}${path}" 2>/dev/null | sha256sum | cut -d' ' -f1
}

{
    echo "# NestGate Content Parity Report — $RUN_ID"
    echo "# Generated: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo ""
    echo "[metadata]"
    echo "ghpages_url = \"$GHPAGES_URL\""
    echo "nestgate_url = \"$NESTGATE_URL\""
    echo "samples_per_path = $SAMPLES"
    echo "generated_at = \"$(date -u +%Y-%m-%dT%H:%M:%SZ)\""
    echo ""
} > "$REPORT"

OVERALL_PARITY=true

for path in "${PATHS[@]}"; do
    echo "Testing path: $path"

    gh_result=$(measure_path "$GHPAGES_URL" "$path" "GH Pages")
    gh_ttfb=$(echo "$gh_result" | tail -1 | cut -d, -f1)
    gh_total=$(echo "$gh_result" | tail -1 | cut -d, -f2)

    ng_result=$(measure_path "$NESTGATE_URL" "$path" "NestGate")
    ng_ttfb=$(echo "$ng_result" | tail -1 | cut -d, -f1)
    ng_total=$(echo "$ng_result" | tail -1 | cut -d, -f2)

    gh_hash=$(content_hash "$GHPAGES_URL" "$path")
    ng_hash=$(content_hash "$NESTGATE_URL" "$path")
    hash_match=$( [[ "$gh_hash" == "$ng_hash" ]] && echo "true" || echo "false" )
    ttfb_ok=$(echo "$ng_ttfb <= $gh_ttfb * 1.1" | bc -l 2>/dev/null || echo "0")

    if [[ "$hash_match" == "false" || "$ttfb_ok" == "0" ]]; then
        OVERALL_PARITY=false
    fi

    safe_path=$(echo "$path" | tr '/' '_' | sed 's/^_//' | sed 's/_$//')
    [[ -z "$safe_path" ]] && safe_path="root"

    {
        echo "[path_${safe_path}]"
        echo "path = \"$path\""
        echo "ghpages_ttfb_ms = $gh_ttfb"
        echo "ghpages_total_ms = $gh_total"
        echo "nestgate_ttfb_ms = $ng_ttfb"
        echo "nestgate_total_ms = $ng_total"
        echo "content_hash_match = $hash_match"
        echo "ttfb_parity = $([ "$ttfb_ok" = "1" ] && echo "true" || echo "false")"
        echo ""
    } >> "$REPORT"

    echo ""
done

{
    echo "[parity]"
    echo "overall = $OVERALL_PARITY"
} >> "$REPORT"

echo "Report: $REPORT"
