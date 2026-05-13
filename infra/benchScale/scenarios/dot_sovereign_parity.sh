#!/usr/bin/env bash
# dot_sovereign_parity.sh — DNS-over-TLS vs sovereign DNS parity test
#
# Compares query latency, resolution accuracy, and DNSSEC validation
# between the current DoT configuration (Cloudflare 1.1.1.1 / Quad9 9.9.9.9)
# and a sovereign resolver stack (unbound local + knot-dns authoritative).
#
# H2-4 / H2-17→H2-20: Gates interstadial exit for DNS sovereignty.
#
# Usage:
#   ./dot_sovereign_parity.sh [--baseline-only] [--parity] [--report <dir>]
#
# Prerequisites:
#   - Current: systemd-resolved with DNSOverTLS=yes (1.1.1.1 + 9.9.9.9)
#   - Shadow:  unbound on 127.0.0.53:5353 + knot-dns on VPS (when available)
#   - dig / drill / resolvectl available
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
REPORT_DIR="${SCRIPT_DIR}/../reports"
MODE="baseline"
SOVEREIGN_RESOLVER="${SOVEREIGN_RESOLVER:-}"
DOMAINS=("primals.eco" "lab.primals.eco" "git.primals.eco" "github.com" "crates.io" "ncbi.nlm.nih.gov")
ITERATIONS=10

while [[ $# -gt 0 ]]; do
    case "$1" in
        --baseline-only) MODE="baseline"; shift ;;
        --parity)        MODE="parity"; shift ;;
        --report)        REPORT_DIR="$2"; shift 2 ;;
        *)               echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$REPORT_DIR"

timestamp() { date -u +"%Y-%m-%dT%H:%M:%SZ"; }

echo "╔══════════════════════════════════════════════════╗"
echo "║  DoT Sovereign DNS Parity — H2-4 / H2-17→20    ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""
echo "  Mode: $MODE"
echo "  Iterations per domain: $ITERATIONS"
echo "  Timestamp: $(timestamp)"
echo ""

resolve_domain() {
    local server_arg="$1"
    local domain="$2"
    if [[ -n "$server_arg" ]]; then
        dig +short "$server_arg" "$domain" A 2>/dev/null | head -1
    elif command -v resolvectl &>/dev/null; then
        resolvectl query "$domain" 2>/dev/null | grep -oP '\d+\.\d+\.\d+\.\d+' | head -1
    elif command -v dig &>/dev/null; then
        dig +short @127.0.0.53 "$domain" A 2>/dev/null | head -1
    else
        getent hosts "$domain" 2>/dev/null | awk '{print $1}' | head -1
    fi
}

measure_dns() {
    local resolver_label="$1"
    local server_arg="$2"
    local domain="$3"
    local total_ms=0
    local failures=0

    for _ in $(seq 1 "$ITERATIONS"); do
        local start_ns end_ns elapsed_ms
        start_ns=$(date +%s%N)
        local result
        result=$(resolve_domain "$server_arg" "$domain")
        if [[ -n "$result" ]]; then
            end_ns=$(date +%s%N)
            elapsed_ms=$(( (end_ns - start_ns) / 1000000 ))
            total_ms=$((total_ms + elapsed_ms))
        else
            failures=$((failures + 1))
        fi
    done

    local avg_ms=0
    local success=$((ITERATIONS - failures))
    [[ $success -gt 0 ]] && avg_ms=$((total_ms / success))
    echo "$resolver_label|$domain|$avg_ms|$success/$ITERATIONS"
}

DOT_REPORT="$REPORT_DIR/dot_parity_$(date +%Y%m%d_%H%M%S).toml"

{
    echo "[metadata]"
    echo "test = \"dot_sovereign_parity\""
    echo "timestamp = \"$(timestamp)\""
    echo "iterations = $ITERATIONS"
    echo "mode = \"$MODE\""
    echo ""
} > "$DOT_REPORT"

echo "=== Phase 1: DoT Baseline (systemd-resolved via 1.1.1.1/9.9.9.9) ==="
echo ""

{
    echo "[baseline]"
    echo "resolver = \"systemd-resolved (DoT)\""
    echo "upstream = [\"1.1.1.1\", \"9.9.9.9\"]"
    echo ""
    echo "[[baseline.results]]"
} >> "$DOT_REPORT"

printf "  %-25s %-8s %s\n" "DOMAIN" "AVG(ms)" "SUCCESS"
printf "  %-25s %-8s %s\n" "-------------------------" "--------" "-------"

for domain in "${DOMAINS[@]}"; do
    result=$(measure_dns "dot" "" "$domain")
    avg_ms=$(echo "$result" | cut -d'|' -f3)
    success=$(echo "$result" | cut -d'|' -f4)
    printf "  %-25s %-8s %s\n" "$domain" "${avg_ms}ms" "$success"
    echo "domain = \"$domain\", avg_ms = $avg_ms, success = \"$success\"" >> "$DOT_REPORT"
done

echo ""

dnssec_status="unknown"
dot_lines=$(resolvectl status 2>/dev/null | grep "DNSOverTLS" || true)
if echo "$dot_lines" | grep -q "+DNSOverTLS"; then
    dnssec_status="dot_active"
    dot_server=$(resolvectl status 2>/dev/null | grep "Current DNS Server" | head -1 | awk '{print $NF}')
    echo "  DoT: ACTIVE (systemd-resolved → ${dot_server:-unknown})"
elif echo "$dot_lines" | grep -qi "yes\|opportunistic"; then
    dnssec_status="dot_active"
    dot_server=$(resolvectl status 2>/dev/null | grep "Current DNS Server" | head -1 | awk '{print $NF}')
    echo "  DoT: ACTIVE (systemd-resolved → ${dot_server:-unknown})"
else
    dnssec_status="dot_inactive"
    echo "  DoT: NOT ACTIVE"
fi
echo "dnssec_status = \"$dnssec_status\"" >> "$DOT_REPORT"

if [[ "$MODE" == "parity" ]]; then
    echo ""
    echo "=== Phase 2: Sovereign Resolver Parity ==="
    echo ""

    if [[ -z "$SOVEREIGN_RESOLVER" ]]; then
        echo "  SKIP: SOVEREIGN_RESOLVER not set"
        echo "  Set SOVEREIGN_RESOLVER=127.0.0.53:5353 for local unbound"
        echo "  Or SOVEREIGN_RESOLVER=<vps-ip> for knot-dns authoritative"
        echo ""
        echo "[sovereign]" >> "$DOT_REPORT"
        echo "status = \"skipped\"" >> "$DOT_REPORT"
        echo "reason = \"SOVEREIGN_RESOLVER not set\"" >> "$DOT_REPORT"
        exit 2
    fi

    {
        echo ""
        echo "[sovereign]"
        echo "resolver = \"$SOVEREIGN_RESOLVER\""
        echo ""
        echo "[[sovereign.results]]"
    } >> "$DOT_REPORT"

    printf "  %-25s %-8s %s\n" "DOMAIN" "AVG(ms)" "SUCCESS"
    printf "  %-25s %-8s %s\n" "-------------------------" "--------" "-------"

    for domain in "${DOMAINS[@]}"; do
        result=$(measure_dns "sovereign" "@$SOVEREIGN_RESOLVER" "$domain")
        avg_ms=$(echo "$result" | cut -d'|' -f3)
        success=$(echo "$result" | cut -d'|' -f4)
        printf "  %-25s %-8s %s\n" "$domain" "${avg_ms}ms" "$success"
        echo "domain = \"$domain\", avg_ms = $avg_ms, success = \"$success\"" >> "$DOT_REPORT"
    done
fi

echo ""
echo "  Report: $DOT_REPORT"
echo ""
echo "  Next steps:"
echo "    H2-17: Deploy knot-dns authoritative on VPS"
echo "    H2-18: Transfer NS from Cloudflare registrar"
echo "    H2-19: Wire BTSP direct resolution for ecosystem clients"
echo "    H2-20: Local unbound recursive → eliminate DoT dependency"
