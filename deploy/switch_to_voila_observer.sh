#!/usr/bin/env bash
# LEGACY: Revert observer surface from static HTML back to dynamic Voila.
# Static observer is the production model since 2026-05-10. This script
# is preserved as an emergency rollback path only.
#
# Usage: sudo bash switch_to_voila_observer.sh

set -euo pipefail

echo "=== Stopping static observer ==="
systemctl stop observer-static.service 2>/dev/null || true
systemctl disable observer-static.service 2>/dev/null || true

echo "=== Restarting Voila services ==="
systemctl enable voila-public.service
systemctl start voila-public.service
systemctl enable voila-redirect.service
systemctl start voila-redirect.service

echo ""
if systemctl is-active --quiet voila-redirect.service; then
    echo "PASS: voila-redirect.service is active on port 8866"
else
    echo "FAIL: voila-redirect.service failed to start"
    systemctl status voila-redirect.service --no-pager
    exit 1
fi

echo "Observer reverted to dynamic Voila."
