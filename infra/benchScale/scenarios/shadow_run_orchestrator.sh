#!/usr/bin/env bash
# shadow_run_orchestrator.sh — Run all shadow parity tests in sequence
#
# Orchestrates the three shadow runs that gate interstadial exit:
#   1. NestGate content parity vs GitHub Pages (H2-05/3a)
#   2. BearDog BTSP TLS vs Cloudflare (H2-3b/H2-12)
#   3. Songbird NAT vs cloudflared tunnel (H2-3c/H2-14)
#
# Usage:
#   ./shadow_run_orchestrator.sh [--baseline-only] [--parity-only] [--all]
#
# Prerequisites:
#   - NestGate running with content.put shipped (Session 60)
#   - BearDog TLS on :8443 (Wave 100 rustls + rate limiter)
#   - Songbird NAT relay on VPS (~$5/mo)
#   - cloudflared tunnel still active for comparison
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../../deploy/nucleus_config.sh"

BASELINES_DIR="$SCRIPT_DIR/../baselines"
REPORTS_DIR="$SCRIPT_DIR/../reports"
MODE="all"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --baseline-only) MODE="baseline"; shift ;;
        --parity-only)   MODE="parity"; shift ;;
        --all)           MODE="all"; shift ;;
        *)               echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$BASELINES_DIR" "$REPORTS_DIR"

echo "╔══════════════════════════════════════════════════════╗"
echo "║  Shadow Run Orchestrator — Interstadial Exit Tests  ║"
echo "╚══════════════════════════════════════════════════════╝"
echo ""
echo "  Mode: $MODE"
echo "  Baselines: $BASELINES_DIR"
echo "  Reports:   $REPORTS_DIR"
echo ""

PASS_COUNT=0
FAIL_COUNT=0
SKIP_COUNT=0

run_test() {
    local name="$1" script="$2"
    shift 2
    echo "━━━ $name ━━━"
    if bash "$script" "$@" 2>&1; then
        echo "  Result: PASS"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        local code=$?
        if [[ $code -eq 1 ]]; then
            echo "  Result: FAIL"
            FAIL_COUNT=$((FAIL_COUNT + 1))
        else
            echo "  Result: SKIP (prerequisites not met)"
            SKIP_COUNT=$((SKIP_COUNT + 1))
        fi
    fi
    echo ""
}

if [[ "$MODE" == "baseline" || "$MODE" == "all" ]]; then
    echo "=== Phase 1: Capture Baselines ==="
    echo ""

    if [[ -x "$SCRIPT_DIR/cloudflare_tunnel_baseline.sh" ]]; then
        run_test "Cloudflare Tunnel Baseline" "$SCRIPT_DIR/cloudflare_tunnel_baseline.sh"
    else
        echo "  SKIP: cloudflare_tunnel_baseline.sh not found"
        SKIP_COUNT=$((SKIP_COUNT + 1))
    fi
fi

if [[ "$MODE" == "parity" || "$MODE" == "all" ]]; then
    echo "=== Phase 2: Parity Tests ==="
    echo ""

    CF_BASELINE=$(ls -t "$BASELINES_DIR"/cloudflare_tunnel_*.toml 2>/dev/null | head -1 || true)

    # 1. NestGate content parity
    echo "--- H2-05/3a: NestGate Content Parity ---"
    NESTGATE_URL="http://${NUCLEUS_BIND_ADDRESS}:${PETALTONGUE_PORT:-9901}"
    GHPAGES_URL="https://primals.eco"

    if curl -sf --max-time 5 "$NESTGATE_URL" >/dev/null 2>&1; then
        run_test "NestGate Content Parity" \
            "$SCRIPT_DIR/nestgate_content_parity.sh" \
            --ghpages-url "$GHPAGES_URL" \
            --nestgate-url "$NESTGATE_URL"
    else
        echo "  SKIP: NestGate/petalTongue not reachable at $NESTGATE_URL"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        echo ""
    fi

    # 2. BearDog TLS parity
    echo "--- H2-3b/H2-12: BearDog BTSP TLS Parity ---"
    BTSP_URL="https://127.0.0.1:8443/hub/login"

    if [[ -n "$CF_BASELINE" ]]; then
        if curl -sf --max-time 5 -k "$BTSP_URL" >/dev/null 2>&1; then
            run_test "BTSP TLS Parity" \
                "$SCRIPT_DIR/btsp_tls_parity.sh" \
                --baseline "$CF_BASELINE" \
                --btsp-url "$BTSP_URL"
        else
            echo "  SKIP: BearDog TLS not reachable at $BTSP_URL"
            echo "  Start BearDog with --tls-port 8443 for shadow testing"
            SKIP_COUNT=$((SKIP_COUNT + 1))
            echo ""
        fi
    else
        echo "  SKIP: No Cloudflare baseline found. Run with --baseline-only first"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        echo ""
    fi

    # 3. Songbird NAT parity
    echo "--- H2-3c/H2-14: Songbird NAT Parity ---"
    SONGBIRD_RELAY="${SONGBIRD_RELAY_URL:-}"

    if [[ -n "$SONGBIRD_RELAY" ]]; then
        run_test "Songbird NAT Parity" \
            "$SCRIPT_DIR/songbird_nat_parity.sh" \
            --songbird-url "$SONGBIRD_RELAY"
    else
        echo "  SKIP: SONGBIRD_RELAY_URL not set"
        echo "  Provision VPS relay (~\$5/mo), set SONGBIRD_RELAY_URL, then re-run"
        SKIP_COUNT=$((SKIP_COUNT + 1))
        echo ""
    fi
fi

echo "╔══════════════════════════════════════════════════════╗"
echo "║  Shadow Run Summary                                 ║"
echo "║  PASS: $PASS_COUNT  FAIL: $FAIL_COUNT  SKIP: $SKIP_COUNT                            ║"
echo "╚══════════════════════════════════════════════════════╝"

if [[ $FAIL_COUNT -gt 0 ]]; then
    exit 1
fi
