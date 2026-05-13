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
CERT_PATH="${GATE_HOME:?}/.beardog/tls/lab.primals.eco.pem"
KEY_PATH="${GATE_HOME:?}/.beardog/tls/lab.primals.eco.key"
BEARDOG_BIN="${PLASMIDBIN_DIR:-$GATE_HOME/plasmidBin}/primals/beardog"
BASELINE_DIR="${SCRIPT_DIR}/../validation/baselines"
SHADOW_LOG="/tmp/beardog-tls-shadow.log"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --cert) CERT_PATH="$2"; shift 2 ;;
        --key)  KEY_PATH="$2"; shift 2 ;;
        --port) SHADOW_PORT="$2"; shift 2 ;;
        *) shift ;;
    esac
done

echo "══ BearDog TLS Shadow Run (H2-12) ══"
echo "  Port:     $SHADOW_PORT"
echo "  Cert:     $CERT_PATH"
echo "  Key:      $KEY_PATH"
echo "  Binary:   $BEARDOG_BIN"
echo "  Log:      $SHADOW_LOG"
echo ""

if [[ ! -x "$BEARDOG_BIN" ]]; then
    echo "ERROR: BearDog binary not found at $BEARDOG_BIN"
    echo "  Run: deploy/fetch_primals.sh to pull from plasmidBin"
    exit 1
fi

# Phase 1: Generate self-signed cert if none exists
if [[ ! -f "$CERT_PATH" ]]; then
    echo "  Generating self-signed TLS certificate..."
    mkdir -p "$(dirname "$CERT_PATH")"
    openssl req -x509 -newkey ec -pkeyopt ec_paramgen_curve:prime256v1 \
        -nodes -keyout "$KEY_PATH" -out "$CERT_PATH" \
        -days 365 -subj "/CN=lab.primals.eco" \
        -addext "subjectAltName=DNS:lab.primals.eco,DNS:localhost" \
        2>/dev/null
    echo "  Self-signed cert written to $CERT_PATH"
fi

# Phase 2: Start BearDog TLS shadow
echo ""
echo "  Starting BearDog TLS shadow on :${SHADOW_PORT}..."
export BEARDOG_FAMILY_SEED="${BEACON_SEED:-$(head -c 32 /dev/urandom | base64)}"

nohup "$BEARDOG_BIN" server \
    --listen "127.0.0.1:${SHADOW_PORT}" \
    --tls-cert "$CERT_PATH" \
    --tls-key "$KEY_PATH" \
    --family-id "${FAMILY_ID:-nucleus}" \
    > "$SHADOW_LOG" 2>&1 &

SHADOW_PID=$!
echo "  PID: $SHADOW_PID"
sleep 2

if ! kill -0 "$SHADOW_PID" 2>/dev/null; then
    echo "  ERROR: BearDog TLS shadow failed to start"
    echo "  Check $SHADOW_LOG for details"
    exit 1
fi

# Phase 3: Baseline comparison probe
echo ""
echo "  Running baseline comparison..."

cf_time=$(curl -so /dev/null -w '%{time_total}' --max-time 5 \
    "https://lab.primals.eco" 2>/dev/null || echo "unreachable")

tls_time=$(curl -so /dev/null -w '%{time_total}' --max-time 5 -k \
    "https://127.0.0.1:${SHADOW_PORT}" 2>/dev/null || echo "unreachable")

echo "  Cloudflare latency: ${cf_time}s"
echo "  BearDog TLS latency: ${tls_time}s"

# Phase 4: Record baseline
TIMESTAMP=$(date -u +%Y%m%dT%H%M%SZ)
BASELINE_FILE="${BASELINE_DIR}/beardog_tls_shadow_${TIMESTAMP}.csv"
mkdir -p "$BASELINE_DIR"
echo "timestamp,cloudflare_s,beardog_tls_s,shadow_port,pid" > "$BASELINE_FILE"
echo "${TIMESTAMP},${cf_time},${tls_time},${SHADOW_PORT},${SHADOW_PID}" >> "$BASELINE_FILE"
echo ""
echo "  Baseline recorded: $BASELINE_FILE"

echo ""
echo "══ Shadow running ══"
echo "  BearDog TLS on :${SHADOW_PORT} (PID $SHADOW_PID)"
echo "  Cloudflare tunnel on :443 (unchanged)"
echo "  Comparison metrics accumulating in $BASELINE_DIR"
echo ""
echo "  To stop: kill $SHADOW_PID"
echo "  To monitor: tail -f $SHADOW_LOG"
echo "  Parity test: infra/benchScale/scenarios/btsp_tls_parity.sh"
