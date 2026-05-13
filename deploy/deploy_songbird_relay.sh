#!/usr/bin/env bash
# deploy_songbird_relay.sh — Deploy Songbird TURN relay to VPS (H2-14)
#
# Provisions a Songbird TURN relay on a VPS for NAT traversal shadow run.
# Follows the upstream deployment guide: primals/songBird/deployment/relay/README.md
#
# Prerequisites:
#   - VPS with SSH access and public IP (Hetzner/OVH/Linode, ~$5/mo)
#   - Songbird binary built for target arch (or from plasmidBin)
#   - BearDog keys for relay credential material
#
# Usage:
#   bash deploy_songbird_relay.sh --host <vps-ip> [--user root] [--port 3478]
#
# See: primals/songBird/deployment/relay/README.md (Wave 202)
#      infra/wateringHole/INTERSTADIAL_EXIT_CRITERIA.md

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

VPS_HOST=""
VPS_USER="root"
RELAY_PORT="3478"
SONGBIRD_BIN="${PLASMIDBIN_DIR:-$GATE_HOME/plasmidBin}/primals/songbird"
SONGBIRD_SRC="/home/irongate/Development/ecoPrimals/primals/songBird"
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --host) VPS_HOST="$2"; shift 2 ;;
        --user) VPS_USER="$2"; shift 2 ;;
        --port) RELAY_PORT="$2"; shift 2 ;;
        --dry-run) DRY_RUN=true; shift ;;
        *) shift ;;
    esac
done

if [[ -z "$VPS_HOST" ]]; then
    echo "Usage: deploy_songbird_relay.sh --host <vps-ip> [--user root] [--port 3478]"
    echo ""
    echo "This script deploys a Songbird TURN relay to a VPS for NAT shadow testing."
    echo "See primals/songBird/deployment/relay/README.md for manual steps."
    exit 1
fi

echo "══ Songbird TURN Relay Deployment (H2-14) ══"
echo "  VPS:      ${VPS_USER}@${VPS_HOST}"
echo "  Port:     ${RELAY_PORT}/udp"
echo "  Binary:   ${SONGBIRD_BIN}"
echo "  Dry run:  ${DRY_RUN}"
echo ""

run_cmd() {
    if $DRY_RUN; then
        echo "  [DRY-RUN] $*"
    else
        "$@"
    fi
}

# Phase 1: Locate binary
if [[ ! -x "$SONGBIRD_BIN" ]]; then
    echo "  Binary not in plasmidBin, checking source build..."
    if [[ -f "$SONGBIRD_SRC/target/release/songbird" ]]; then
        SONGBIRD_BIN="$SONGBIRD_SRC/target/release/songbird"
        echo "  Using source build: $SONGBIRD_BIN"
    else
        echo "ERROR: No songbird binary found."
        echo "  Build: cd $SONGBIRD_SRC && cargo build --release"
        echo "  Or:    deploy/fetch_primals.sh"
        exit 1
    fi
fi

# Phase 2: Generate credential material
RELAY_KEY=$(openssl rand -hex 32)
RELAY_USER="nucleus-relay"
echo "  Generated relay credential: ${RELAY_USER}:${RELAY_KEY:0:16}..."

# Phase 3: Copy binary
echo ""
echo "  Phase 1: Copying binary to VPS..."
run_cmd scp "$SONGBIRD_BIN" "${VPS_USER}@${VPS_HOST}:/usr/local/bin/songbird"

# Phase 4: Copy systemd unit
echo "  Phase 2: Installing systemd service..."
UNIT_SRC="$SONGBIRD_SRC/deployment/systemd/songbird-relay.service"
if [[ -f "$UNIT_SRC" ]]; then
    run_cmd scp "$UNIT_SRC" "${VPS_USER}@${VPS_HOST}:/etc/systemd/system/"
else
    echo "  WARNING: systemd unit not found at $UNIT_SRC"
    echo "  Creating from template..."
fi

# Phase 5: Configure credentials + firewall + start
echo "  Phase 3: Configuring relay on VPS..."
run_cmd ssh "${VPS_USER}@${VPS_HOST}" bash -s <<REMOTE_SETUP
set -euo pipefail
mkdir -p /etc/songbird
cat > /etc/songbird/relay-credentials <<CRED
# nucleus relay — generated $(date -u +%Y-%m-%dT%H:%M:%SZ)
${RELAY_USER}:${RELAY_KEY}
CRED
chmod 640 /etc/songbird/relay-credentials

chmod +x /usr/local/bin/songbird

# Firewall
ufw allow ${RELAY_PORT}/udp comment "TURN relay signaling" 2>/dev/null || true
ufw allow 49152:65535/udp comment "TURN relay ephemeral" 2>/dev/null || true

# Start
systemctl daemon-reload
systemctl enable --now songbird-relay

echo "=== Relay status ==="
systemctl status songbird-relay --no-pager -l || true
REMOTE_SETUP

# Phase 6: Record local config for NAT shadow
echo ""
echo "  Phase 4: Recording local NAT shadow config..."

NAT_CONFIG="${SCRIPT_DIR}/songbird_relay.env"
cat > "$NAT_CONFIG" <<ENV_FILE
# Songbird TURN relay — deployed $(date -u +%Y-%m-%dT%H:%M:%SZ)
# Source these vars for NAT shadow run (H2-14)
SONGBIRD_TURN_SERVER=${VPS_HOST}:${RELAY_PORT}
SONGBIRD_TURN_USERNAME=${RELAY_USER}
SONGBIRD_TURN_KEY=${RELAY_KEY}
ENV_FILE
chmod 600 "$NAT_CONFIG"

echo "  Config saved: $NAT_CONFIG"

echo ""
echo "══ Deployment Complete ══"
echo "  Relay running on ${VPS_HOST}:${RELAY_PORT}/udp"
echo "  Credentials: /etc/songbird/relay-credentials on VPS"
echo "  Local env: $NAT_CONFIG"
echo ""
echo "  To use in shadow run:"
echo "    source $NAT_CONFIG"
echo "    # Songbird ConnectionFallbackChain Tier 4 will auto-route through relay"
echo ""
echo "  To verify:"
echo "    ssh ${VPS_USER}@${VPS_HOST} journalctl -u songbird-relay -f"
echo ""
echo "  Parity test:"
echo "    infra/benchScale/scenarios/songbird_nat_parity.sh"
