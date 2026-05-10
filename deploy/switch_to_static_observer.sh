#!/usr/bin/env bash
# Switch observer surface from dynamic Voila to static pre-rendered HTML.
#
# Steps:
#   1. Run pappusCast export to generate all static HTML
#   2. Stop voila-public + voila-redirect services
#   3. Install and start observer-static service on the same port (8866)
#
# Reversible: run switch_to_voila_observer.sh to go back.
#
# Usage: sudo bash switch_to_static_observer.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

GATE_HOME="${GATE_HOME:-$(getent passwd irongate | cut -d: -f6 2>/dev/null || echo /home/irongate)}"
export GATE_HOME HOME="$GATE_HOME"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null || true

PYTHON="${GATE_HOME}/miniforge3/envs/jupyterhub/bin/python3"

echo "=== Step 1: Generate static HTML via pappusCast ==="
sudo -u voila \
    GATE_HOME="$GATE_HOME" HOME="$GATE_HOME" \
    ABG_SHARED="$ABG_SHARED" JUPYTER_BIN="${GATE_HOME}/miniforge3/envs/jupyterhub/bin" \
    "$PYTHON" "${SCRIPT_DIR}/pappusCast.py" export
echo ""

HTML_DIR="${ABG_SHARED:-$HOME/shared/abg}/public/.pappusCast/html_export"
if [ ! -f "${HTML_DIR}/index.html" ]; then
    echo "ERROR: index.html not generated. Check pappusCast export output."
    exit 1
fi

NB_COUNT=$(find "$HTML_DIR" -name "*.html" ! -name "index.html" | wc -l)
echo "Static HTML ready: ${NB_COUNT} notebooks + index.html"
echo ""

echo "=== Step 2: Stop Voila services ==="
systemctl stop voila-redirect.service 2>/dev/null || true
systemctl stop voila-public.service 2>/dev/null || true
systemctl disable voila-redirect.service 2>/dev/null || true
systemctl disable voila-public.service 2>/dev/null || true
echo "Voila services stopped and disabled"
echo ""

echo "=== Step 3: Install static observer service ==="
cp "${SCRIPT_DIR}/systemd/observer-static.service" /etc/systemd/system/
systemctl daemon-reload
systemctl enable observer-static.service
systemctl start observer-static.service
echo ""

echo "=== Verification ==="
sleep 1
if systemctl is-active --quiet observer-static.service; then
    echo "PASS: observer-static.service is active"
else
    echo "FAIL: observer-static.service failed to start"
    systemctl status observer-static.service --no-pager
    exit 1
fi

HTTP_CODE=$(curl -sf -o /dev/null -w "%{http_code}" http://127.0.0.1:8866/ --max-time 5 2>/dev/null || echo "000")
if [ "$HTTP_CODE" = "200" ]; then
    echo "PASS: port 8866 returns HTTP 200"
else
    echo "WARN: port 8866 returned HTTP ${HTTP_CODE}"
fi

echo ""
echo "Observer switched to static pre-rendered HTML."
echo "Voila services disabled (can re-enable with: sudo bash switch_to_voila_observer.sh)"
