#!/usr/bin/env bash
# tier_test_all.sh — Unified test runner for all access tiers
#
# Runs observer, reviewer, compute, and JupyterHub API tests in sequence,
# producing a combined report. Each test explores the project from its tier's
# perspective, catching issues like missing kernels, broken notebooks,
# permission leaks, and rendering failures.
#
# Usage:
#     sudo bash deploy/tier_test_all.sh [--json] [--tier observer|reviewer|compute|hub|pappusCast|sporePrint|all]
#
# Outputs:
#     deploy/tier_test_results/<timestamp>/
#       observer.txt
#       reviewer.txt
#       compute.txt
#       hub.txt
#       summary.json

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
TIMESTAMP=$(date +%Y%m%d-%H%M%S)
RESULTS_DIR="${SCRIPT_DIR}/tier_test_results/${TIMESTAMP}"
mkdir -p "$RESULTS_DIR"

PYTHON="${PYTHON:-/home/irongate/miniforge3/envs/jupyterhub/bin/python3}"
TIER="${2:-all}"
JSON_FLAG=""

for arg in "$@"; do
    case "$arg" in
        --json) JSON_FLAG="--json" ;;
        --tier)  ;;
        observer|reviewer|compute|hub|pappusCast|sporePrint|all) TIER="$arg" ;;
    esac
done

TOTAL_PASS=0
TOTAL_FAIL=0

run_test() {
    local name="$1"
    local script="$2"
    shift 2
    local outfile="${RESULTS_DIR}/${name}.txt"

    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Running: ${name} tier test"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    set +e
    "$PYTHON" "$script" "$@" 2>&1 | tee "$outfile"
    local exit_code=$?
    set -e

    local pass_count
    pass_count=$(grep -c "^PASS|" "$outfile" 2>/dev/null) || pass_count=0
    local fail_count
    fail_count=$(grep -c "^FAIL|" "$outfile" 2>/dev/null) || fail_count=0

    TOTAL_PASS=$((TOTAL_PASS + pass_count))
    TOTAL_FAIL=$((TOTAL_FAIL + fail_count))

    echo "  → ${name}: ${pass_count} pass, ${fail_count} fail (exit=${exit_code})"
}

echo "╔═══════════════════════════════════════════════════╗"
echo "║  ABG Tier Test Suite                              ║"
echo "║  Date: $(date +%Y-%m-%dT%H:%M:%S%z)              ║"
echo "║  Tier: ${TIER}                                    ║"
echo "║  Output: ${RESULTS_DIR}                           ║"
echo "╚═══════════════════════════════════════════════════╝"

if [[ "$TIER" == "all" || "$TIER" == "observer" ]]; then
    run_test "observer" "${SCRIPT_DIR}/tier_test_observer.py"
fi

if [[ "$TIER" == "all" || "$TIER" == "reviewer" ]]; then
    run_test "reviewer" "${SCRIPT_DIR}/tier_test_reviewer.py"
fi

if [[ "$TIER" == "all" || "$TIER" == "compute" ]]; then
    run_test "compute" "${SCRIPT_DIR}/tier_test_compute.py"
fi

if [[ "$TIER" == "all" || "$TIER" == "hub" ]]; then
    run_test "hub" "${SCRIPT_DIR}/jupyterhub_tier_test.py" --skip-voila
fi

if [[ "$TIER" == "all" || "$TIER" == "pappusCast" ]]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Running: pappusCast health"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    set +e
    "$PYTHON" "${SCRIPT_DIR}/pappusCast.py" health 2>&1 | tee "${RESULTS_DIR}/pappusCast.txt"
    pg_exit=$?
    set -e
    if [ $pg_exit -eq 0 ]; then
        TOTAL_PASS=$((TOTAL_PASS + 1))
        echo "PASS|pappusCast|health|healthy" >> "${RESULTS_DIR}/pappusCast.txt"
    else
        TOTAL_FAIL=$((TOTAL_FAIL + 1))
        echo "FAIL|pappusCast|health|unhealthy" >> "${RESULTS_DIR}/pappusCast.txt"
    fi
    echo "  → pappusCast: health check (exit=${pg_exit})"
fi

if [[ "$TIER" == "all" || "$TIER" == "sporePrint" ]]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  Running: sporePrint dual-origin verification"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    set +e
    bash "${SCRIPT_DIR}/sporeprint_verify.sh" --tier-test 2>&1 | tee "${RESULTS_DIR}/sporePrint.txt"
    sp_exit=$?
    set -e
    sp_pass=$(grep -c "^PASS|" "${RESULTS_DIR}/sporePrint.txt" 2>/dev/null) || sp_pass=0
    sp_fail=$(grep -c "^FAIL|" "${RESULTS_DIR}/sporePrint.txt" 2>/dev/null) || sp_fail=0
    TOTAL_PASS=$((TOTAL_PASS + sp_pass))
    TOTAL_FAIL=$((TOTAL_FAIL + sp_fail))
    echo "  → sporePrint: ${sp_pass} pass, ${sp_fail} fail (exit=${sp_exit})"
fi

# Write summary
cat > "${RESULTS_DIR}/summary.json" << SUMEOF
{
  "timestamp": "${TIMESTAMP}",
  "tier_filter": "${TIER}",
  "total_pass": ${TOTAL_PASS},
  "total_fail": ${TOTAL_FAIL},
  "result_dir": "${RESULTS_DIR}"
}
SUMEOF

echo ""
echo "╔═══════════════════════════════════════════════════╗"
echo "║  Combined Results: ${TOTAL_PASS} PASS, ${TOTAL_FAIL} FAIL"
echo "║  Reports: ${RESULTS_DIR}/"
echo "╚═══════════════════════════════════════════════════╝"

if [ "$TOTAL_FAIL" -gt 0 ]; then
    echo ""
    echo "FAILURES DETECTED — review individual tier reports above"
    grep "^FAIL|" "${RESULTS_DIR}"/*.txt 2>/dev/null || true
fi

exit $(( TOTAL_FAIL > 125 ? 125 : TOTAL_FAIL ))
