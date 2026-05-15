#!/usr/bin/env bash
# membrane_telemetry.sh — Unified shadow collection across both membranes
#
# Collects telemetry from the external membrane (VPS) and internal membrane
# (gate) in a single pass. Designed for cron (every 15 min or hourly).
#
# Shadow data is PERMANENT — it does not stop after cutover. Continuous
# comparison detects regressions, baseline drift, and cost anomalies.
#
# Usage:
#   bash membrane_telemetry.sh                     # full collection
#   bash membrane_telemetry.sh --external-only     # VPS probes only
#   bash membrane_telemetry.sh --internal-only     # gate probes only
#
# Cron example (every 15 minutes):
#   */15 * * * * /path/to/deploy/membrane_telemetry.sh >> /tmp/membrane_telemetry.log 2>&1

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh" 2>/dev/null || true

MODE="all"
[[ "${1:-}" == "--external-only" ]] && MODE="external"
[[ "${1:-}" == "--internal-only" ]] && MODE="internal"

TELEMETRY_DIR="${MEMBRANE_TELEMETRY_DIR:-${NUCLEUS_PROJECT_ROOT:-$SCRIPT_DIR/..}/validation/baselines/daily}"
mkdir -p "$TELEMETRY_DIR"

TODAY="$(date -u +%Y-%m-%d)"
CSV_FILE="${TELEMETRY_DIR}/membrane_telemetry_${TODAY}.csv"

if [ ! -f "$CSV_FILE" ]; then
    echo "timestamp_utc,probe_name,target,latency_ms,status,http_code,extra" > "$CSV_FILE"
fi

emit() {
    local probe="$1" target="$2" latency="$3" status="$4" code="${5:-0}" extra="${6:-}"
    local ts
    ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "${ts},${probe},${target},${latency},${status},${code},${extra}" >> "$CSV_FILE"
}

probe_http() {
    local probe_name="$1" url="$2" extra_curl="${3:-}"
    local start_ns end_ns latency_ms code
    start_ns=$(date +%s%N)
    code=$(curl -sS -o /dev/null -w "%{http_code}" --max-time 10 $extra_curl "$url" 2>/dev/null) || code="0"
    end_ns=$(date +%s%N)
    latency_ms=$(( (end_ns - start_ns) / 1000000 ))

    local status="ok"
    [[ "$code" == "0" ]] && status="unreachable"
    [[ "$code" =~ ^[45] ]] && status="error"

    emit "$probe_name" "$url" "$latency_ms" "$status" "$code"
}

probe_tcp() {
    local probe_name="$1" host="$2" port="$3"
    local start_ns end_ns latency_ms status="ok"
    start_ns=$(date +%s%N)
    if nc -z -w 5 "$host" "$port" 2>/dev/null; then
        end_ns=$(date +%s%N)
        latency_ms=$(( (end_ns - start_ns) / 1000000 ))
    else
        end_ns=$(date +%s%N)
        latency_ms=$(( (end_ns - start_ns) / 1000000 ))
        status="unreachable"
    fi
    emit "$probe_name" "${host}:${port}" "$latency_ms" "$status"
}

probe_rpc() {
    local probe_name="$1" host="$2" port="$3"
    local start_ns end_ns latency_ms status="ok" response
    start_ns=$(date +%s%N)
    response=$(timeout 2 bash -c '
        exec 3<>/dev/tcp/$1/$2 2>/dev/null || exit 1
        echo "{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}" >&3
        read -t 1 line <&3
        exec 3>&-
        echo "$line"
    ' _ "$host" "$port" 2>/dev/null) || response=""
    end_ns=$(date +%s%N)
    latency_ms=$(( (end_ns - start_ns) / 1000000 ))

    if [[ -z "$response" ]]; then
        status="unreachable"
    elif echo "$response" | grep -q '"error"'; then
        status="auth_required"
    fi
    emit "$probe_name" "${host}:${port}" "$latency_ms" "$status" "0" "rpc"
}

echo "[$(date -u +%H:%M:%S)] membrane_telemetry — mode=$MODE"

if [[ "$MODE" == "external" || "$MODE" == "all" ]]; then
    echo "  Probing external membrane (VPS)..."

    VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
    VPS_HTTP="${MEMBRANE_HTTP_PORT:-80}"

    # Caddy health endpoint
    probe_http "caddy_health" "http://${VPS_IP}:${VPS_HTTP}/health"

    # Songbird TURN reachability (UDP — TURN listens on UDP only)
    probe_turn_udp() {
        local host="$1" port="$2"
        local start_ns end_ns latency_ms status="ok"
        start_ns=$(date +%s%N)
        if nc -z -u -w 3 "$host" "$port" 2>/dev/null; then
            end_ns=$(date +%s%N)
            latency_ms=$(( (end_ns - start_ns) / 1000000 ))
        else
            end_ns=$(date +%s%N)
            latency_ms=$(( (end_ns - start_ns) / 1000000 ))
            status="unreachable"
        fi
        emit "turn_udp" "${host}:${port}" "$latency_ms" "$status"
    }
    probe_turn_udp "$VPS_IP" "3478"

    # RustDesk hbbs reachability
    probe_tcp "rustdesk_hbbs" "$VPS_IP" "21116"

    # BearDog TLS shadow (BTSP RPC on :8443, not HTTP)
    BTSP_SHADOW_HOST="${BTSP_SHADOW_HOST:-127.0.0.1}"
    BTSP_SHADOW_PORT="${BTSP_SHADOW_PORT:-8443}"
    if nc -z -w 2 "$BTSP_SHADOW_HOST" "$BTSP_SHADOW_PORT" 2>/dev/null; then
        probe_rpc "beardog_tls_shadow" "$BTSP_SHADOW_HOST" "$BTSP_SHADOW_PORT"
    else
        emit "beardog_tls_shadow" "${BTSP_SHADOW_HOST}:${BTSP_SHADOW_PORT}" "0" "not_running"
    fi

    # VPS resource snapshot (requires SSH access)
    VPS_USER="${MEMBRANE_VPS_USER:-root}"
    if ssh -o ConnectTimeout=5 -o BatchMode=yes "${VPS_USER}@${VPS_IP}" true 2>/dev/null; then
        ram_free=$(ssh -o ConnectTimeout=5 "${VPS_USER}@${VPS_IP}" "free -m | awk '/Mem:/{print \$4}'" 2>/dev/null) || ram_free="0"
        disk_pct=$(ssh -o ConnectTimeout=5 "${VPS_USER}@${VPS_IP}" "df -h / | awk 'NR==2{print \$5}' | tr -d '%'" 2>/dev/null) || disk_pct="0"
        svc_count=$(ssh -o ConnectTimeout=5 "${VPS_USER}@${VPS_IP}" "systemctl list-units '*-membrane*' --no-pager --no-legend | grep -c active" 2>/dev/null) || svc_count="0"
        emit "vps_resources" "$VPS_IP" "0" "ok" "0" "ram_free_mb=${ram_free},disk_pct=${disk_pct},active_services=${svc_count}"
    else
        emit "vps_resources" "$VPS_IP" "0" "ssh_unreachable"
    fi
fi

if [[ "$MODE" == "internal" || "$MODE" == "all" ]]; then
    echo "  Probing internal membrane (gate)..."

    BIND="${NUCLEUS_BIND_ADDRESS:-127.0.0.1}"

    # Cloudflare tunnel latency (external baseline — always captured)
    CF_URL="${LAB_URL:-https://lab.primals.eco}/hub/login"
    probe_http "cloudflare_tunnel" "$CF_URL"

    # Per-primal health probes (sample 5 key primals, not all 13)
    for primal in beardog songbird nestgate skunkbat biomeos; do
        port_var="${primal^^}_PORT"
        port="${!port_var:-0}"
        [[ "$port" == "0" ]] && continue
        probe_rpc "primal_${primal}" "$BIND" "$port"
    done

    # Content parity: VPS cache vs GitHub Pages
    VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
    VPS_HTTP="${MEMBRANE_HTTP_PORT:-80}"
    if curl -sf --max-time 5 "http://${VPS_IP}:${VPS_HTTP}/" >/dev/null 2>&1; then
        # Capture TTFB for VPS
        vps_ttfb=$(curl -sS -o /dev/null -w "%{time_starttransfer}" --max-time 10 "http://${VPS_IP}:${VPS_HTTP}/" 2>/dev/null) || vps_ttfb="0"
        vps_ttfb_ms=$(echo "$vps_ttfb * 1000" | bc 2>/dev/null || echo "0")
        emit "content_vps_ttfb" "http://${VPS_IP}:${VPS_HTTP}/" "$vps_ttfb_ms" "ok"

        # Capture TTFB for GitHub Pages
        gh_ttfb=$(curl -sS -o /dev/null -w "%{time_starttransfer}" --max-time 10 "https://primals.eco/" 2>/dev/null) || gh_ttfb="0"
        gh_ttfb_ms=$(echo "$gh_ttfb * 1000" | bc 2>/dev/null || echo "0")
        emit "content_github_ttfb" "https://primals.eco/" "$gh_ttfb_ms" "ok"
    fi

    # BTSP auth events — scan journald (primary) then fall back to log file
    btsp_count=0; pam_count=0; fail_count=0
    auth_source="none"
    if command -v journalctl >/dev/null 2>&1; then
        auth_source="journald"
        jh_log=$(journalctl -u jupyterhub --since "today" --no-pager 2>/dev/null) || jh_log=""
        if [[ -n "$jh_log" ]]; then
            btsp_count=$(echo "$jh_log" | grep -ci "BTSP") || btsp_count=0
            pam_count=$(echo "$jh_log" | grep -ci "PAMAuthenticator\|AUTH_PAM") || pam_count=0
            fail_count=$(echo "$jh_log" | grep -ci "AUTH_FAIL\|failed login\|authentication failed") || fail_count=0
        fi
    else
        JH_LOG="${JUPYTERHUB_DIR:-/home/irongate/jupyterhub}/jupyterhub.log"
        if [[ -f "$JH_LOG" ]]; then
            auth_source="logfile"
            btsp_count=$(grep -c "AUTH_BTSP" "$JH_LOG" 2>/dev/null) || btsp_count=0
            pam_count=$(grep -c "AUTH_PAM" "$JH_LOG" 2>/dev/null) || pam_count=0
            fail_count=$(grep -c "AUTH_FAIL" "$JH_LOG" 2>/dev/null) || fail_count=0
        fi
    fi
    emit "auth_events" "jupyterhub/${auth_source}" "0" "ok" "0" "btsp=${btsp_count};pam=${pam_count};fail=${fail_count}"
fi

LINES_ADDED=$(wc -l < "$CSV_FILE")
echo "  Done. ${LINES_ADDED} total rows in ${CSV_FILE}"
