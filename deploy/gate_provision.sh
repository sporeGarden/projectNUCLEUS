#!/usr/bin/env bash
# gate_provision.sh — Provision a gate as a tunnel membrane replica
#
# Sets up a remote host as a cloudflared tunnel replica. The tunnel
# carries only membrane traffic (lab.primals.eco, git.primals.eco) —
# the public face (primals.eco) lives on GitHub Pages CDN and is not
# routed through the tunnel.
#
# Usage:
#   gate_provision.sh <target-host> [--dry-run] [--full]
#
#   target-host   SSH-reachable hostname or user@host
#   --dry-run     Show what would happen without executing
#   --full        Include JupyterHub routes (primary gate)
#                 Default is replica (observer + Forgejo only)
#
# Prerequisites:
#   - SSH key access to target host (passwordless)
#   - sudo access on target host
#
# What gets installed:
#   - cloudflared binary + tunnel credentials
#   - cloudflared-replica.service (Restart=always)
#
# Evolution: bash (now) → Rust (tunnelKeeper absorbs) → primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

TUNNEL_NAME="nucleus-lab"
CF_CRED_FILE="${CLOUDFLARED_DIR}/d4c15fb6-d047-40fe-82d6-e324a5593421.json"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${GREEN}[provision]${NC} $*"; }
warn() { echo -e "${YELLOW}[provision]${NC} $*"; }
err()  { echo -e "${RED}[provision]${NC} $*" >&2; }
info() { echo -e "${CYAN}[provision]${NC} $*"; }

usage() {
    echo "Usage: gate_provision.sh <target-host> [--dry-run] [--full]"
    echo
    echo "Provisions a remote host as a tunnel membrane replica."
    echo
    echo "Cell membrane model:"
    echo "  primals.eco       → GitHub Pages CDN (extracellular, not tunneled)"
    echo "  lab.primals.eco   → tunnel (membrane channel → observer/JupyterHub)"
    echo "  git.primals.eco   → tunnel (membrane channel → Forgejo)"
    echo
    echo "Options:"
    echo "  --dry-run   Show commands without executing"
    echo "  --full      Include JupyterHub routes (for primary gate)"
    exit 1
}

DRY_RUN=false
FULL_MODE=false
TARGET=""

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --full)    FULL_MODE=true ;;
        --help|-h) usage ;;
        *)         TARGET="$arg" ;;
    esac
done

[[ -z "$TARGET" ]] && usage

ssh_cmd() {
    if $DRY_RUN; then
        echo "  [dry-run] ssh $TARGET: $*"
    else
        ssh -o ConnectTimeout=10 -o BatchMode=yes "$TARGET" "$@"
    fi
}

run_cmd() {
    if $DRY_RUN; then
        echo "  [dry-run] $*"
    else
        "$@"
    fi
}

# ── Phase 1: Pre-flight ──────────────────────────────────────────────────

log "Phase 1: Pre-flight checks"

if ! $DRY_RUN; then
    if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$TARGET" true 2>/dev/null; then
        err "Cannot SSH to $TARGET — check key access"
        exit 1
    fi
fi
log "  SSH connectivity: OK"

if [[ ! -f "$CF_CRED_FILE" ]]; then
    err "Tunnel credentials not found at $CF_CRED_FILE"
    exit 1
fi
log "  Local tunnel credentials: found"

REMOTE_USER=$(ssh_cmd 'whoami' 2>/dev/null || echo "unknown")
REMOTE_HOME=$(ssh_cmd 'echo $HOME' 2>/dev/null || echo "/home/$REMOTE_USER")
log "  Remote user: $REMOTE_USER ($REMOTE_HOME)"

# ── Phase 2: Install cloudflared ──────────────────────────────────────────

log "Phase 2: cloudflared"

HAS_CF=$(ssh_cmd 'which cloudflared 2>/dev/null || echo ""')
if [[ -n "$HAS_CF" ]]; then
    CF_VER=$(ssh_cmd 'cloudflared --version 2>&1 | head -1')
    log "  Already installed: $CF_VER"
else
    log "  Installing cloudflared..."
    ssh_cmd 'curl -fsSL https://github.com/cloudflare/cloudflared/releases/latest/download/cloudflared-linux-amd64 -o /tmp/cloudflared && chmod +x /tmp/cloudflared && sudo mv /tmp/cloudflared /usr/local/bin/cloudflared'
    log "  Installed cloudflared"
fi

# ── Phase 3: Transfer tunnel credentials ──────────────────────────────────

log "Phase 3: Tunnel credentials"

ssh_cmd "mkdir -p ${REMOTE_HOME}/.cloudflared"
run_cmd scp -q "$CF_CRED_FILE" "${TARGET}:${REMOTE_HOME}/.cloudflared/"
ssh_cmd "chmod 600 ${REMOTE_HOME}/.cloudflared/*.json"
log "  Credentials transferred"

# ── Phase 4: Generate and deploy config.yml ───────────────────────────────

log "Phase 4: Tunnel configuration (membrane channels)"

if $FULL_MODE; then
    info "  Mode: FULL (JupyterHub + observer + Forgejo)"
    TUNNEL_CONFIG="tunnel: ${TUNNEL_NAME}
credentials-file: ${REMOTE_HOME}/.cloudflared/$(basename "$CF_CRED_FILE")

# Cell membrane — tunnel carries only inward-bound traffic.
# primals.eco lives on GitHub Pages CDN (extracellular).

ingress:
  # JupyterHub core routes (authenticated compute access)
  - hostname: lab.primals.eco
    path: /hub/.*
    service: http://127.0.0.1:${JUPYTERHUB_PORT}
  - hostname: lab.primals.eco
    path: /user/.*
    service: http://127.0.0.1:${JUPYTERHUB_PORT}
  - hostname: lab.primals.eco
    path: /services/.*
    service: http://127.0.0.1:${JUPYTERHUB_PORT}
  - hostname: lab.primals.eco
    path: /api/.*
    service: http://127.0.0.1:${JUPYTERHUB_PORT}

  # Static observer surface
  - hostname: lab.primals.eco
    service: http://127.0.0.1:8866

  # Forgejo (sovereign git)
  - hostname: git.primals.eco
    service: http://127.0.0.1:3000

  - service: http_status:404"
else
    info "  Mode: REPLICA (observer + Forgejo)"
    TUNNEL_CONFIG="tunnel: ${TUNNEL_NAME}
credentials-file: ${REMOTE_HOME}/.cloudflared/$(basename "$CF_CRED_FILE")

# Cell membrane — replica gate. Observer and Forgejo channels only.
# JupyterHub requests will 502 here and Cloudflare retries on the
# primary gate where JupyterHub is running.

ingress:
  # Static observer surface
  - hostname: lab.primals.eco
    service: http://127.0.0.1:8866

  # Forgejo (sovereign git)
  - hostname: git.primals.eco
    service: http://127.0.0.1:3000

  - service: http_status:404"
fi

if $DRY_RUN; then
    echo "  [dry-run] Would write config.yml:"
    echo "$TUNNEL_CONFIG" | head -5
    echo "  ..."
else
    echo "$TUNNEL_CONFIG" | ssh "$TARGET" "cat > ${REMOTE_HOME}/.cloudflared/config.yml"
fi
log "  config.yml deployed"

# ── Phase 5: Install systemd service ─────────────────────────────────────

log "Phase 5: systemd service"

CF_SERVICE="[Unit]
Description=cloudflared tunnel membrane replica (${TUNNEL_NAME})
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
User=${REMOTE_USER}
Group=${REMOTE_USER}
ExecStart=/usr/local/bin/cloudflared tunnel --config ${REMOTE_HOME}/.cloudflared/config.yml run ${TUNNEL_NAME}
Restart=always
RestartSec=5
TimeoutStopSec=10

[Install]
WantedBy=multi-user.target"

if $DRY_RUN; then
    echo "  [dry-run] Would install: cloudflared-replica.service"
else
    echo "$CF_SERVICE" | ssh "$TARGET" "sudo tee /etc/systemd/system/cloudflared-replica.service > /dev/null"
    ssh_cmd "sudo systemctl daemon-reload"
    ssh_cmd "sudo systemctl enable --now cloudflared-replica.service"
fi
log "  Service installed and started"

# ── Phase 6: Verify ──────────────────────────────────────────────────────

log "Phase 6: Verification"

if ! $DRY_RUN; then
    sleep 3

    CF_STATUS=$(ssh_cmd "systemctl is-active cloudflared-replica.service 2>/dev/null" || echo "unknown")
    if [[ "$CF_STATUS" == "active" ]]; then
        log "  cloudflared replica: ${GREEN}ACTIVE${NC}"
    else
        warn "  cloudflared replica: $CF_STATUS"
    fi

    CONNECTOR_COUNT=$(ssh_cmd "cloudflared tunnel info ${TUNNEL_NAME} 2>/dev/null | grep -c 'connector' || echo '?'")
    log "  Tunnel connectors: $CONNECTOR_COUNT"
fi

# ── Summary ───────────────────────────────────────────────────────────────

echo
log "Gate provisioned as membrane replica:"
log "  Target:    $TARGET"
log "  Tunnel:    $TUNNEL_NAME"
log "  Mode:      $($FULL_MODE && echo "FULL (primary)" || echo "REPLICA")"
log "  Service:   cloudflared-replica.service"
info ""
info "  Cell membrane model:"
info "    Extracellular: primals.eco → GitHub Pages CDN (not tunneled)"
info "    Membrane:      lab/git.primals.eco → this tunnel replica"
info ""
info "  The gate will rejoin the membrane automatically on reboot."

if $DRY_RUN; then
    echo
    warn "DRY RUN — no changes were made"
fi
