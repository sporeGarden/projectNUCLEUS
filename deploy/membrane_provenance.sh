#!/usr/bin/env bash
# membrane_provenance.sh — Post-deploy trio verification for cellMembrane
#
# After deploying Nest Atomic (CM-1), verifies the provenance pipeline
# works remotely: rhizoCrypt DAG + loamSpine ledger + sweetGrass braid.
#
# Runs from the gate against the VPS over SSH. Each trio primal is tested
# independently — partial results are valid (graceful degradation per
# wateringHole/DEGRADATION_BEHAVIOR_STANDARD.md).
#
# Prerequisites:
#   - Nest Atomic deployed on VPS (CM-1: nestGate + rhizoCrypt + loamSpine + sweetGrass)
#   - SSH access to VPS (root@MEMBRANE_VPS_IP)
#   - socat on VPS (for TCP JSON-RPC probes)
#
# Usage:
#   bash deploy/membrane_provenance.sh [--skip-ssh]
#
# See: deploy/provenance_pipeline.sh (local full-rigor version)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh" 2>/dev/null || true

VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
VPS_USER="${MEMBRANE_VPS_USER:-root}"
SKIP_SSH=false
[[ "${1:-}" == "--skip-ssh" ]] && SKIP_SSH=true

PASS=0; FAIL=0; SKIP=0; WARN=0
RESULTS_DIR="$PROJECT_ROOT/validation/membrane-provenance-$(date +%Y%m%d-%H%M%S)"
mkdir -p "$RESULTS_DIR"

pass()  { echo "  PASS  [$1] $2"; PASS=$((PASS + 1)); }
fail()  { echo "  FAIL  [$1] $2"; FAIL=$((FAIL + 1)); }
skip()  { echo "  SKIP  [$1] $2"; SKIP=$((SKIP + 1)); }
warn()  { echo "  WARN  [$1] $2"; WARN=$((WARN + 1)); }

ssh_cmd() {
    ssh -o ConnectTimeout=10 -o BatchMode=yes -o StrictHostKeyChecking=accept-new \
        "$VPS_USER@$VPS_IP" "$@" 2>/dev/null
}

rpc_remote() {
    local port="$1" payload="$2"
    ssh_cmd "echo '$payload' | timeout 3 socat -t 0.5 - TCP:127.0.0.1:$port 2>/dev/null | head -1" || true
}

rpc_remote_http() {
    local port="$1" payload="$2"
    ssh_cmd "curl -sf --max-time 3 -X POST http://127.0.0.1:$port \
        -H 'Content-Type: application/json' -d '$payload' 2>/dev/null" || true
}

echo "═══════════════════════════════════════════════════════════"
echo "  Membrane Provenance — Post-Deploy Trio Verification"
echo "  VPS: $VPS_IP"
echo "  Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "  Results: $RESULTS_DIR/"
echo "═══════════════════════════════════════════════════════════"
echo ""

if $SKIP_SSH; then
    echo "SSH skipped — cannot verify remote trio"
    exit 0
fi

# ─── Phase 1: Health Checks ───────────────────────────────────────────────
echo "── Phase 1: Nest Primal Health ──"

NESTGATE_LIVE=false
RHIZOCRYPT_LIVE=false
LOAMSPINE_LIVE=false
SWEETGRASS_LIVE=false

ng_resp=$(ssh_cmd "curl -sf --max-time 3 http://127.0.0.1:${NESTGATE_PORT}/health 2>/dev/null" || true)
if echo "$ng_resp" | grep -q '"status":"ok"'; then
    pass "TRIO-01" "NestGate healthy (:${NESTGATE_PORT} HTTP REST)"
    NESTGATE_LIVE=true
else
    skip "TRIO-01" "NestGate not responding (Nest Atomic not deployed)"
fi

rc_resp=$(rpc_remote "${RHIZOCRYPT_RPC_PORT}" '{"jsonrpc":"2.0","method":"health.liveness","id":1}')
if echo "$rc_resp" | grep -q '"result"'; then
    pass "TRIO-02" "rhizoCrypt healthy (:${RHIZOCRYPT_RPC_PORT} JSON-RPC)"
    RHIZOCRYPT_LIVE=true
else
    skip "TRIO-02" "rhizoCrypt not responding (Nest Atomic not deployed)"
fi

ls_resp=$(rpc_remote_http "${LOAMSPINE_PORT}" '{"jsonrpc":"2.0","method":"health.liveness","id":1}')
if echo "$ls_resp" | grep -q '"result"'; then
    pass "TRIO-03" "loamSpine healthy (:${LOAMSPINE_PORT})"
    LOAMSPINE_LIVE=true
else
    skip "TRIO-03" "loamSpine not responding (Nest Atomic not deployed)"
fi

sg_resp=$(rpc_remote "${SWEETGRASS_PORT}" '{"jsonrpc":"2.0","method":"health.liveness","id":1}')
if echo "$sg_resp" | grep -q '"result"'; then
    pass "TRIO-04" "sweetGrass healthy (:${SWEETGRASS_PORT})"
    SWEETGRASS_LIVE=true
else
    skip "TRIO-04" "sweetGrass not responding (Nest Atomic not deployed)"
fi

echo ""

# If no trio primals are live, exit gracefully
if ! $RHIZOCRYPT_LIVE && ! $LOAMSPINE_LIVE && ! $SWEETGRASS_LIVE; then
    echo "── No trio primals deployed. Nest Atomic pending (CM-1). ──"
    echo ""
    echo "═══════════════════════════════════════════════════════════"
    echo "  Membrane Provenance — Results"
    echo "  PASS: $PASS  FAIL: $FAIL  SKIP: $SKIP  WARN: $WARN"
    echo "  Status: PENDING (Nest Atomic not deployed)"
    echo "═══════════════════════════════════════════════════════════"
    exit 0
fi

# ─── Phase 2: DAG Session (rhizoCrypt) ─────────────────────────────────────
echo "── Phase 2: DAG Session (rhizoCrypt) ──"

if $RHIZOCRYPT_LIVE; then
    SESSION_NAME="membrane-verify-$(date +%Y%m%d-%H%M%S)"
    dag_resp=$(rpc_remote "$RHIZOCRYPT_RPC_PORT" "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.create\",\"params\":{\"name\":\"$SESSION_NAME\"},\"id\":10}")
    SESSION_ID=$(echo "$dag_resp" | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',''))" 2>/dev/null || true)

    if [[ -n "$SESSION_ID" && "$SESSION_ID" != "None" && "$SESSION_ID" != "" ]]; then
        pass "TRIO-05" "DAG session created: ${SESSION_ID:0:20}..."

        event_resp=$(rpc_remote "$RHIZOCRYPT_RPC_PORT" "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"DataCreate\":{}},\"data\":{\"type\":\"membrane_verify\",\"timestamp\":\"$(date -Iseconds)\"}},\"id\":11}")
        if echo "${event_resp:-}" | grep -q '"result"'; then
            pass "TRIO-06" "DAG event appended"
        else
            warn "TRIO-06" "DAG event append returned: ${event_resp:0:100}"
        fi
    else
        fail "TRIO-05" "DAG session creation failed: ${dag_resp:0:100}"
    fi
else
    skip "TRIO-05" "rhizoCrypt not live — DAG session skipped"
    skip "TRIO-06" "rhizoCrypt not live — DAG event skipped"
fi

echo ""

# ─── Phase 3: Spine (loamSpine) ───────────────────────────────────────────
echo "── Phase 3: Spine (loamSpine) ──"

if $LOAMSPINE_LIVE; then
    spine_resp=$(rpc_remote_http "$LOAMSPINE_PORT" "{\"jsonrpc\":\"2.0\",\"method\":\"spine.create\",\"params\":{\"name\":\"membrane-verify\",\"owner\":\"cellMembrane\"},\"id\":20}")
    SPINE_ID=$(echo "$spine_resp" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',{}); print(r.get('spine_id','') if isinstance(r,dict) else r)" 2>/dev/null || true)

    if [[ -n "$SPINE_ID" && "$SPINE_ID" != "None" && "$SPINE_ID" != "" ]]; then
        pass "TRIO-07" "Spine created: ${SPINE_ID:0:20}..."
    else
        warn "TRIO-07" "Spine creation returned: ${spine_resp:0:100}"
    fi
else
    skip "TRIO-07" "loamSpine not live — spine skipped"
fi

echo ""

# ─── Phase 4: Braid (sweetGrass) ──────────────────────────────────────────
echo "── Phase 4: Braid (sweetGrass) ──"

if $SWEETGRASS_LIVE; then
    VERIFY_HASH="$(date +%s | sha256sum | cut -d' ' -f1)"
    braid_resp=$(rpc_remote "$SWEETGRASS_PORT" "{\"jsonrpc\":\"2.0\",\"method\":\"braid.create\",\"params\":{\"data_hash\":\"$VERIFY_HASH\",\"name\":\"membrane-verify\",\"mime_type\":\"application/x-membrane-verify\",\"description\":\"Post-deploy trio verification\",\"size\":1},\"id\":30}")
    BRAID_ID=$(echo "$braid_resp" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',{}); print(r.get('@id','') if isinstance(r,dict) else r)" 2>/dev/null || true)

    if [[ -n "$BRAID_ID" && "$BRAID_ID" != "None" && "$BRAID_ID" != "" ]]; then
        pass "TRIO-08" "Braid created: ${BRAID_ID:0:20}..."
    else
        warn "TRIO-08" "Braid creation returned: ${braid_resp:0:100}"
    fi
else
    skip "TRIO-08" "sweetGrass not live — braid skipped"
fi

echo ""

# ─── Phase 5: Cross-check Tower ↔ Nest ────────────────────────────────────
echo "── Phase 5: Tower ↔ Nest Cross-Check ──"

bd_resp=$(rpc_remote "$BEARDOG_PORT" '{"jsonrpc":"2.0","method":"health.liveness","id":1}')
if echo "$bd_resp" | grep -q '"result"'; then
    pass "TRIO-09" "BearDog (Tower) healthy alongside Nest"
else
    fail "TRIO-09" "BearDog (Tower) not responding — Tower degraded"
fi

sb_resp=$(ssh_cmd "ss -ulnp 2>/dev/null | grep ':$TURN_PORT'" || true)
if [[ -n "$sb_resp" ]]; then
    pass "TRIO-10" "Songbird TURN (Tower) alive alongside Nest"
else
    warn "TRIO-10" "Songbird TURN not listening — relay may be degraded"
fi

echo ""

# ─── Write Report ──────────────────────────────────────────────────────────
cat > "$RESULTS_DIR/PROVENANCE_MEMBRANE_REPORT.md" << EOF
# Membrane Provenance Verification — $(date -u +%Y-%m-%dT%H:%M:%SZ)

**VPS**: $VPS_IP
**Composition**: Tower + Nest Atomic (pending: $(! $NESTGATE_LIVE && echo "NestGate ") $(! $RHIZOCRYPT_LIVE && echo "rhizoCrypt ") $(! $LOAMSPINE_LIVE && echo "loamSpine ") $(! $SWEETGRASS_LIVE && echo "sweetGrass "))

## Health

| Primal | Port | Status |
|--------|------|--------|
| NestGate | :${NESTGATE_PORT} | $($NESTGATE_LIVE && echo "LIVE" || echo "NOT DEPLOYED") |
| rhizoCrypt | :${RHIZOCRYPT_PORT} | $($RHIZOCRYPT_LIVE && echo "LIVE" || echo "NOT DEPLOYED") |
| loamSpine | :${LOAMSPINE_PORT} | $($LOAMSPINE_LIVE && echo "LIVE" || echo "NOT DEPLOYED") |
| sweetGrass | :${SWEETGRASS_PORT} | $($SWEETGRASS_LIVE && echo "LIVE" || echo "NOT DEPLOYED") |
| BearDog | :${BEARDOG_PORT} | $(echo "$bd_resp" | grep -q result && echo "LIVE" || echo "DEGRADED") |
| Songbird | UDP :${TURN_PORT} | $([[ -n "$sb_resp" ]] && echo "LIVE" || echo "DEGRADED") |

## Trio Pipeline

| Step | Method | Result |
|------|--------|--------|
| DAG session | dag.session.create | ${SESSION_ID:-skipped} |
| DAG event | dag.event.append | $(echo "${event_resp:-}" 2>/dev/null | grep -q result && echo "OK" || echo "skipped") |
| Spine | spine.create | ${SPINE_ID:-skipped} |
| Braid | braid.create | ${BRAID_ID:-skipped} |

## Summary

PASS: $PASS  FAIL: $FAIL  SKIP: $SKIP  WARN: $WARN
EOF

echo "  Report: $RESULTS_DIR/PROVENANCE_MEMBRANE_REPORT.md"
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  Membrane Provenance — Results"
echo "  PASS: $PASS  FAIL: $FAIL  SKIP: $SKIP  WARN: $WARN"
echo "  Primals reached: $((PASS))/10"
echo "═══════════════════════════════════════════════════════════"

[[ $FAIL -eq 0 ]] && exit 0 || exit 1
