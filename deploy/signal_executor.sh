#!/usr/bin/env bash
# signal_executor.sh — Bridge squirrel signal_plan to biomeOS signal.dispatch
#
# Closes the agent loop: takes a natural language intent, asks squirrel
# to decompose it into atomic signals via signal_plan mode, then dispatches
# each signal through biomeOS's Neural API.
#
# biomeOS signal.dispatch accepts:
#   { "signal": "tier.operation", "params": {...} }
#   { "tier": "tower", "operation": "publish", "params": {...} }
#
# biomeOS returns:
#   { "signal": "tier.operation", "graph_id": "signals/tier_op", "execution": {...} }
#
# Usage:
#   bash signal_executor.sh "check the health of all tower primals"
#   bash signal_executor.sh --plan-only "deploy a nest composition"
#   bash signal_executor.sh --signal tower.health      # dispatch one signal directly
#   bash signal_executor.sh --dry-run "store this data securely"
#   bash signal_executor.sh --shadow                   # validate graph via shadow deploy
#
# Prerequisites:
#   - Agent composition running (deploy.sh --composition agent --graph-deploy)
#   - squirrel on SQUIRREL_PORT (default 9300)
#   - biomeOS neural-api on BIOMEOS_PORT (default 9800)
#   - signal_tools.toml accessible to squirrel

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh" 2>/dev/null || true

SQUIRREL_URL="http://127.0.0.1:${SQUIRREL_PORT:-9300}"
BIOMEOS_URL="http://127.0.0.1:${BIOMEOS_PORT:-9800}"
SIGNAL_TOOLS="${SIGNAL_TOOLS:-}"

PLAN_ONLY=false
DRY_RUN=false
SHADOW=false
DIRECT_SIGNAL=""
DIRECT_PARAMS="{}"
INTENT=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --plan-only)   PLAN_ONLY=true; shift ;;
        --dry-run)     DRY_RUN=true; shift ;;
        --shadow)      SHADOW=true; shift ;;
        --signal)      DIRECT_SIGNAL="$2"; shift 2 ;;
        --params)      DIRECT_PARAMS="$2"; shift 2 ;;
        --squirrel)    SQUIRREL_URL="http://127.0.0.1:$2"; shift 2 ;;
        --biomeos)     BIOMEOS_URL="http://127.0.0.1:$2"; shift 2 ;;
        --tools)       SIGNAL_TOOLS="$2"; shift 2 ;;
        --help)
            echo "Usage: $0 [OPTIONS] \"intent\""
            echo ""
            echo "Options:"
            echo "  --plan-only        Show the signal plan without dispatching"
            echo "  --dry-run          Show dispatch calls without executing"
            echo "  --shadow           Run composition.deploy.shadow validation"
            echo "  --signal SIG       Dispatch a single signal directly (skip planning)"
            echo "  --params JSON      JSON params for --signal (default: {})"
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
    local id="${4:-1}"
    curl -sf --max-time 30 "$url" \
        -X POST -H 'Content-Type: application/json' \
        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"params\":$params,\"id\":$id}" \
        2>/dev/null
}

extract_error() {
    python3 -c "
import sys, json
d = json.load(sys.stdin)
err = d.get('error')
if err:
    print(err.get('message', str(err)))
" 2>/dev/null <<< "$1" || true
}

extract_result() {
    python3 -c "
import sys, json
d = json.load(sys.stdin)
r = d.get('result', {})
print(json.dumps(r, indent=2))
" 2>/dev/null <<< "$1" || true
}

# ── Shadow deploy validation ─────────────────────────────────────────

if $SHADOW; then
    echo "signal_executor: shadow deploy validation"
    echo ""
    local_result=$(rpc_call "$BIOMEOS_URL" "composition.deploy.shadow" \
        '{"graph_id":"tower_agent"}')

    if [[ -z "$local_result" ]]; then
        echo "  [FAIL] No response from biomeOS at $BIOMEOS_URL"
        exit 1
    fi

    error=$(extract_error "$local_result")
    if [[ -n "$error" ]]; then
        echo "  [FAIL] composition.deploy.shadow: $error"
        exit 1
    fi

    python3 -c "
import sys, json
d = json.load(sys.stdin).get('result', {})
valid = d.get('valid', False)
status = 'PASS' if valid else 'FAIL'
print(f'  [{status}] Graph: {d.get(\"graph_id\",\"?\")} v{d.get(\"version\",\"?\")}')
print(f'  Nodes: {d.get(\"node_count\",0)}, Phases: {d.get(\"phase_count\",0)}')
print(f'  Coordination: {d.get(\"coordination\",\"?\")}')
comp = d.get('composition_model')
if comp:
    print(f'  Composition model: {comp}')
integrity = d.get('integrity', {})
if integrity:
    print(f'  Integrity: hash={integrity.get(\"content_hash\",\"?\")[:16]}..., match={integrity.get(\"hash_match\")}')
for cap in d.get('capability_resolution', []):
    r = cap.get('resolvable', True)
    tag = 'OK' if r else 'WARN'
    print(f'  [{tag}] {cap.get(\"node\",\"?\")} → {cap.get(\"resolved_provider\",\"unresolved\")}')
for err in d.get('validation_errors', []):
    print(f'  [ERROR] {err}')
for w in d.get('warnings', []):
    print(f'  [WARN] {w}')
" <<< "$local_result" 2>/dev/null || echo "  (could not parse shadow deploy result)"
    exit 0
fi

# ── Direct signal dispatch (bypass planning) ──────────────────────────

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
    error=$(extract_error "$result")
    if [[ -n "$error" ]]; then
        echo "  [FAIL] signal.dispatch: $signal — $error"
        return 1
    fi

    echo "  [OK] $signal"
    local execution
    execution=$(python3 -c "
import sys, json
d = json.load(sys.stdin).get('result', {})
print(f'    graph: {d.get(\"graph_id\",\"?\")}')
exec_result = d.get('execution', {})
if isinstance(exec_result, dict):
    status = exec_result.get('status', exec_result.get('state', ''))
    if status:
        print(f'    status: {status}')
" 2>/dev/null <<< "$result" || true)
    [[ -n "$execution" ]] && echo "$execution"
    return 0
}

if [[ -n "$DIRECT_SIGNAL" ]]; then
    echo "signal_executor: direct dispatch — $DIRECT_SIGNAL"
    dispatch_signal "$DIRECT_SIGNAL" "$DIRECT_PARAMS"
    exit $?
fi

# ── Intent-based execution (plan then dispatch) ──────────────────────

if [[ -z "$INTENT" ]]; then
    echo "ERROR: Provide an intent string, --signal, or --shadow"
    echo "  Example: $0 \"check the health of all tower primals\""
    exit 1
fi

echo "signal_executor: planning — \"$INTENT\""
echo ""

ESCAPED_INTENT=$(python3 -c "import json; print(json.dumps('$INTENT'))" 2>/dev/null)

PLAN_PARAMS="{\"prompt\":$ESCAPED_INTENT,\"mode\":\"signal_plan\""
if [[ -n "$SIGNAL_TOOLS" ]]; then
    PLAN_PARAMS="$PLAN_PARAMS,\"tool_schema\":\"$SIGNAL_TOOLS\""
fi
PLAN_PARAMS="$PLAN_PARAMS}"

PLAN_RESULT=$(rpc_call "$SQUIRREL_URL" "ai.query" "$PLAN_PARAMS")

if [[ -z "$PLAN_RESULT" ]]; then
    echo "ERROR: No response from squirrel at $SQUIRREL_URL"
    echo "  Is the agent composition running? (deploy.sh --composition agent --graph-deploy)"
    exit 1
fi

PLAN_ERROR=$(extract_error "$PLAN_RESULT")
if [[ -n "$PLAN_ERROR" ]]; then
    echo "ERROR: squirrel signal_plan failed — $PLAN_ERROR"
    exit 1
fi

STEPS=$(python3 -c "
import sys, json
d = json.load(sys.stdin)
result = d.get('result', {})
steps = result.get('steps', result.get('plan', []))
if isinstance(steps, str):
    steps = json.loads(steps)
print(json.dumps(steps))
" 2>/dev/null <<< "$PLAN_RESULT")

STEP_COUNT=$(python3 -c "import sys, json; print(len(json.load(sys.stdin)))" 2>/dev/null <<< "$STEPS" || echo 0)

echo "Plan: $STEP_COUNT signal(s)"
python3 -c "
import sys, json
steps = json.load(sys.stdin)
for i, s in enumerate(steps):
    signal = s.get('signal', s.get('name', '?'))
    reason = s.get('reason', s.get('description', ''))
    print(f'  {i+1}. {signal}' + (f' — {reason}' if reason else ''))
" 2>/dev/null <<< "$STEPS" || true
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

while IFS=$'\t' read -r signal params; do
    if dispatch_signal "$signal" "$params"; then
        PASSED=$((PASSED + 1))
    else
        FAILED=$((FAILED + 1))
    fi
done < <(python3 -c "
import sys, json
steps = json.load(sys.stdin)
for s in steps:
    signal = s.get('signal', s.get('name', ''))
    params = json.dumps(s.get('params', s.get('parameters', {})))
    print(f'{signal}\t{params}')
" 2>/dev/null <<< "$STEPS")

echo ""
echo "signal_executor: complete ($PASSED passed, $FAILED failed)"
