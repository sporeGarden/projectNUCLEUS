#!/usr/bin/env bash
#
# Rotate JupyterHub cookie secret. Invalidates all active sessions.
# Run monthly via cron or manually after a security event.
#
# Usage: sudo bash deploy/rotate_cookie_secret.sh
#
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

COOKIE_SECRET="${JUPYTERHUB_DIR}/jupyterhub_cookie_secret"
SERVICE="jupyterhub"

if [[ $EUID -ne 0 ]]; then
    echo "ERROR: must run as root (sudo)" >&2
    exit 1
fi

echo "=== Cookie Secret Rotation ==="
echo "Date: $(date -Iseconds)"

if [[ -f "$COOKIE_SECRET" ]]; then
    old_mod=$(stat -c "%y" "$COOKIE_SECRET" 2>/dev/null)
    echo "Old secret: $COOKIE_SECRET (modified $old_mod)"
    rm -f "$COOKIE_SECRET"
    echo "Deleted old secret."
else
    echo "No existing secret found — will generate fresh."
fi

echo "Restarting $SERVICE..."
systemctl restart "$SERVICE"
sleep 3

if systemctl is-active --quiet "$SERVICE"; then
    echo "JupyterHub restarted successfully."
else
    echo "ERROR: JupyterHub failed to restart!" >&2
    systemctl status "$SERVICE" --no-pager >&2
    exit 1
fi

if [[ -f "$COOKIE_SECRET" ]]; then
    new_mod=$(stat -c "%y" "$COOKIE_SECRET" 2>/dev/null)
    new_perms=$(stat -c "%a" "$COOKIE_SECRET" 2>/dev/null)
    echo "New secret generated: $COOKIE_SECRET (modified $new_mod, mode $new_perms)"

    if [[ "$new_perms" != "600" ]]; then
        chmod 600 "$COOKIE_SECRET"
        echo "Fixed permissions to 600."
    fi
else
    echo "WARNING: cookie secret not regenerated — check JupyterHub config" >&2
fi

echo "=== Rotation complete. All active sessions invalidated. ==="
