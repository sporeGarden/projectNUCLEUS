#!/usr/bin/env bash
# deploy_btsp_auth_shadow.sh — Enable BTSP dual-auth shadow on JupyterHub (H2-01→H2-04)
#
# Installs the BTSPAuthenticator plugin alongside existing PAM auth.
# During the 7-day shadow run, both BTSP ionic tokens and PAM passwords
# are accepted. Auth method is logged for comparison analysis.
#
# Prerequisites:
#   - BearDog running on $BEARDOG_PORT (default 9100) with auth.verify_ionic
#   - JupyterHub config writable
#   - jupyterhub_btsp_auth.py in deploy/
#
# Usage:
#   bash deploy_btsp_auth_shadow.sh [--enable|--disable|--status]
#
# See: specs/EVOLUTION_GAPS.md (H2-01 through H2-04)
#      specs/TUNNEL_EVOLUTION.md (Step 2b: BTSP Authentication)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

PLUGIN_SRC="$SCRIPT_DIR/jupyterhub_btsp_auth.py"
PLUGIN_DST="${JUPYTERHUB_DIR}/jupyterhub_btsp_auth.py"
SHADOW_LOG="/tmp/btsp-auth-shadow.log"
SHADOW_MARKER="${JUPYTERHUB_DIR}/.btsp_shadow_active"
BTSP_CONFIG_SNIPPET="c.JupyterHub.authenticator_class = 'jupyterhub_btsp_auth.BTSPAuthenticator'"

ACTION="${1:---status}"

log() { echo "[$(date -Iseconds)] $*" | tee -a "$SHADOW_LOG"; }

check_beardog() {
    if echo '{"jsonrpc":"2.0","method":"health.liveness","id":1}' | \
        nc -w 2 "${BEARDOG_HOST:-127.0.0.1}" "${BEARDOG_PORT:-9100}" 2>/dev/null | grep -q "result"; then
        return 0
    fi
    return 1
}

enable_shadow() {
    log "=== BTSP Dual-Auth Shadow — ENABLE ==="

    if [[ ! -f "$PLUGIN_SRC" ]]; then
        log "ERROR: Plugin source not found: $PLUGIN_SRC"
        exit 1
    fi

    if ! check_beardog; then
        log "WARNING: BearDog not reachable on ${BEARDOG_HOST:-127.0.0.1}:${BEARDOG_PORT:-9100}"
        log "  Shadow will start but BTSP auth will fail until BearDog is up"
    fi

    log "Installing plugin to $PLUGIN_DST..."
    cp "$PLUGIN_SRC" "$PLUGIN_DST"
    chmod 644 "$PLUGIN_DST"

    if [[ -f "$JUPYTERHUB_CONFIG" ]]; then
        if grep -q "BTSPAuthenticator" "$JUPYTERHUB_CONFIG"; then
            log "BTSPAuthenticator already in JupyterHub config"
        else
            log "Adding BTSPAuthenticator to JupyterHub config..."
            {
                echo ""
                echo "# --- BTSP Dual-Auth Shadow (H2-01→H2-04) ---"
                echo "import sys, os"
                echo "sys.path.insert(0, os.path.dirname(__file__))"
                echo "$BTSP_CONFIG_SNIPPET"
                echo "c.BTSPAuthenticator.dual_auth = True"
                echo "c.BTSPAuthenticator.beardog_port = int(os.environ.get('BEARDOG_PORT', '${BEARDOG_PORT:-9100}'))"
                echo "# --- End BTSP Shadow ---"
            } >> "$JUPYTERHUB_CONFIG"
        fi
    else
        log "WARNING: JupyterHub config not found: $JUPYTERHUB_CONFIG"
        log "  Manual configuration required"
    fi

    touch "$SHADOW_MARKER"
    date -Iseconds > "$SHADOW_MARKER"
    log "Shadow marker created: $SHADOW_MARKER"
    log "Shadow run started. Monitor with: tail -f $SHADOW_LOG"
    log "Review auth patterns: grep AUTH_ /var/log/jupyterhub.log"
    log ""
    log "IMPORTANT: Restart JupyterHub to activate: sudo systemctl restart jupyterhub"
}

disable_shadow() {
    log "=== BTSP Dual-Auth Shadow — DISABLE ==="

    if [[ -f "$JUPYTERHUB_CONFIG" ]]; then
        if grep -q "BTSP Dual-Auth Shadow" "$JUPYTERHUB_CONFIG"; then
            log "Removing BTSPAuthenticator from JupyterHub config..."
            local tmp
            tmp=$(mktemp)
            awk '/# --- BTSP Dual-Auth Shadow/,/# --- End BTSP Shadow ---/{next}1' \
                "$JUPYTERHUB_CONFIG" > "$tmp"
            mv "$tmp" "$JUPYTERHUB_CONFIG"
            log "Config reverted"
        else
            log "BTSPAuthenticator not found in config — nothing to remove"
        fi
    fi

    rm -f "$SHADOW_MARKER"
    log "Shadow disabled. Restart JupyterHub: sudo systemctl restart jupyterhub"
}

show_status() {
    echo "=== BTSP Dual-Auth Shadow Status ==="
    echo ""

    if [[ -f "$SHADOW_MARKER" ]]; then
        local started
        started=$(cat "$SHADOW_MARKER")
        echo "  Shadow: ACTIVE (since $started)"
    else
        echo "  Shadow: INACTIVE"
    fi

    if [[ -f "$PLUGIN_DST" ]]; then
        echo "  Plugin: INSTALLED ($PLUGIN_DST)"
    else
        echo "  Plugin: NOT INSTALLED"
    fi

    if check_beardog; then
        echo "  BearDog: REACHABLE (${BEARDOG_HOST:-127.0.0.1}:${BEARDOG_PORT:-9100})"
    else
        echo "  BearDog: UNREACHABLE"
    fi

    if [[ -f "$JUPYTERHUB_CONFIG" ]] && grep -q "BTSPAuthenticator" "$JUPYTERHUB_CONFIG"; then
        echo "  Config: BTSP authenticator CONFIGURED"
    else
        echo "  Config: PAM only (no BTSP)"
    fi

    if [[ -f "$SHADOW_LOG" ]]; then
        local count
        count=$(grep -c "AUTH_BTSP\|AUTH_PAM\|AUTH_FAIL" "$SHADOW_LOG" 2>/dev/null || echo 0)
        echo "  Auth events: $count logged"
    fi

    echo ""
}

case "$ACTION" in
    --enable)  enable_shadow ;;
    --disable) disable_shadow ;;
    --status)  show_status ;;
    *)
        echo "Usage: $0 [--enable|--disable|--status]"
        exit 1
        ;;
esac
