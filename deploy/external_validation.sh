#!/usr/bin/env bash
# External Validation Pipeline — Phase 2a
#
# Tests the full external access path (cell membrane model):
#   Browser → lab.primals.eco → cloudflared (membrane) → JupyterHub → primals → provenance
#   primals.eco is served externally via GitHub Pages CDN (extracellular layer)
#
# This script can be run in two modes:
#   --local     Test localhost paths only (no tunnel required)
#   --tunnel    Start cloudflared tunnel and test external path
#
# Usage:
#   bash external_validation.sh --local
#   bash external_validation.sh --tunnel --hostname lab.primals.eco
#
# Requires: curl, python3, nc, cloudflared (for --tunnel mode)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

PROJECT_ROOT="$NUCLEUS_PROJECT_ROOT"
CLOUDFLARED="${CLOUDFLARED:-$(command -v cloudflared 2>/dev/null || echo "$HOME/bin/cloudflared")}"
JUPYTERHUB_URL="http://127.0.0.1:$JUPYTERHUB_PORT"
TUNNEL_HOSTNAME="${TUNNEL_HOSTNAME:-}"
MODE="local"
RESULTS_DIR="$PROJECT_ROOT/validation/external-$(date +%Y%m%d-%H%M%S)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --local)     MODE="local"; shift ;;
        --tunnel)    MODE="tunnel"; shift ;;
        --hostname)  TUNNEL_HOSTNAME="$2"; shift 2 ;;
        --results)   RESULTS_DIR="$2"; shift 2 ;;
        *)           echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$RESULTS_DIR"

log() { echo "[$(date +%H:%M:%S)] $*"; }
pass() { log "  [PASS] $*"; }
fail() { log "  [FAIL] $*"; }

PASS_COUNT=0
FAIL_COUNT=0

check() {
    local name="$1" result="$2"
    if [[ "$result" == "pass" ]]; then
        pass "$name"
        PASS_COUNT=$((PASS_COUNT + 1))
    else
        fail "$name"
        FAIL_COUNT=$((FAIL_COUNT + 1))
    fi
}

log "═══════════════════════════════════════════════════════════"
log "  External Validation Pipeline — Phase 2a"
log "  Mode: $MODE"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

# ══════════════════════════════════════════════════════════════
# STAGE 1: Prerequisite checks
# ══════════════════════════════════════════════════════════════
log ""
log "── Stage 1: Prerequisites ──"

if curl -sf "$JUPYTERHUB_URL/hub/api/" -o /dev/null 2>/dev/null; then
    check "JupyterHub reachable at $JUPYTERHUB_URL" "pass"
    HUB_VERSION=$(curl -sf "$JUPYTERHUB_URL/hub/api/" 2>/dev/null | python3 -c "import sys,json; print(json.load(sys.stdin).get('version','?'))" 2>/dev/null || echo "?")
    log "       JupyterHub version: $HUB_VERSION"
else
    check "JupyterHub reachable at $JUPYTERHUB_URL" "fail"
    log "       Start JupyterHub: cd ~/jupyterhub && bash start.sh"
    exit 1
fi

PRIMAL_COUNT=0
for pair in "beardog:$BEARDOG_PORT" "songbird:$SONGBIRD_PORT" "toadstool:$TOADSTOOL_PORT" "nestgate:$NESTGATE_PORT" "rhizocrypt:$RHIZOCRYPT_RPC_PORT" "loamspine:$LOAMSPINE_PORT" "sweetgrass:$SWEETGRASS_PORT" "squirrel:$SQUIRREL_PORT" "barracuda:$BARRACUDA_PORT" "coralreef:$CORALREEF_PORT" "biomeos:$BIOMEOS_PORT" "petaltongue:$PETALTONGUE_PORT" "skunkbat:$SKUNKBAT_PORT"; do
    name="${pair%%:*}"
    port="${pair#*:}"
    if [[ "$name" == "songbird" ]]; then
        resp=$(curl -sf --max-time 2 "http://127.0.0.1:$port/health" 2>/dev/null) || resp=""
        [[ "$resp" == "OK" ]] && PRIMAL_COUNT=$((PRIMAL_COUNT + 1))
    else
        resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","id":1}\n' | nc -w 2 127.0.0.1 "$port" 2>/dev/null) || resp=""
        echo "$resp" | grep -q '"result"' 2>/dev/null && PRIMAL_COUNT=$((PRIMAL_COUNT + 1))
    fi
done

check "Primal composition healthy ($PRIMAL_COUNT/13)" "$( [[ $PRIMAL_COUNT -ge 7 ]] && echo pass || echo fail )"

if [[ "$MODE" == "tunnel" ]]; then
    if [[ -x "$CLOUDFLARED" ]]; then
        CF_VERSION=$("$CLOUDFLARED" --version 2>&1 | head -1)
        check "cloudflared available: $CF_VERSION" "pass"
    else
        check "cloudflared available" "fail"
        log "       Install: curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o ~/bin/cloudflared && chmod +x ~/bin/cloudflared"
        exit 1
    fi
fi

# ══════════════════════════════════════════════════════════════
# STAGE 2: JupyterHub endpoint validation
# ══════════════════════════════════════════════════════════════
log ""
log "── Stage 2: JupyterHub Endpoint Validation ──"

LOGIN_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" "$JUPYTERHUB_URL/hub/login" 2>/dev/null)
check "GET /hub/login returns 200 (got $LOGIN_STATUS)" "$( [[ "$LOGIN_STATUS" == "200" ]] && echo pass || echo fail )"

API_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" "$JUPYTERHUB_URL/hub/api/" 2>/dev/null)
check "GET /hub/api/ returns 200 (got $API_STATUS)" "$( [[ "$API_STATUS" == "200" ]] && echo pass || echo fail )"

HEALTH_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" "$JUPYTERHUB_URL/hub/health" 2>/dev/null)
check "GET /hub/health returns 200 (got $HEALTH_STATUS)" "$( [[ "$HEALTH_STATUS" == "200" ]] && echo pass || echo fail )"

# ══════════════════════════════════════════════════════════════
# STAGE 3: Primal API validation through localhost
# ══════════════════════════════════════════════════════════════
log ""
log "── Stage 3: Primal API Validation ──"

CAPS_RESP=$(printf '{"jsonrpc":"2.0","method":"capabilities.list","id":1}\n' | nc -w 3 127.0.0.1 "$TOADSTOOL_PORT" 2>/dev/null) || CAPS_RESP=""
if echo "$CAPS_RESP" | grep -q '"toadstool"' 2>/dev/null; then
    check "ToadStool capabilities.list" "pass"
    TOAD_VERSION=$(echo "$CAPS_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['version'])" 2>/dev/null || echo "?")
    log "       ToadStool version: $TOAD_VERSION"
else
    check "ToadStool capabilities.list" "fail"
fi

DAG_RESP=$(printf '{"jsonrpc":"2.0","method":"dag.session.list","id":1}\n' | nc -w 3 127.0.0.1 "$RHIZOCRYPT_RPC_PORT" 2>/dev/null) || DAG_RESP=""
if echo "$DAG_RESP" | grep -q '"result"' 2>/dev/null; then
    check "rhizoCrypt DAG session.list" "pass"
    SESSION_COUNT=$(echo "$DAG_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(len(r) if isinstance(r,list) else r)" 2>/dev/null || echo "?")
    log "       Active sessions: $SESSION_COUNT"
else
    check "rhizoCrypt DAG session.list" "fail"
fi

SPINE_RESP=$(curl -sf --max-time 3 "http://127.0.0.1:${LOAMSPINE_PORT}" \
    -X POST -H 'Content-Type: application/json' \
    -d '{"jsonrpc":"2.0","method":"capabilities.list","id":1}' 2>/dev/null) || SPINE_RESP=""
if echo "$SPINE_RESP" | grep -q '"result"' 2>/dev/null; then
    check "loamSpine capabilities.list" "pass"
    CAP_COUNT=$(echo "$SPINE_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; c=r.get('capabilities',r); print(len(c) if isinstance(c,list) else c)" 2>/dev/null || echo "?")
    log "       Capabilities: $CAP_COUNT"
else
    check "loamSpine capabilities.list" "fail"
fi

# ══════════════════════════════════════════════════════════════
# STAGE 4: Provenance pipeline smoke test
# ══════════════════════════════════════════════════════════════
log ""
log "── Stage 4: Provenance Pipeline Smoke Test ──"

LATEST_MANIFEST=""
for d in "$PROJECT_ROOT/validation"/provenance-run-* "$PROJECT_ROOT/validation/archive"/provenance-run-*; do
    [[ -f "$d/PROVENANCE_MANIFEST.md" ]] && LATEST_MANIFEST="$d/PROVENANCE_MANIFEST.md"
done

if [[ -n "$LATEST_MANIFEST" ]]; then
    check "Provenance manifest exists" "pass"
    log "       Location: $LATEST_MANIFEST"

    MERKLE=$(python3 -c "
import re
text = open('$LATEST_MANIFEST').read()
m = re.search(r'Merkle Root.*\x60([0-9a-f]{32,})\x60', text)
print(m.group(1) if m else '')
" 2>/dev/null)
    if [[ -n "$MERKLE" ]]; then
        check "Merkle root present: ${MERKLE:0:32}..." "pass"
    else
        check "Merkle root present" "fail"
    fi

    BRAID=$(python3 -c "
import re
text = open('$LATEST_MANIFEST').read()
m = re.search(r'SweetGrass Braid.*\x60(urn:braid:[0-9a-f]+)\x60', text)
print(m.group(1) if m else '')
" 2>/dev/null)
    if [[ -n "$BRAID" ]]; then
        check "Braid URN present: ${BRAID:0:48}..." "pass"
    else
        check "Braid URN present" "fail"
    fi
else
    check "Provenance manifest exists" "fail"
    log "       Run: cd deploy && bash provenance_pipeline.sh"
fi

LATEST_BRAID=""
for d in "$PROJECT_ROOT/validation"/provenance-run-* "$PROJECT_ROOT/validation/archive"/provenance-run-*; do
    [[ -f "$d/braid.json" ]] && LATEST_BRAID="$d/braid.json"
done

if [[ -n "$LATEST_BRAID" ]]; then
    HAS_WITNESS=$(python3 -c "import json; b=json.load(open('$LATEST_BRAID')); r=b.get('result',b); print('yes' if r.get('witness',{}).get('algorithm')=='ed25519' else 'no')" 2>/dev/null || echo "no")
    check "Braid has ed25519 witness" "$( [[ "$HAS_WITNESS" == "yes" ]] && echo pass || echo fail )"
fi

# ══════════════════════════════════════════════════════════════
# STAGE 5: Tunnel validation (if --tunnel mode)
# ══════════════════════════════════════════════════════════════
if [[ "$MODE" == "tunnel" ]]; then
    log ""
    log "── Stage 5: Cloudflare Tunnel ──"

    if [[ -z "$TUNNEL_HOSTNAME" ]]; then
        log "  Starting quick tunnel (no Cloudflare account needed)..."
        log "  This creates a temporary public URL forwarding to JupyterHub."

        TUNNEL_LOG="$RESULTS_DIR/tunnel.log"
        "$CLOUDFLARED" tunnel --url "$JUPYTERHUB_URL" > "$TUNNEL_LOG" 2>&1 &
        TUNNEL_PID=$!
        log "       Tunnel PID: $TUNNEL_PID"

        sleep 5

        TUNNEL_URL=$(grep -oP 'https://[a-z0-9-]+\.trycloudflare\.com' "$TUNNEL_LOG" 2>/dev/null | head -1)
        if [[ -n "$TUNNEL_URL" ]]; then
            check "Quick tunnel established" "pass"
            log "       URL: $TUNNEL_URL"

            EXT_STATUS=$(curl -sf -o /dev/null -w "%{http_code}" -L "$TUNNEL_URL/hub/api/" --max-time 15 2>/dev/null)
            check "External JupyterHub API via tunnel (HTTP $EXT_STATUS)" "$( [[ "$EXT_STATUS" == "200" ]] && echo pass || echo fail )"

            EXT_LOGIN=$(curl -sf -o /dev/null -w "%{http_code}" -L "$TUNNEL_URL/hub/login" --max-time 15 2>/dev/null)
            check "External /hub/login via tunnel (HTTP $EXT_LOGIN)" "$( [[ "$EXT_LOGIN" == "200" ]] && echo pass || echo fail )"

            EXT_LATENCY=$(curl -sf -o /dev/null -w "%{time_total}" -L "$TUNNEL_URL/hub/api/" --max-time 15 2>/dev/null || echo "timeout")
            log "       Tunnel latency (API round-trip): ${EXT_LATENCY}s"
            echo "$EXT_LATENCY" > "$RESULTS_DIR/tunnel_latency.txt"

            log ""
            log "  Tunnel URL for ABG testing: $TUNNEL_URL"
            log "  Login with gate system credentials."
            log ""
            log "  Keeping tunnel alive for manual testing..."
            log "  Press Ctrl+C or kill $TUNNEL_PID to stop."

            echo "$TUNNEL_URL" > "$RESULTS_DIR/tunnel_url.txt"
        else
            check "Quick tunnel established" "fail"
            log "       Check $TUNNEL_LOG for errors"
            kill $TUNNEL_PID 2>/dev/null || true
        fi
    else
        log "  Named tunnel to $TUNNEL_HOSTNAME — requires Cloudflare account setup."
        log "  Use: cloudflared tunnel create nucleus-compute"
        log "       cloudflared tunnel route dns nucleus-compute $TUNNEL_HOSTNAME"
        log "  Then run: cloudflared tunnel run --url $JUPYTERHUB_URL nucleus-compute"
        check "Named tunnel configuration" "fail"
        log "       (Not yet configured — see specs/TUNNEL_EVOLUTION.md Step 2a)"
    fi
fi

# ══════════════════════════════════════════════════════════════
# RESULTS
# ══════════════════════════════════════════════════════════════
log ""
log "═══════════════════════════════════════════════════════════"
log "  External Validation Results"
log "  Passed: $PASS_COUNT"
log "  Failed: $FAIL_COUNT"
log "  Mode:   $MODE"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

cat > "$RESULTS_DIR/RESULTS.md" << EOF
# External Validation — $(date -Iseconds)

**Mode**: $MODE
**JupyterHub**: $JUPYTERHUB_URL (v$HUB_VERSION)
**Primals**: $PRIMAL_COUNT/13 healthy
**Passed**: $PASS_COUNT checks
**Failed**: $FAIL_COUNT checks

## Checks

$(if [[ $FAIL_COUNT -eq 0 ]]; then echo "All checks passed."; else echo "$FAIL_COUNT check(s) failed — see log output above."; fi)

## Environment

- Gate: $(uname -n) ($(uname -m))
- Kernel: $(uname -r)
- Date: $(date -Iseconds)
$(if [[ "$MODE" == "tunnel" ]] && [[ -f "$RESULTS_DIR/tunnel_url.txt" ]]; then echo "- Tunnel URL: $(cat "$RESULTS_DIR/tunnel_url.txt")"; fi)
$(if [[ "$MODE" == "tunnel" ]] && [[ -f "$RESULTS_DIR/tunnel_latency.txt" ]]; then echo "- Tunnel latency: $(cat "$RESULTS_DIR/tunnel_latency.txt")s"; fi)
EOF

log "  Results written to $RESULTS_DIR/RESULTS.md"

exit $FAIL_COUNT
