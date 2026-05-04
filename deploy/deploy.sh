#!/usr/bin/env bash
# projectNUCLEUS/deploy/deploy.sh — Deploy a NUCLEUS composition to a gate
#
# Wraps plasmidBin's bootstrap with lessons learned from live deployment:
#   - Correct CLI args for each primal (toadstool --port, coralreef --rpc-bind)
#   - Family seed initialization via seed_workflow.sh
#   - Health verification after startup
#   - Node Atomic includes barraCuda + coralReef (not just Tower + ToadStool)
#
# Usage:
#   bash deploy.sh --composition node --gate mygate
#   bash deploy.sh --composition tower --gate nuc-intake
#   bash deploy.sh --stop                  # stop all primals
#   bash deploy.sh --status                # check running primals
#
# Prerequisites:
#   - plasmidBin cloned with binaries fetched (fetch.sh --all)
#   - Set PLASMIDBIN_DIR if not at default location

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

PLASMIDBIN_DIR="${PLASMIDBIN_DIR:-$(cd "$PROJECT_ROOT/../../infra/plasmidBin" 2>/dev/null && pwd)}"
RUNTIME_DIR="/tmp/biomeos"
FAMILY_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/biomeos/family"

COMPOSITION="node"
GATE=""
FAMILY_NAME="${NUCLEUS_FAMILY_NAME:-$(hostname -s)-sovereign}"
STOP=false
STATUS=false

# Standard ports (from plasmidBin/ports.env)
BEARDOG_PORT=9100
SONGBIRD_PORT=9200
NESTGATE_PORT=9300
TOADSTOOL_PORT=9400
BARRACUDA_PORT=9500
CORALREEF_PORT=9730

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --composition NAME   tower|node|nest|full (default: node)"
    echo "  --gate NAME          Gate name (matches gates/<name>.toml)"
    echo "  --family-name NAME   Family name for seed init (default: \$(hostname)-sovereign)"
    echo "  --plasmidbin DIR     Path to plasmidBin (default: auto-detect)"
    echo "  --stop               Stop all running primals"
    echo "  --status             Show status of running primals"
    echo "  --help               Show this help"
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --composition)  COMPOSITION="$2"; shift 2 ;;
        --gate)         GATE="$2"; shift 2 ;;
        --family-name)  FAMILY_NAME="$2"; shift 2 ;;
        --plasmidbin)   PLASMIDBIN_DIR="$2"; shift 2 ;;
        --stop)         STOP=true; shift ;;
        --status)       STATUS=true; shift ;;
        --help)         usage; exit 0 ;;
        -*)             echo "Unknown option: $1"; usage; exit 1 ;;
        *)              echo "Unknown argument: $1"; usage; exit 1 ;;
    esac
done

primals_for_composition() {
    case "$1" in
        tower) echo "beardog songbird" ;;
        node)  echo "beardog songbird toadstool barracuda coralreef" ;;
        nest)  echo "beardog songbird nestgate" ;;
        full)  echo "beardog songbird toadstool barracuda coralreef nestgate" ;;
        *)     echo "ERROR: Unknown composition: $1" >&2; return 1 ;;
    esac
}

# ── Stop command ───────────────────────────────────────────────────────────

if $STOP; then
    echo "Stopping all primals..."
    for p in beardog songbird toadstool barracuda coralreef nestgate squirrel biomeos; do
        pkill -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null || true
    done
    sleep 1
    echo "Done."
    exit 0
fi

# ── Status command ─────────────────────────────────────────────────────────

if $STATUS; then
    echo "=== NUCLEUS Status ==="
    for p in beardog songbird toadstool barracuda coralreef nestgate squirrel biomeos; do
        pid=$(pgrep -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null | head -1) || true
        if [[ -n "$pid" ]]; then
            echo "  $p: PID $pid — RUNNING"
        fi
    done
    running=$(pgrep -cf "$PLASMIDBIN_DIR/primals/" 2>/dev/null) || running=0
    echo "  Total: $running primal(s) running"
    exit 0
fi

# ── Validate prerequisites ─────────────────────────────────────────────────

if [[ ! -d "$PLASMIDBIN_DIR" ]]; then
    echo "ERROR: plasmidBin not found at $PLASMIDBIN_DIR"
    echo "  Set PLASMIDBIN_DIR or use --plasmidbin"
    exit 1
fi

PRIMALS=$(primals_for_composition "$COMPOSITION")
echo ""
echo "╔══════════════════════════════════════════════╗"
echo "║  projectNUCLEUS — Deploy $COMPOSITION"
echo "╚══════════════════════════════════════════════╝"
echo ""
echo "  Gate:        ${GATE:-$(hostname -s)}"
echo "  Composition: $COMPOSITION ($PRIMALS)"
echo "  plasmidBin:  $PLASMIDBIN_DIR"
echo "  Family:      $FAMILY_NAME"
echo ""

# Check binaries exist
MISSING=0
for p in $PRIMALS; do
    if [[ ! -x "$PLASMIDBIN_DIR/primals/$p" ]]; then
        echo "  MISSING: $p — run 'bash $PLASMIDBIN_DIR/fetch.sh --all' first"
        MISSING=$((MISSING + 1))
    fi
done
if [[ $MISSING -gt 0 ]]; then
    echo "ERROR: $MISSING binaries missing."
    exit 1
fi

# ── Phase 1: Family seed ──────────────────────────────────────────────────

echo "=== Phase 1: Family seed ==="

FAMILY_ID_FILE="$FAMILY_DIR/family_id"
if [[ -f "$FAMILY_ID_FILE" ]]; then
    FAMILY_ID=$(cat "$FAMILY_ID_FILE")
    echo "  Existing family: $FAMILY_ID"
else
    echo "  Initializing new family: $FAMILY_NAME"
    bash "$PLASMIDBIN_DIR/seed_workflow.sh" init --family-name "$FAMILY_NAME"
    FAMILY_ID=$(cat "$FAMILY_ID_FILE")
    echo "  Family ID: $FAMILY_ID"
fi

NODE_ID="${GATE:-$(hostname -s)}"
LINEAGE_FILE="$FAMILY_DIR/nodes/${NODE_ID}.lineage.seed"
if [[ ! -f "$LINEAGE_FILE" ]]; then
    echo "  Adding node: $NODE_ID"
    bash "$PLASMIDBIN_DIR/seed_workflow.sh" add-node --node-id "$NODE_ID"
fi

BEACON_SEED="$FAMILY_DIR/.beacon.seed"
echo "  Beacon seed: $BEACON_SEED"
echo ""

# ── Phase 2: Stop existing primals ────────────────────────────────────────

echo "=== Phase 2: Clean slate ==="
for p in $PRIMALS; do
    pkill -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null || true
done
sleep 1
echo "  Previous instances stopped."
echo ""

# ── Phase 3: Start primals ────────────────────────────────────────────────

echo "=== Phase 3: Start primals ==="

mkdir -p "$RUNTIME_DIR/biomeos"

export FAMILY_ID="$FAMILY_ID"
export NODE_ID="$NODE_ID"
export ECOPRIMALS_PLASMID_BIN="$PLASMIDBIN_DIR"
export XDG_RUNTIME_DIR="$RUNTIME_DIR"

BEARDOG_SOCKET="$RUNTIME_DIR/biomeos/beardog-$FAMILY_ID.sock"

for p in $PRIMALS; do
    case "$p" in
        beardog)
            echo "  Starting beardog (UDS + TCP $BEARDOG_PORT)..."
            export BEARDOG_FAMILY_SEED="$BEACON_SEED"
            nohup "$PLASMIDBIN_DIR/primals/beardog" server \
                --socket "$BEARDOG_SOCKET" \
                --family-id "$FAMILY_ID" \
                --listen "0.0.0.0:$BEARDOG_PORT" \
                > /tmp/beardog.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        songbird)
            echo "  Starting songbird (HTTP $SONGBIRD_PORT)..."
            export BEARDOG_SOCKET="$BEARDOG_SOCKET"
            export BEARDOG_MODE=direct
            export SONGBIRD_SECURITY_PROVIDER=beardog
            nohup "$PLASMIDBIN_DIR/primals/songbird" server \
                --port "$SONGBIRD_PORT" \
                --socket "$RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock" \
                > /tmp/songbird.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        toadstool)
            echo "  Starting toadstool (TCP $TOADSTOOL_PORT)..."
            export TOADSTOOL_FAMILY_ID="$FAMILY_ID"
            export TOADSTOOL_NODE_ID="$NODE_ID"
            export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1
            nohup "$PLASMIDBIN_DIR/primals/toadstool" server \
                --port "$TOADSTOOL_PORT" \
                --family-id "$FAMILY_ID" \
                > /tmp/toadstool.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        barracuda)
            echo "  Starting barracuda (TCP $BARRACUDA_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/barracuda" server \
                --port "$BARRACUDA_PORT" \
                > /tmp/barracuda.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        coralreef)
            echo "  Starting coralreef (RPC $CORALREEF_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/coralreef" server \
                --rpc-bind "127.0.0.1:$CORALREEF_PORT" \
                > /tmp/coralreef.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        nestgate)
            echo "  Starting nestgate (UDS)..."
            export NESTGATE_FAMILY_ID="$FAMILY_ID"
            export NESTGATE_JWT_SECRET="projectnucleus-$NODE_ID-$FAMILY_ID"
            nohup "$PLASMIDBIN_DIR/primals/nestgate" daemon \
                --socket-only --dev \
                > /tmp/nestgate.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;
    esac
done

echo ""

# ── Phase 4: Verify ──────────────────────────────────────────────────────

echo "=== Phase 4: Verify ==="

ALL_OK=true
for p in $PRIMALS; do
    pid=$(pgrep -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null | head -1) || true
    if [[ -n "$pid" ]]; then
        echo "  $p: PID $pid — OK"
    else
        echo "  $p: NOT RUNNING — check /tmp/$p.log"
        ALL_OK=false
    fi
done

echo ""

if $ALL_OK; then
    echo "╔══════════════════════════════════════════════╗"
    echo "║  All primals running. Gate is live.          ║"
    echo "╚══════════════════════════════════════════════╝"
else
    echo "WARNING: Some primals failed to start. Check logs in /tmp/"
fi

echo ""
echo "  Family ID: $FAMILY_ID"
echo "  Node ID:   $NODE_ID"
echo "  Logs:      /tmp/{$(echo $PRIMALS | tr ' ' ',')}.log"
echo ""
echo "  To stop:   bash $0 --stop"
echo "  To check:  bash $0 --status"
echo ""

# Quick toadstool dispatch test (if toadstool is in the composition)
if echo "$PRIMALS" | grep -q toadstool; then
    echo "  Testing toadstool dispatch..."
    RESPONSE=$(curl -sf --max-time 5 "http://127.0.0.1:$TOADSTOOL_PORT" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"system.health","id":1}' 2>/dev/null) || RESPONSE=""
    if [[ -n "$RESPONSE" ]]; then
        echo "  toadstool JSON-RPC: responding"
    else
        echo "  toadstool JSON-RPC: no response (may still be initializing)"
    fi
fi
