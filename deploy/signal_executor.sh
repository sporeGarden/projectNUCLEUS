#!/usr/bin/env bash
# signal_executor.sh — Bridge squirrel signal_plan to biomeOS signal.dispatch
#
# Closes the agent loop: takes a natural language intent, asks squirrel
# to decompose it into atomic signals via signal_plan mode, then dispatches
# each signal through biomeOS's Neural API.
#
# Usage:
#   bash signal_executor.sh "check the health of all tower primals"
#   bash signal_executor.sh --plan-only "deploy a nest composition"
#   bash signal_executor.sh --signal tower.health      # dispatch one signal directly
#   bash signal_executor.sh --dry-run "store this data securely"
#
# Prerequisites:
#   - Agent composition running (deploy.sh --composition agent --graph-deploy)
#   - squirrel on SQUIRREL_PORT (default 9300)
#   - biomeOS neural-api on BIOMEOS_PORT (default 9800)
#   - signal_tools.toml accessible to squirrel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh" 2>/dev/null || true

SQUIRREL_URL="http://127.0.0.1:${SQUIRREL_PORT:-9300}"
BIOMEOS_URL="http://127.0.0.1:${BIOMEOS_PORT:-9800}"
SIGNAL_TOOLS="${SIGNAL_TOOLS:-$(cd "$SCRIPT_DIR/../.." 2>/dev/null && pwd)/springs/primalSpring/config/signal_tools.toml}"

PLAN_ONLY=false
DRY_RUN=false
DIRECT_SIGNAL=""
INTENT=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --plan-only)   PLAN_ONLY=true; shift ;;
        --dry-run)     DRY_RUN=true; shift ;;
        --signal)      DIRECT_SIGNAL="$2"; shift 2 ;;
        --squirrel)    SQUIRREL_URL="http://127.0.0.1:$2"; shift 2 ;;
        --biomeos)     BIOMEOS_URL="http://127.0.0.1:$2"; shift 2 ;;
        --tools)       SIGNAL_TOOLS="$2"; shift 2 ;;
        --help)
            echo "Usage: $0 [OPTIONS] \"intent\""
            echo ""
            echo "Options:"
            echo "  --plan-only        Show the signal plan without dispatching"
            echo "  --dry-run          Show dispatch calls without executing"
            echo "  --signal SIG       Dispatch a single signal directly (skip planning)"
            echo "  --squirrel PORT    Squirrel port (default: 9300)"
            echo "  --biomeos PORT     biomeOS Neural API port (default: 9800)"
            echo "  --tools PATH       Path to signal_tools.toml"
            exit 0
            ;;
        -*)            echo "Unknown option: $1"; exit 1 ;;
        *)             INTENT="$1"; shift ;;
    esac
done

rpc_call() {
    local url="$1"
    local method="$2"
    local params="$3"
    curl -sf --max-time 30 "$url" \
        -X POST -H 'Content-Type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":1}" \
        2>/dev/null
}

dispatch_signal() {
    local signal="$1"
    local params="${2:-{}}"

    if $DRY_RUN; then
        echo "  [dry-run] signal.dispatch: $signal $params"
        return 0
    fi

    local result
    result=$(rpc_call "$BIOMEOS_URL" "signal.dispatch" \
        "{\"signal\":\"$signal\",\"params\":$params}")

    if [[ -z "$result" ]]; then
        echo "  [FAIL] signal.dispatch: $signal — no response from biomeOS"
        return 1
    fi

    local error
    error=$(echo "$result" | python3 -c "import sys,json; d=json.load(sys.stdin); print(d.get('error',{}).get('message',''))" 2>/dev/null || true)
    if [[ -n "$error" ]]; then
        echo "  [FAIL] signal.dispatch: $signal — $error"
        return 1
    fi

    echo "  [OK] $signal"
    echo "$result" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',{}); print(json.dumps(r,indent=2))" 2>/dev/null || true
    return 0
}

# ── Direct signal dispatch (bypass planning) ──────────────────────────

if [[ -n "$DIRECT_SIGNAL" ]]; then
    echo "signal_executor: direct dispatch — $DIRECT_SIGNAL"
    dispatch_signal "$DIRECT_SIGNAL" "{}"
    exit $?
fi

# ── Intent-based execution (plan then dispatch) ──────────────────────

if [[ -z "$INTENT" ]]; then
    echo "ERROR: Provide an intent string or --signal"
    echo "  Example: $0 \"check the health of all tower primals\""
    exit 1
fi

echo "signal_executor: planning — \"$INTENT\""
echo ""

PLAN_PARAMS=$(python3 -c "
import json
print(json.dumps({
    'prompt': $(python3 -c "import json; print(json.dumps('$INTENT'))"),
    'mode': 'signal_plan',
    'tool_schema': '$SIGNAL_TOOLS'
}))
" 2>/dev/null)

PLAN_RESULT=$(rpc_call "$SQUIRREL_URL" "ai.query" "$PLAN_PARAMS")

if [[ -z "$PLAN_RESULT" ]]; then
    echo "ERROR: No response from squirrel at $SQUIRREL_URL"
    echo "  Is the agent composition running? (deploy.sh --composition agent --graph-deploy)"
    exit 1
fi

PLAN_ERROR=$(echo "$PLAN_RESULT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
if 'error' in d:
    print(d['error'].get('message', str(d['error'])))
" 2>/dev/null || true)

if [[ -n "$PLAN_ERROR" ]]; then
    echo "ERROR: squirrel signal_plan failed — $PLAN_ERROR"
    exit 1
fi

STEPS=$(echo "$PLAN_RESULT" | python3 -c "
import sys, json
d = json.load(sys.stdin)
result = d.get('result', {})
steps = result.get('steps', result.get('plan', []))
if isinstance(steps, str):
    steps = json.loads(steps)
print(json.dumps(steps))
" 2>/dev/null)

STEP_COUNT=$(echo "$STEPS" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))" 2>/dev/null || echo 0)

echo "Plan: $STEP_COUNT signal(s)"
echo "$STEPS" | python3 -c "
import sys, json
steps = json.load(sys.stdin)
for i, s in enumerate(steps):
    signal = s.get('signal', s.get('name', '?'))
    reason = s.get('reason', s.get('description', ''))
    print(f'  {i+1}. {signal}' + (f' — {reason}' if reason else ''))
" 2>/dev/null || true
echo ""

if $PLAN_ONLY; then
    echo "(plan-only mode — not dispatching)"
    exit 0
fi

# ── Dispatch each step ───────────────────────────────────────────────

echo "Dispatching..."
echo ""

PASSED=0
FAILED=0

echo "$STEPS" | python3 -c "
import sys, json
steps = json.load(sys.stdin)
for s in steps:
    signal = s.get('signal', s.get('name', ''))
    params = json.dumps(s.get('params', s.get('parameters', {})))
    print(f'{signal}\t{params}')
" 2>/dev/null | while IFS=$'\t' read -r signal params; do
    if dispatch_signal "$signal" "$params"; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done

echo ""
echo "signal_executor: complete"
