#!/usr/bin/env bash
# gate_watchdog.sh — Cell membrane health monitor
#
# Monitors the tunnel membrane (lab.primals.eco, git.primals.eco) and
# logs health state. The public face (primals.eco) lives on GitHub Pages
# CDN and does not need monitoring — it has its own SLA.
#
# The watchdog does NOT swap DNS. The membrane is either up (tunnel
# replicas serving) or down (gate is offline, compute unavailable).
# When a gate reboots, systemd restarts cloudflared and the membrane
# self-heals. The watchdog observes and logs.
#
# Modes:
#   check      — single health check cycle
#   loop       — persistent loop with configurable interval (systemd)
#   --status   — show current membrane state
#   --install  — install as systemd service
#
# State files in /tmp/gate_watchdog/:
#   membrane_state     — "healthy", "degraded", or "down"
#   watchdog.log       — health transition log (skunkBat-ready)
#
# Evolution: bash (now) → Rust (tunnelKeeper absorbs) → primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

STATE_DIR="/tmp/gate_watchdog"
STATE_FILE="${STATE_DIR}/membrane_state"
LOG_FILE="${STATE_DIR}/watchdog.log"
CHECK_INTERVAL="${WATCHDOG_INTERVAL:-30}"
CHECK_TIMEOUT=10

mkdir -p "$STATE_DIR"

log() {
    local msg="$(date '+%Y-%m-%d %H:%M:%S') $*"
    echo "$msg"
    echo "$msg" >> "$LOG_FILE"
}

get_state() {
    cat "$STATE_FILE" 2>/dev/null || echo "unknown"
}

set_state() {
    local new_state="$1"
    local old_state
    old_state=$(get_state)
    echo "$new_state" > "$STATE_FILE"
    if [[ "$old_state" != "$new_state" ]]; then
        log "TRANSITION: ${old_state} → ${new_state}"
    fi
}

# Probe a membrane channel. Returns HTTP status code.
probe() {
    local url="$1"
    curl -s -o /dev/null -w '%{http_code}' --max-time "$CHECK_TIMEOUT" "$url" 2>/dev/null || echo "000"
}

do_check() {
    local lab_status observer_status git_status
    lab_status=$(probe "https://lab.primals.eco/")
    git_status=$(probe "https://git.primals.eco/")

    # Also check the extracellular surface (informational only)
    local public_status
    public_status=$(probe "https://primals.eco/")

    local channels_up=0
    local channels_total=2

    [[ "$lab_status" == "200" || "$lab_status" == "302" ]] && channels_up=$((channels_up + 1))
    [[ "$git_status" == "200" || "$git_status" == "302" ]] && channels_up=$((channels_up + 1))

    local new_state
    if [[ "$channels_up" -eq "$channels_total" ]]; then
        new_state="healthy"
    elif [[ "$channels_up" -gt 0 ]]; then
        new_state="degraded"
    else
        new_state="down"
    fi

    local old_state
    old_state=$(get_state)

    set_state "$new_state"

    # Log transitions and periodic status (every 10 minutes when healthy)
    if [[ "$old_state" != "$new_state" || "$new_state" != "healthy" ]]; then
        log "MEMBRANE: ${new_state} (lab:${lab_status} git:${git_status} public:${public_status}) channels=${channels_up}/${channels_total}"
    fi
}

do_loop() {
    log "Membrane watchdog started (interval=${CHECK_INTERVAL}s)"
    log "  Extracellular: primals.eco → GitHub Pages CDN (no tunnel)"
    log "  Membrane:      lab.primals.eco, git.primals.eco → tunnel"
    while true; do
        do_check
        sleep "$CHECK_INTERVAL"
    done
}

do_status() {
    local state
    state=$(get_state)

    local lab_status git_status public_status
    lab_status=$(probe "https://lab.primals.eco/")
    git_status=$(probe "https://git.primals.eco/")
    public_status=$(probe "https://primals.eco/")

    echo "Cell Membrane Status"
    echo ""
    echo "  Extracellular (CDN — no tunnel):"
    echo "    primals.eco:       ${public_status} (GitHub Pages + Cloudflare)"
    echo ""
    echo "  Membrane channels (tunnel → sovereign compute):"
    echo "    lab.primals.eco:   ${lab_status} (observer + JupyterHub)"
    echo "    git.primals.eco:   ${git_status} (Forgejo)"
    echo ""
    echo "  Membrane state:     ${state}"
    echo "  Check interval:     ${CHECK_INTERVAL}s"
    echo ""

    if [[ -f "$LOG_FILE" ]]; then
        echo "Recent log:"
        tail -10 "$LOG_FILE"
    fi
}

do_install() {
    local service_file="/etc/systemd/system/gate-watchdog.service"

    echo "Installing gate-watchdog.service (membrane monitor)..."

    local unit="[Unit]
Description=Cell membrane watchdog — monitors tunnel health for lab/git.primals.eco
After=network-online.target cloudflared-replica.service
Wants=network-online.target

[Service]
Type=simple
User=$(whoami)
Group=$(whoami)
Environment=GATE_HOME=${GATE_HOME}
ExecStart=/usr/bin/bash ${SCRIPT_DIR}/gate_watchdog.sh loop
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target"

    echo "$unit" | sudo tee "$service_file" > /dev/null
    sudo systemctl daemon-reload
    sudo systemctl enable --now gate-watchdog.service

    echo "Installed and started gate-watchdog.service"
    echo "  Monitors: lab.primals.eco, git.primals.eco (membrane)"
    echo "  Does NOT manage: primals.eco (lives on CDN, always up)"
}

case "${1:-check}" in
    check)     do_check ;;
    loop)      do_loop ;;
    --status)  do_status ;;
    --install) do_install ;;
    --help|-h)
        echo "Usage: gate_watchdog.sh [check|loop|--status|--install]"
        echo "  check      Single membrane health check"
        echo "  loop       Persistent monitoring loop (systemd)"
        echo "  --status   Show current membrane state"
        echo "  --install  Install as systemd service"
        ;;
    *)
        echo "Unknown command: $1"
        exit 1
        ;;
esac
