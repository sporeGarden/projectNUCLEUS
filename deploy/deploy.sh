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
GRAPH_DEPLOY=false

BIND_ADDRESS="${NUCLEUS_BIND_ADDRESS}"

usage() {
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --composition NAME   tower|node|nest|full (default: node)"
    echo "  --gate NAME          Gate name (matches gates/<name>.toml)"
    echo "  --family-name NAME   Family name for seed init (default: \$(hostname)-sovereign)"
    echo "  --plasmidbin DIR     Path to plasmidBin (default: auto-detect)"
    echo "  --graph-deploy       Use graph-driven deploy (composition.deploy pattern)"
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
        --graph-deploy) GRAPH_DEPLOY=true; shift ;;
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

if $GRAPH_DEPLOY; then
    echo "=== Phase 3: Graph-driven deploy (composition.deploy pattern) ==="
    source "$SCRIPT_DIR/deploy_graph.sh"

    export FAMILY_ID="$FAMILY_ID"
    export NODE_ID="$NODE_ID"
    export ECOPRIMALS_PLASMID_BIN="$PLASMIDBIN_DIR"
    export XDG_RUNTIME_DIR="$RUNTIME_DIR"
    export BEARDOG_AUTH_MODE="$NUCLEUS_AUTH_MODE"
    export BEARDOG_FAMILY_SEED="$BEACON_SEED"

    if deploy_from_graph "$GRAPH_FILE" "$PLASMIDBIN_DIR" "$RUNTIME_DIR" "$BIND_ADDRESS" "$FAMILY_ID"; then
        echo ""
        echo "╔══════════════════════════════════════════════╗"
        echo "║  Graph deploy complete. Gate is live.        ║"
        echo "╚══════════════════════════════════════════════╝"
    else
        echo ""
        echo "WARNING: Graph deploy had failures. Check logs in /tmp/"
    fi

    echo ""
    echo "  Family ID:   $FAMILY_ID"
    echo "  Node ID:     $NODE_ID"
    echo "  Graph:       $GRAPH_FILE"
    echo "  Composition: $COMPOSITION (graph-driven)"
    echo ""
    echo "  To stop:   bash $0 --stop"
    echo "  To check:  bash $0 --status"
    exit 0
fi

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

source "$SCRIPT_DIR/deploy_primal_start.sh"

for p in $PRIMALS; do
    start_primal "$p"
done

echo ""

# ── Phase 4: Verify ──────────────────────────────────────────────────────

echo "=== Phase 4: Verify ==="

source "$SCRIPT_DIR/deploy_health_check.sh"

if verify_primals "$PRIMALS"; then
    ALL_OK=true
else
    ALL_OK=false
fi

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
