#!/usr/bin/env bash
# nucleus_config.sh — Single source of truth for projectNUCLEUS configuration
#
# Source this file from any deploy, validation, or benchScale script:
#   SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
#   source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
#     || source "${SCRIPT_DIR}/../deploy/nucleus_config.sh" 2>/dev/null \
#     || source "${SCRIPT_DIR}/../../deploy/nucleus_config.sh" 2>/dev/null \
#     || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }
#
# All values are overridable via environment variables.

# --- Paths ---
GATE_HOME="${GATE_HOME:-$HOME}"
NUCLEUS_PROJECT_ROOT="${NUCLEUS_PROJECT_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(cd "$NUCLEUS_PROJECT_ROOT/../.." 2>/dev/null && pwd)}"
PLASMIDBIN_DIR="${PLASMIDBIN_DIR:-$ECOPRIMALS_ROOT/infra/plasmidBin}"
WETSPRING_DIR="${WETSPRING_DIR:-$ECOPRIMALS_ROOT/springs/wetSpring}"
ABG_SHARED="${ABG_SHARED:-$GATE_HOME/shared/abg}"
JUPYTERHUB_DIR="${JUPYTERHUB_DIR:-$GATE_HOME/jupyterhub}"
JUPYTERHUB_CONFIG="${JUPYTERHUB_CONFIG:-$JUPYTERHUB_DIR/jupyterhub_config.py}"
SPOREPRINT_REPO="${SPOREPRINT_REPO:-$ECOPRIMALS_ROOT/infra/sporePrint}"
CLOUDFLARED_DIR="${CLOUDFLARED_DIR:-$GATE_HOME/.cloudflared}"
RUNTIME_DIR="${RUNTIME_DIR:-/tmp/biomeos}"

# --- Network ---
NUCLEUS_BIND_ADDRESS="${NUCLEUS_BIND_ADDRESS:-127.0.0.1}"
JUPYTERHUB_PORT="${JUPYTERHUB_PORT:-8000}"
SPOREPRINT_LOCAL_PORT="${SPOREPRINT_LOCAL_PORT:-8880}"
LAB_URL="${LAB_URL:-https://lab.primals.eco}"
GIT_URL="${GIT_URL:-https://git.primals.eco}"
SITE_URL="${SITE_URL:-https://primals.eco}"

# --- Cloudflare / DNS ---
CF_ZONE_NAME="${CF_ZONE_NAME:-primals.eco}"
CF_TUNNEL_ID="${CF_TUNNEL_ID:-d4c15fb6-d047-40fe-82d6-e324a5593421}"
CF_TUNNEL_CNAME="${CF_TUNNEL_CNAME:-${CF_TUNNEL_ID}.cfargotunnel.com}"
CF_API_TOKEN_FILE="${CF_API_TOKEN_FILE:-$GATE_HOME/.cloudflared/cf_api_token}"

# --- Primal ports (post-interstadial canonical) ---
BEARDOG_PORT="${BEARDOG_PORT:-9100}"
SONGBIRD_PORT="${SONGBIRD_PORT:-9200}"
SQUIRREL_PORT="${SQUIRREL_PORT:-9300}"
TOADSTOOL_PORT="${TOADSTOOL_PORT:-9400}"
NESTGATE_PORT="${NESTGATE_PORT:-9500}"
RHIZOCRYPT_PORT="${RHIZOCRYPT_PORT:-9601}"
RHIZOCRYPT_RPC_PORT="${RHIZOCRYPT_RPC_PORT:-9602}"
LOAMSPINE_PORT="${LOAMSPINE_PORT:-9700}"
CORALREEF_PORT="${CORALREEF_PORT:-9730}"
BARRACUDA_PORT="${BARRACUDA_PORT:-9740}"
BIOMEOS_PORT="${BIOMEOS_PORT:-9800}"
SWEETGRASS_PORT="${SWEETGRASS_PORT:-9850}"
SWEETGRASS_BTSP_PORT="${SWEETGRASS_BTSP_PORT:-9851}"
PETALTONGUE_PORT="${PETALTONGUE_PORT:-9900}"
SKUNKBAT_PORT="${SKUNKBAT_PORT:-9140}"

# --- MethodGate (JH-0/JH-1, Phase 60) ---
# "enforced" (default) = require valid ionic token with scope match (-32001 rejection)
# "permissive" = log + allow unauthenticated calls (testing only)
NUCLEUS_AUTH_MODE="${NUCLEUS_AUTH_MODE:-enforced}"

# --- cellMembrane (fieldMouse VPS) ---
MEMBRANE_VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
MEMBRANE_VPS_USER="${MEMBRANE_VPS_USER:-root}"
SONGBIRD_TURN_SERVER="${SONGBIRD_TURN_SERVER:-${MEMBRANE_VPS_IP}:3478}"
SONGBIRD_TURN_USERNAME="${SONGBIRD_TURN_USERNAME:-nucleus-relay}"
RUSTDESK_ID_SERVER="${RUSTDESK_ID_SERVER:-${MEMBRANE_VPS_IP}}"
RUSTDESK_RELAY_SERVER="${RUSTDESK_RELAY_SERVER:-${MEMBRANE_VPS_IP}}"

# --- ABG user management ---
ABG_UID_MIN="${ABG_UID_MIN:-1001}"
ABG_UID_MAX="${ABG_UID_MAX:-1099}"
ABG_TIERS=(observer compute admin reviewer)

# Primal name-to-port map (for iteration)
declare -A PRIMAL_PORTS=(
    [beardog]=$BEARDOG_PORT
    [songbird]=$SONGBIRD_PORT
    [squirrel]=$SQUIRREL_PORT
    [toadstool]=$TOADSTOOL_PORT
    [nestgate]=$NESTGATE_PORT
    [rhizocrypt]=$RHIZOCRYPT_PORT
    [loamspine]=$LOAMSPINE_PORT
    [coralreef]=$CORALREEF_PORT
    [barracuda]=$BARRACUDA_PORT
    [biomeos]=$BIOMEOS_PORT
    [sweetgrass]=$SWEETGRASS_PORT
    [petaltongue]=$PETALTONGUE_PORT
    [skunkbat]=$SKUNKBAT_PORT
)

# All primal ports as a flat list (for iteration in validation scripts)
ALL_PRIMAL_PORTS_LIST=(
    $BEARDOG_PORT $SONGBIRD_PORT $SQUIRREL_PORT $TOADSTOOL_PORT
    $NESTGATE_PORT $RHIZOCRYPT_PORT $RHIZOCRYPT_RPC_PORT
    $LOAMSPINE_PORT $CORALREEF_PORT $BARRACUDA_PORT
    $BIOMEOS_PORT $SWEETGRASS_PORT $PETALTONGUE_PORT $SKUNKBAT_PORT
)
