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
source "$SCRIPT_DIR/nucleus_config.sh"

PROJECT_ROOT="$NUCLEUS_PROJECT_ROOT"
PLASMIDBIN_DIR="${PLASMIDBIN_DIR}"
RUNTIME_DIR="${RUNTIME_DIR}"
FAMILY_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/biomeos/family"

COMPOSITION="node"
GATE=""
FAMILY_NAME="${NUCLEUS_FAMILY_NAME:-$(hostname -s)-sovereign}"
STOP=false
STATUS=false

BIND_ADDRESS="${NUCLEUS_BIND_ADDRESS}"

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
        node)  echo "beardog songbird toadstool barracuda coralreef skunkbat" ;;
        nest)  echo "beardog songbird nestgate rhizocrypt loamspine sweetgrass skunkbat" ;;
        full)
            # Discover all available primal binaries from plasmidBin.
            # Boot order: beardog first (security), songbird second (network),
            # then remaining primals alphabetically. This replaces hardcoded lists.
            local discovered=""
            local bin_dir="$PLASMIDBIN_DIR/primals"
            [[ -d "$bin_dir/x86_64-unknown-linux-musl" ]] && bin_dir="$bin_dir/x86_64-unknown-linux-musl"
            local boot_first="beardog songbird"
            local rest=""
            for bin in "$bin_dir"/*; do
                [[ -x "$bin" ]] || continue
                local name
                name=$(basename "$bin")
                [[ "$name" == "beardog" || "$name" == "songbird" ]] && continue
                rest="$rest $name"
            done
            echo "$boot_first$(echo "$rest" | tr ' ' '\n' | sort | tr '\n' ' ')"
            ;;
        *)     echo "ERROR: Unknown composition: $1" >&2; return 1 ;;
    esac
}

# ── Stop command ───────────────────────────────────────────────────────────

if $STOP; then
    echo "Stopping all primals..."
    local_bin_dir="$PLASMIDBIN_DIR/primals"
    [[ -d "$local_bin_dir/x86_64-unknown-linux-musl" ]] && local_bin_dir="$local_bin_dir/x86_64-unknown-linux-musl"
    for bin in "$local_bin_dir"/*; do
        [[ -x "$bin" ]] || continue
        p=$(basename "$bin")
        pkill -f "$PLASMIDBIN_DIR/primals/.*$p" 2>/dev/null || true
    done
    sleep 1
    echo "Done."
    exit 0
fi

# ── Status command ─────────────────────────────────────────────────────────

if $STATUS; then
    echo "=== NUCLEUS Status ==="
    local_bin_dir="$PLASMIDBIN_DIR/primals"
    [[ -d "$local_bin_dir/x86_64-unknown-linux-musl" ]] && local_bin_dir="$local_bin_dir/x86_64-unknown-linux-musl"
    running_count=0
    for bin in "$local_bin_dir"/*; do
        [[ -x "$bin" ]] || continue
        p=$(basename "$bin")
        pid=$(pgrep -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null | head -1) || true
        if [[ -n "$pid" ]]; then
            echo "  $p: PID $pid — RUNNING"
            running_count=$((running_count + 1))
        fi
    done
    echo "  Total: $running_count primal(s) running"
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

# ── Load gate config (if --gate specified) ────────────────────────────────
# Gate TOMLs override ports and composition. Minimal TOML parsing via grep.

GATE_FILE=""
if [[ -n "$GATE" ]] && [[ -f "$PROJECT_ROOT/gates/${GATE}.toml" ]]; then
    GATE_FILE="$PROJECT_ROOT/gates/${GATE}.toml"
    echo "  Loading gate config: $GATE_FILE"

    toml_val() { grep "^$1 " "$GATE_FILE" 2>/dev/null | head -1 | sed 's/.*= *//; s/"//g; s/ *$//'; }

    _port=$(toml_val "beardog");    [[ -n "$_port" ]] && BEARDOG_PORT="$_port"
    _port=$(toml_val "songbird");   [[ -n "$_port" ]] && SONGBIRD_PORT="$_port"
    _port=$(toml_val "squirrel");   [[ -n "$_port" ]] && SQUIRREL_PORT="$_port"
    _port=$(toml_val "toadstool");  [[ -n "$_port" ]] && TOADSTOOL_PORT="$_port"
    _port=$(toml_val "nestgate");   [[ -n "$_port" ]] && NESTGATE_PORT="$_port"
    _port=$(toml_val "rhizocrypt"); [[ -n "$_port" ]] && RHIZOCRYPT_PORT="$_port"
    _port=$(toml_val "loamspine");  [[ -n "$_port" ]] && LOAMSPINE_PORT="$_port"
    _port=$(toml_val "coralreef");  [[ -n "$_port" ]] && CORALREEF_PORT="$_port"
    _port=$(toml_val "barracuda");  [[ -n "$_port" ]] && BARRACUDA_PORT="$_port"
    _port=$(toml_val "sweetgrass"); [[ -n "$_port" ]] && SWEETGRASS_PORT="$_port"
    unset _port
fi

# ── Deployment readiness check ─────────────────────────────────────────────
# Mirrors primalSpring's validate_deployment_readiness():
#   1. Structure  — graph TOML exists and parses
#   2. Binary     — each primal binary is present and executable
#   3. Env        — required environment variables are set
#   4. Bonding    — bonding_policy consistency (BTSP requires BearDog)

graph_for_composition() {
    case "$1" in
        tower) echo "$PROJECT_ROOT/graphs/tower_atomic.toml" ;;
        node)  echo "$PROJECT_ROOT/graphs/node_atomic_compute.toml" ;;
        nest)  echo "$PROJECT_ROOT/graphs/nest_atomic.toml" ;;
        full)  echo "$PROJECT_ROOT/graphs/nucleus_complete.toml" ;;
        *)     echo "$PROJECT_ROOT/graphs/nucleus_complete.toml" ;;
    esac
}

GRAPH_FILE=$(graph_for_composition "$COMPOSITION")

READINESS_ISSUES=0

echo "=== Deployment Readiness ==="
echo "  Graph: $GRAPH_FILE"

# 1. Structure — graph file must exist
if [[ ! -f "$GRAPH_FILE" ]]; then
    echo "  [Structure] Graph file not found: $GRAPH_FILE"
    READINESS_ISSUES=$((READINESS_ISSUES + 1))
fi

# 2. Binary discovery
for p in $PRIMALS; do
    if [[ ! -x "$PLASMIDBIN_DIR/primals/$p" ]]; then
        echo "  [BinaryMissing] $p — run 'bash $PLASMIDBIN_DIR/fetch.sh --all' first"
        READINESS_ISSUES=$((READINESS_ISSUES + 1))
    fi
done

# 3. Environment checks
if echo "$PRIMALS" | grep -q beardog; then
    if [[ -z "${BEARDOG_FAMILY_SEED:-}" ]] && [[ ! -f "${FAMILY_DIR}/.beacon.seed" ]]; then
        echo "  [EnvMissing] BEARDOG_FAMILY_SEED not set and no .beacon.seed found"
        READINESS_ISSUES=$((READINESS_ISSUES + 1))
    fi
fi

if echo "$PRIMALS" | grep -q nestgate; then
    if [[ -z "${NESTGATE_JWT_SECRET:-}" ]]; then
        echo "  [EnvMissing] NESTGATE_JWT_SECRET not set (will auto-generate)"
    fi
fi

# 4. Bonding consistency — BTSP compositions require BearDog
if ! echo "$PRIMALS" | grep -q beardog; then
    echo "  [BondingInconsistent] BTSP required but beardog not in composition"
    READINESS_ISSUES=$((READINESS_ISSUES + 1))
fi

# 5. primalspring_guidestone structural validation (if available)
GUIDESTONE=$(command -v primalspring_guidestone 2>/dev/null || echo "")
if [[ -n "$GUIDESTONE" ]] && [[ -f "$GRAPH_FILE" ]]; then
    echo "  Running primalspring_guidestone against $GRAPH_FILE..."
    if ! "$GUIDESTONE" validate --graph "$GRAPH_FILE"; then
        echo "  [Structure] guidestone validation reported issues"
        READINESS_ISSUES=$((READINESS_ISSUES + 1))
    fi
fi

if [[ $READINESS_ISSUES -gt 0 ]]; then
    echo ""
    echo "ERROR: Deployment readiness check found $READINESS_ISSUES issue(s)."
    echo "  Fix issues above or set NUCLEUS_SKIP_READINESS=1 to override."
    if [[ "${NUCLEUS_SKIP_READINESS:-0}" != "1" ]]; then
        exit 1
    fi
    echo "  NUCLEUS_SKIP_READINESS=1 — proceeding despite issues."
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

# Clean stale sockets from previous runs
for sock in "$RUNTIME_DIR"/biomeos/*.sock; do
    [[ -S "$sock" ]] && rm -f "$sock" 2>/dev/null
done

export FAMILY_ID="$FAMILY_ID"
export NODE_ID="$NODE_ID"
export ECOPRIMALS_PLASMID_BIN="$PLASMIDBIN_DIR"
export XDG_RUNTIME_DIR="$RUNTIME_DIR"

# MethodGate auth mode — propagate to all primals (Phase 60)
export BEARDOG_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export SONGBIRD_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export TOADSTOOL_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export NESTGATE_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export RHIZOCRYPT_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export LOAMSPINE_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export SWEETGRASS_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export SQUIRREL_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export SKUNKBAT_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export BIOMEOS_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export PETALTONGUE_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export CORALREEF_AUTH_MODE="$NUCLEUS_AUTH_MODE"
export BARRACUDA_AUTH_MODE="$NUCLEUS_AUTH_MODE"

BEARDOG_SOCKET="$RUNTIME_DIR/biomeos/beardog-$FAMILY_ID.sock"

for p in $PRIMALS; do
    case "$p" in
        beardog)
            echo "  Starting beardog (UDS + TCP $BEARDOG_PORT)..."
            export BEARDOG_FAMILY_SEED="$BEACON_SEED"
            nohup "$PLASMIDBIN_DIR/primals/beardog" server \
                --socket "$BEARDOG_SOCKET" \
                --family-id "$FAMILY_ID" \
                --listen "$BIND_ADDRESS:$BEARDOG_PORT" \
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
                --bind "$BIND_ADDRESS:$BARRACUDA_PORT" \
                > /tmp/barracuda.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        coralreef)
            echo "  Starting coralreef (RPC $CORALREEF_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/coralreef" server \
                --rpc-bind "$BIND_ADDRESS:$CORALREEF_PORT" \
                > /tmp/coralreef.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        nestgate)
            echo "  Starting nestgate (UDS + TCP $NESTGATE_PORT)..."
            export NESTGATE_FAMILY_ID="$FAMILY_ID"
            export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(head -c 32 /dev/urandom | base64)}"
            nohup "$PLASMIDBIN_DIR/primals/nestgate" daemon \
                --socket-only \
                --port "$NESTGATE_PORT" \
                --bind "$BIND_ADDRESS" \
                > /tmp/nestgate.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        rhizocrypt)
            echo "  Starting rhizocrypt (TCP $RHIZOCRYPT_PORT, JSON-RPC $((RHIZOCRYPT_PORT+1)))..."
            export FAMILY_SEED="$BEACON_SEED"
            nohup "$PLASMIDBIN_DIR/primals/rhizocrypt" server \
                --port "$RHIZOCRYPT_PORT" \
                --host "$BIND_ADDRESS" \
                > /tmp/rhizocrypt.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        loamspine)
            echo "  Starting loamspine (TCP $LOAMSPINE_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/loamspine" server \
                --port "$LOAMSPINE_PORT" \
                --bind-address "$BIND_ADDRESS" \
                > /tmp/loamspine.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        sweetgrass)
            echo "  Starting sweetgrass (TCP $SWEETGRASS_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/sweetgrass" server \
                --port "$SWEETGRASS_PORT" \
                --http-address "$BIND_ADDRESS:$((SWEETGRASS_PORT + 1))" \
                > /tmp/sweetgrass.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        squirrel)
            echo "  Starting squirrel (TCP $SQUIRREL_PORT)..."
            export CAPABILITY_REGISTRY_SOCKET="$RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock"
            nohup "$PLASMIDBIN_DIR/primals/squirrel" server \
                --port "$SQUIRREL_PORT" \
                --bind "$BIND_ADDRESS" \
                > /tmp/squirrel.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        skunkbat)
            echo "  Starting skunkbat (TCP $SKUNKBAT_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/skunkbat" server \
                --port "$SKUNKBAT_PORT" \
                > /tmp/skunkbat.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        biomeos)
            echo "  Starting biomeos neural-api (TCP $BIOMEOS_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/biomeos" neural-api \
                --port "$BIOMEOS_PORT" \
                --family-id "$FAMILY_ID" \
                --btsp-optional \
                > /tmp/biomeos.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        petaltongue)
            echo "  Starting petaltongue server (TCP $PETALTONGUE_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/petaltongue" server \
                --port "$PETALTONGUE_PORT" \
                > /tmp/petaltongue.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        *)
            # Unknown primal — attempt standard server --port pattern
            local primal_bin="$PLASMIDBIN_DIR/primals/$p"
            if [[ -x "$primal_bin" ]]; then
                echo "  Starting $p (discovered, attempting server --port)..."
                nohup "$primal_bin" server --port 0 \
                    > "/tmp/$p.log" 2>&1 &
                echo "    PID: $! (port auto-assigned — check /tmp/$p.log)"
                sleep 1
            else
                echo "  SKIP $p — no binary found"
            fi
            ;;
    esac
done

echo ""

# ── Phase 4: Verify ──────────────────────────────────────────────────────

echo "=== Phase 4: Verify ==="

port_for_primal() {
    case "$1" in
        beardog)     echo "$BEARDOG_PORT" ;;
        songbird)    echo "$SONGBIRD_PORT" ;;
        toadstool)   echo "$TOADSTOOL_PORT" ;;
        barracuda)   echo "$BARRACUDA_PORT" ;;
        coralreef)   echo "$CORALREEF_PORT" ;;
        nestgate)    echo "$NESTGATE_PORT" ;;
        rhizocrypt)  echo "$RHIZOCRYPT_PORT" ;;
        loamspine)   echo "$LOAMSPINE_PORT" ;;
        sweetgrass)  echo "$SWEETGRASS_PORT" ;;
        squirrel)    echo "$SQUIRREL_PORT" ;;
        skunkbat)    echo "$SKUNKBAT_PORT" ;;
        biomeos)     echo "$BIOMEOS_PORT" ;;
        petaltongue) echo "$PETALTONGUE_PORT" ;;
        *)           echo "" ;;
    esac
}

rpc_health_check() {
    local port="$1"
    curl -sf --max-time 3 "http://127.0.0.1:$port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","id":1}' 2>/dev/null
}

ALL_OK=true
for p in $PRIMALS; do
    pid=$(pgrep -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null | head -1) || true
    if [[ -z "$pid" ]]; then
        echo "  $p: NOT RUNNING — check /tmp/$p.log"
        ALL_OK=false
        continue
    fi

    port=$(port_for_primal "$p")
    if [[ -n "$port" ]]; then
        resp=$(rpc_health_check "$port") || resp=""
        if [[ -n "$resp" ]]; then
            echo "  $p: PID $pid, TCP $port — HEALTHY"
        else
            echo "  $p: PID $pid, TCP $port — running (health probe pending)"
        fi
    else
        echo "  $p: PID $pid — running"
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
echo "  Family ID:   $FAMILY_ID"
echo "  Node ID:     $NODE_ID"
echo "  Graph:       $GRAPH_FILE"
echo "  Composition: $COMPOSITION ($(echo $PRIMALS | wc -w | tr -d ' ') primals)"
echo "  Logs:        /tmp/{$(echo $PRIMALS | tr ' ' ',')}.log"
echo ""
echo "  To stop:   bash $0 --stop"
echo "  To check:  bash $0 --status"
