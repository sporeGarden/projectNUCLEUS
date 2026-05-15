#!/usr/bin/env bash
# vps_resize.sh — Resize cellMembrane VPS droplet via doctl
#
# Usage:
#   DIGITALOCEAN_ACCESS_TOKEN=<token> bash vps_resize.sh [--size s-1vcpu-1gb]
#
# Defaults to s-1vcpu-2gb ($12/mo, recommended for Tower + NestGate + knot-dns).
# The droplet will be powered off, resized, then powered on.

set -uo pipefail

DROPLET_NAME="${DROPLET_NAME:-membrane-relay}"
TARGET_SIZE="${1:-s-1vcpu-2gb}"

command -v doctl >/dev/null 2>&1 || { echo "ERROR: doctl not installed"; exit 1; }

if [[ -z "${DIGITALOCEAN_ACCESS_TOKEN:-}" ]]; then
  echo "ERROR: DIGITALOCEAN_ACCESS_TOKEN env var required"
  echo "Get one from: https://cloud.digitalocean.com/account/api/tokens"
  exit 1
fi

echo "Finding droplet: $DROPLET_NAME"
DROPLET_ID=$(doctl compute droplet list --tag-name membrane --format ID,Name --no-header 2>/dev/null | grep "$DROPLET_NAME" | awk '{print $1}')

if [[ -z "$DROPLET_ID" ]]; then
  DROPLET_ID=$(doctl compute droplet list --format ID,Name --no-header 2>/dev/null | grep "$DROPLET_NAME" | awk '{print $1}')
fi

if [[ -z "$DROPLET_ID" ]]; then
  echo "ERROR: Droplet '$DROPLET_NAME' not found"
  echo "Available droplets:"
  doctl compute droplet list --format ID,Name,Size,Region,Status --no-header
  exit 1
fi

CURRENT_SIZE=$(doctl compute droplet get "$DROPLET_ID" --format Size --no-header)
echo "Droplet ID: $DROPLET_ID"
echo "Current size: $CURRENT_SIZE"
echo "Target size: $TARGET_SIZE"

if [[ "$CURRENT_SIZE" == "$TARGET_SIZE" ]]; then
  echo "Already at target size. No resize needed."
  exit 0
fi

echo ""
echo "=== Size Options ==="
echo "  s-1vcpu-512mb-10gb  -> \$4/mo (512MB, relay-only)"
echo "  s-1vcpu-1gb         -> \$6/mo (1GB, Tower comfortable)"
echo "  s-1vcpu-2gb         -> \$12/mo (2GB, Tower + cache + DNS, RECOMMENDED)"
echo "  s-2vcpu-4gb         -> \$24/mo (4GB, full Model A)"
echo ""

read -rp "Proceed with resize $CURRENT_SIZE -> $TARGET_SIZE? [y/N] " confirm
if [[ "$confirm" != [yY] ]]; then
  echo "Aborted."
  exit 0
fi

echo "Powering off droplet..."
doctl compute droplet-action power-off "$DROPLET_ID" --wait

echo "Resizing to $TARGET_SIZE..."
doctl compute droplet-action resize "$DROPLET_ID" --size "$TARGET_SIZE" --wait

echo "Powering on droplet..."
doctl compute droplet-action power-on "$DROPLET_ID" --wait

echo "Waiting for SSH..."
IP=$(doctl compute droplet get "$DROPLET_ID" --format PublicIPv4 --no-header)
attempts=0
while ! ssh -o BatchMode=yes -o ConnectTimeout=3 "root@$IP" "echo ok" 2>/dev/null; do
  attempts=$((attempts + 1))
  if [[ $attempts -ge 30 ]]; then
    echo "SSH not available after 90s. Check droplet status."
    exit 1
  fi
  sleep 3
done

echo ""
echo "=== Resize Complete ==="
ssh -o BatchMode=yes "root@$IP" "free -h && echo '---' && systemctl list-units --type=service --all | grep -E 'songbird|hbb|beardog|skunkbat'"
echo ""
echo "VPS resized to $TARGET_SIZE and all services running."
