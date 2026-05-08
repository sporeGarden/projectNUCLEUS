#!/usr/bin/env bash
# Security Validation Pipeline — Five-Layer Penetration Testing
#
# Tests security posture above, at, and below the primal layer:
#
#   Layer 1 (BELOW):      OS/network — open ports, exposed services, firewall
#   Layer 2 (AT):         Primal APIs — auth enforcement, input fuzzing, BTSP
#   Layer 3 (ABOVE):      Application — JupyterHub, tunnel, web endpoints
#   Layer 4 (TIERS):      ABG tier enforcement — filesystem, network, process, API
#   Layer 5 (DARK_FOREST): Adversarial pen testing + protocol fuzzing
#
# skunkBat observes the entire run and records metrics.
# Results feed back into the tunnel evolution validation targets.
#
# Usage:
#   bash security_validation.sh [--layer all|below|at|above|tiers|darkforest] [--tunnel-url URL]
#
# Requires: curl, openssl, nc, ss, python3
# Requires running: Full NUCLEUS composition (13 primals), JupyterHub

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

PROJECT_ROOT="$NUCLEUS_PROJECT_ROOT"
RESULTS_DIR="$PROJECT_ROOT/validation/security-$(date +%Y%m%d-%H%M%S)"

LAYER="all"
TUNNEL_URL=""
TARGET_HOST="127.0.0.1"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --layer)      LAYER="$2"; shift 2 ;;
        --tunnel-url) TUNNEL_URL="$2"; shift 2 ;;
        --target)     TARGET_HOST="$2"; shift 2 ;;
        --results)    RESULTS_DIR="$2"; shift 2 ;;
        *)            echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

mkdir -p "$RESULTS_DIR"

PASS=0; FAIL=0; WARN=0; INFO=0

log()  { echo "[$(date +%H:%M:%S)] $*" | tee -a "$RESULTS_DIR/security.log"; }
pass() { log "  [PASS] $*"; PASS=$((PASS + 1)); }
fail() { log "  [FAIL] $*"; FAIL=$((FAIL + 1)); }
warn() { log "  [WARN] $*"; WARN=$((WARN + 1)); }
info() { log "  [INFO] $*"; INFO=$((INFO + 1)); }

rpc_skunkbat() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 9140 2>/dev/null
}

log "═══════════════════════════════════════════════════════════"
log "  Security Validation Pipeline — Five-Layer Pen Testing"
log "  Target: $TARGET_HOST"
log "  Layer: $LAYER"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

# Notify skunkBat that a security scan is starting
SCAN_START=$(rpc_skunkbat '{"jsonrpc":"2.0","method":"security.scan","params":{},"id":1}')
log "  skunkBat scan baseline captured"

# ══════════════════════════════════════════════════════════════
# LAYER 1: BELOW THE PRIMALS — OS / Network
# ══════════════════════════════════════════════════════════════
if [[ "$LAYER" == "all" || "$LAYER" == "below" ]]; then
    log ""
    log "══ Layer 1: Below the Primals (OS / Network) ══"

    # 1a: Port exposure — what's listening?
    log ""
    log "── 1a: Port Exposure Scan ──"

    LISTENING=$(ss -tlnp 2>/dev/null | grep LISTEN)
    echo "$LISTENING" > "$RESULTS_DIR/listening_ports.txt"

    EXTERNAL_LISTENERS=$(echo "$LISTENING" | grep -v "127.0.0.1" | grep -v "::1" | grep -v "\[::1\]" || true)
    if [[ -n "$EXTERNAL_LISTENERS" ]]; then
        EXTERNAL_COUNT=$(echo "$EXTERNAL_LISTENERS" | wc -l)
        warn "Found $EXTERNAL_COUNT non-localhost listeners:"
        echo "$EXTERNAL_LISTENERS" | while read -r line; do
            info "  $line"
        done
    else
        pass "No non-localhost listeners — all services bound to 127.0.0.1"
    fi

    # Verify primal ports are NOT externally exposed
    for port in "${ALL_PRIMAL_PORTS_LIST[@]}"; do
        bind=$(echo "$LISTENING" | grep ":$port " | head -1)
        if echo "$bind" | grep -q "0.0.0.0" 2>/dev/null; then
            fail "Port $port bound to 0.0.0.0 (externally exposed)"
        elif echo "$bind" | grep -q "127.0.0.1" 2>/dev/null; then
            pass "Port $port bound to 127.0.0.1 only"
        elif [[ -n "$bind" ]]; then
            info "Port $port: $bind"
        fi
    done

    # JupyterHub binding
    HUB_BIND=$(echo "$LISTENING" | grep ":8000 " | head -1)
    if echo "$HUB_BIND" | grep -q "127.0.0.1" 2>/dev/null; then
        pass "JupyterHub (8000) bound to 127.0.0.1 — tunnel-only access"
    elif echo "$HUB_BIND" | grep -q "0.0.0.0" 2>/dev/null; then
        fail "JupyterHub (8000) bound to 0.0.0.0 — directly exposed"
    fi

    # 1b: Unnecessary services
    log ""
    log "── 1b: Unnecessary Service Check ──"

    for svc in sshd apache2 nginx mysql postgres docker; do
        if pgrep -x "$svc" > /dev/null 2>&1; then
            if [[ "$svc" == "sshd" ]]; then
                info "sshd running (expected for remote management)"
            else
                warn "$svc running — verify this is intentional"
            fi
        fi
    done

    # 1c: Firewall status
    log ""
    log "── 1c: Firewall Status ──"

    if command -v ufw &>/dev/null; then
        UFW_STATUS=$(ufw status 2>/dev/null || echo "permission denied")
        if echo "$UFW_STATUS" | grep -q "Status: active"; then
            pass "UFW firewall active"
        else
            warn "UFW installed but not active: $UFW_STATUS"
        fi
    elif command -v iptables &>/dev/null; then
        RULES=$(iptables -L -n 2>/dev/null | wc -l || echo "0")
        if [[ "$RULES" -gt 5 ]]; then
            info "iptables has $RULES rules"
        else
            warn "iptables has minimal rules ($RULES lines)"
        fi
    else
        warn "No firewall detected"
    fi

    # 1d: File permissions on sensitive paths
    log ""
    log "── 1d: Sensitive File Permissions ──"

    for path in "$HOME/.config/biomeos/family" "$HOME/jupyterhub/jupyterhub_cookie_secret" "$HOME/jupyterhub/jupyterhub.sqlite"; do
        if [[ -e "$path" ]]; then
            perms=$(stat -c '%a' "$path" 2>/dev/null || stat -f '%OLp' "$path" 2>/dev/null)
            if [[ "$perms" == "600" || "$perms" == "700" ]]; then
                pass "$path: mode $perms (restricted)"
            elif [[ "$perms" == "644" || "$perms" == "755" ]]; then
                warn "$path: mode $perms (world-readable)"
            else
                info "$path: mode $perms"
            fi
        fi
    done
fi

# ══════════════════════════════════════════════════════════════
# LAYER 2: AT THE PRIMAL LAYER — API Security
# ══════════════════════════════════════════════════════════════
if [[ "$LAYER" == "all" || "$LAYER" == "at" ]]; then
    log ""
    log "══ Layer 2: At the Primal Layer (API Security) ══"

    # 2a: Unauthenticated access to primal APIs
    log ""
    log "── 2a: Unauthenticated API Probe ──"

    for pair in "beardog:9100" "toadstool:9400" "nestgate:9500" "rhizocrypt:9602" "loamspine:9700" "sweetgrass:9850" "skunkbat:9140"; do
        name="${pair%%:*}"
        port="${pair#*:}"

        # Probe health (should work — health is public)
        resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","id":1}\n' | nc -w 3 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
        if echo "$resp" | grep -q '"result"' 2>/dev/null; then
            info "$name health.liveness accessible (expected — public health endpoint)"
        fi

        # Probe sensitive method without auth
        case "$name" in
            nestgate)
                resp=$(printf '{"jsonrpc":"2.0","method":"storage.list","params":{"prefix":""},"id":1}\n' | nc -w 3 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
                if echo "$resp" | grep -q '"error"' 2>/dev/null; then
                    pass "$name storage.list rejects unauthenticated request"
                elif echo "$resp" | grep -q '"result"' 2>/dev/null; then
                    warn "$name storage.list accessible without auth"
                fi
                ;;
            rhizocrypt)
                resp=$(printf '{"jsonrpc":"2.0","method":"dag.session.list","params":{},"id":1}\n' | nc -w 3 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
                if echo "$resp" | grep -q '"result"' 2>/dev/null; then
                    info "$name dag.session.list accessible (BTSP enforced at connection level)"
                fi
                ;;
            sweetgrass)
                resp=$(printf '{"jsonrpc":"2.0","method":"braid.list","params":{},"id":1}\n' | nc -w 3 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
                if echo "$resp" | grep -q '"error"' 2>/dev/null; then
                    pass "$name braid.list rejects unauthenticated request"
                elif echo "$resp" | grep -q '"result"' 2>/dev/null; then
                    info "$name braid.list accessible (BTSP at transport layer)"
                fi
                ;;
        esac
    done

    # 2b: Input fuzzing — malformed JSON-RPC
    log ""
    log "── 2b: Input Fuzzing (Malformed Requests) ──"

    FUZZ_TARGETS="beardog:9100 toadstool:9400 nestgate:9500 skunkbat:9140"
    FUZZ_PAYLOADS=(
        'not json at all'
        '{"jsonrpc":"2.0"}'
        '{"jsonrpc":"2.0","method":"","id":1}'
        '{"jsonrpc":"2.0","method":"../../../etc/passwd","id":1}'
        '{"jsonrpc":"2.0","method":"health.liveness","params":"not-an-object","id":1}'
        '{"jsonrpc":"2.0","method":"health.liveness","id":null}'
        "{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"params\":{\"key\":\"$(python3 -c "print('A'*10000)")\"}, \"id\":1}"
    )

    for target in $FUZZ_TARGETS; do
        name="${target%%:*}"
        port="${target#*:}"
        crashes=0

        for payload in "${FUZZ_PAYLOADS[@]}"; do
            resp=$(printf '%s\n' "$payload" | nc -w 2 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
            if [[ -z "$resp" ]]; then
                # Connection closed without response — check if primal still alive
                alive_check=$(printf '{"jsonrpc":"2.0","method":"health.liveness","id":99}\n' | nc -w 2 "$TARGET_HOST" "$port" 2>/dev/null) || alive_check=""
                if ! echo "$alive_check" | grep -q '"result"' 2>/dev/null; then
                    crashes=$((crashes + 1))
                fi
            fi
        done

        if [[ $crashes -eq 0 ]]; then
            pass "$name survived all ${#FUZZ_PAYLOADS[@]} fuzz payloads without crash"
        else
            fail "$name crashed on $crashes/${#FUZZ_PAYLOADS[@]} fuzz payloads"
        fi
    done

    # 2c: Method enumeration — try to discover hidden methods
    log ""
    log "── 2c: Method Enumeration ──"

    HIDDEN_METHODS=("admin.shutdown" "system.exec" "debug.dump" "internal.config" "shell.exec" "eval" "rpc.discover")
    for target in "beardog:9100" "toadstool:9400" "nestgate:9500"; do
        name="${target%%:*}"
        port="${target#*:}"
        found=0
        for method in "${HIDDEN_METHODS[@]}"; do
            resp=$(printf '{"jsonrpc":"2.0","method":"%s","params":{},"id":1}\n' "$method" | nc -w 2 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
            if echo "$resp" | grep -q '"result"' 2>/dev/null; then
                fail "$name exposes hidden method: $method"
                found=$((found + 1))
            fi
        done
        if [[ $found -eq 0 ]]; then
            pass "$name rejects all ${#HIDDEN_METHODS[@]} suspicious method probes"
        fi
    done

    # 2d: BTSP enforcement check
    log ""
    log "── 2d: BTSP Enforcement ──"

    for pair in "sweetgrass:9850" "rhizocrypt:9601"; do
        name="${pair%%:*}"
        port="${pair#*:}"
        resp=$(printf 'PLAINTEXT PROBE\n' | nc -w 2 "$TARGET_HOST" "$port" 2>/dev/null) || resp=""
        if [[ -z "$resp" ]] || echo "$resp" | grep -qi "btsp\|reject\|error\|unauthorized" 2>/dev/null; then
            pass "$name (port $port) rejects plaintext connection"
        else
            warn "$name (port $port) responded to plaintext: ${resp:0:80}"
        fi
    done
fi

# ══════════════════════════════════════════════════════════════
# LAYER 3: ABOVE THE PRIMALS — Application Security
# ══════════════════════════════════════════════════════════════
if [[ "$LAYER" == "all" || "$LAYER" == "above" ]]; then
    log ""
    log "══ Layer 3: Above the Primals (Application Security) ══"

    # 3a: JupyterHub security headers
    log ""
    log "── 3a: JupyterHub Security Headers ──"

    HEADERS=$(curl -sf -D - "http://127.0.0.1:8000/hub/login" -o /dev/null 2>/dev/null)
    echo "$HEADERS" > "$RESULTS_DIR/hub_headers.txt"

    for header in "X-Frame-Options" "X-Content-Type-Options" "Content-Security-Policy" "X-XSS-Protection"; do
        if echo "$HEADERS" | grep -qi "$header"; then
            HVAL=$(echo "$HEADERS" | grep -i "$header" | head -1 | tr -d '\r')
            pass "JupyterHub sends $HVAL"
        else
            warn "JupyterHub missing header: $header"
        fi
    done

    # Server header disclosure
    SERVER_HEADER=$(echo "$HEADERS" | grep -i "^server:" | tr -d '\r' | sed 's/^[Ss]erver: *//')
    if [[ -z "$SERVER_HEADER" || "$SERVER_HEADER" == " " ]]; then
        pass "Server header suppressed (dark forest)"
    elif echo "$SERVER_HEADER" | grep -qi "tornado\|python\|jupyter\|nginx\|apache"; then
        warn "Server header leaks implementation: $SERVER_HEADER"
    else
        pass "Server header present but non-identifying: $SERVER_HEADER"
    fi

    # 3b: JupyterHub authentication enforcement
    log ""
    log "── 3b: Authentication Enforcement ──"

    # Try to access user API without auth
    UNAUTH_API=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:8000/hub/api/users" 2>/dev/null || echo "000")
    if [[ "$UNAUTH_API" == "403" || "$UNAUTH_API" == "401" ]]; then
        pass "JupyterHub /hub/api/users requires auth (HTTP $UNAUTH_API)"
    else
        fail "JupyterHub /hub/api/users accessible without auth (HTTP $UNAUTH_API)"
    fi

    # Try to access spawn endpoint without auth
    UNAUTH_SPAWN=$(curl -s -o /dev/null -w "%{http_code}" -X POST "http://127.0.0.1:8000/hub/api/users/testuser/server" 2>/dev/null || echo "000")
    if [[ "$UNAUTH_SPAWN" == "403" || "$UNAUTH_SPAWN" == "401" || "$UNAUTH_SPAWN" == "302" ]]; then
        pass "JupyterHub spawn endpoint requires auth (HTTP $UNAUTH_SPAWN)"
    else
        warn "JupyterHub spawn endpoint returned HTTP $UNAUTH_SPAWN"
    fi

    # 3c: Path traversal probes
    log ""
    log "── 3c: Path Traversal Probes ──"

    TRAVERSAL_PATHS=(
        "/hub/../../../etc/passwd"
        "/hub/%2e%2e/%2e%2e/etc/passwd"
        "/hub/login?next=//evil.com"
        "/hub/api/../../../etc/shadow"
    )

    for path in "${TRAVERSAL_PATHS[@]}"; do
        resp=$(curl -sf -o /dev/null -w "%{http_code}" "http://127.0.0.1:8000$path" --max-time 5 2>/dev/null || echo "000")
        if [[ "$resp" == "200" ]]; then
            # Check if we actually got /etc/passwd content
            content=$(curl -sf "http://127.0.0.1:8000$path" --max-time 5 2>/dev/null | head -1)
            if echo "$content" | grep -q "root:" 2>/dev/null; then
                fail "Path traversal succeeded: $path"
            else
                pass "Path $path returned 200 but no sensitive content"
            fi
        else
            pass "Path traversal blocked: $path (HTTP $resp)"
        fi
    done

    # 3d: Tunnel security (if URL provided)
    if [[ -n "$TUNNEL_URL" ]]; then
        log ""
        log "── 3d: Tunnel Security ──"

        # TLS version
        TLS_INFO=$(curl -sf -v "$TUNNEL_URL/hub/api/" 2>&1 | grep -i "SSL connection\|TLS" | head -3)
        if [[ -n "$TLS_INFO" ]]; then
            if echo "$TLS_INFO" | grep -qi "TLSv1.3\|TLSv1.2"; then
                pass "Tunnel uses modern TLS: $(echo "$TLS_INFO" | head -1)"
            else
                warn "Tunnel TLS: $TLS_INFO"
            fi
        fi

        # Certificate check
        CERT_INFO=$(echo | openssl s_client -connect "${TUNNEL_URL#https://}:443" -servername "${TUNNEL_URL#https://}" 2>/dev/null | openssl x509 -noout -dates -issuer 2>/dev/null)
        if [[ -n "$CERT_INFO" ]]; then
            pass "Tunnel TLS certificate valid"
            info "  $CERT_INFO"
        fi

        # HSTS header
        TUNNEL_HEADERS=$(curl -sf -D - "$TUNNEL_URL/hub/api/" -o /dev/null 2>/dev/null)
        if echo "$TUNNEL_HEADERS" | grep -qi "strict-transport-security"; then
            pass "Tunnel sends HSTS header"
        else
            warn "Tunnel missing HSTS header"
        fi
    fi
fi

# ══════════════════════════════════════════════════════════════
# LAYER 4: TIER ENFORCEMENT — ABG User Boundaries
# ══════════════════════════════════════════════════════════════
if [[ "$LAYER" == "all" || "$LAYER" == "tiers" ]]; then
    log ""
    log "══ Layer 4: ABG Tier Enforcement ══"

    # 4a: OS-level tier probes
    log ""
    log "── 4a: OS-Level Tier Enforcement ──"

    TIER_SCRIPT="$SCRIPT_DIR/tier_enforcement_test.sh"
    if [[ -x "$TIER_SCRIPT" ]]; then
        TIER_OS_OUT="$RESULTS_DIR/tier_os_results.txt"
        bash "$TIER_SCRIPT" 2>&1 | tee "$TIER_OS_OUT" || true

        TIER_OS_PASS=$(grep -c '^PASS|' "$TIER_OS_OUT" 2>/dev/null || true)
        TIER_OS_FAIL=$(grep -c '^FAIL|' "$TIER_OS_OUT" 2>/dev/null || true)
        TIER_OS_GAP=$(grep -c '^KNOWN_GAP|' "$TIER_OS_OUT" 2>/dev/null || true)
        : "${TIER_OS_PASS:=0}" "${TIER_OS_FAIL:=0}" "${TIER_OS_GAP:=0}"

        PASS=$((PASS + TIER_OS_PASS))
        FAIL=$((FAIL + TIER_OS_FAIL))

        if [[ "$TIER_OS_FAIL" -eq 0 ]]; then
            pass "OS-level tier enforcement: $TIER_OS_PASS assertions pass ($TIER_OS_GAP known gaps)"
        else
            fail "OS-level tier enforcement: $TIER_OS_FAIL failures out of $((TIER_OS_PASS + TIER_OS_FAIL)) assertions"
        fi
    else
        warn "tier_enforcement_test.sh not found at $TIER_SCRIPT"
    fi

    # 4b: JupyterHub API tier probes
    log ""
    log "── 4b: JupyterHub API Tier Enforcement ──"

    TIER_API_SCRIPT="$SCRIPT_DIR/jupyterhub_tier_test.py"
    if [[ -f "$TIER_API_SCRIPT" ]]; then
        TIER_API_OUT="$RESULTS_DIR/tier_api_results.txt"
        python3 "$TIER_API_SCRIPT" 2>&1 | tee "$TIER_API_OUT" || true

        TIER_API_PASS=$(grep -c '^PASS|' "$TIER_API_OUT" 2>/dev/null || true)
        TIER_API_FAIL=$(grep -c '^FAIL|' "$TIER_API_OUT" 2>/dev/null || true)
        TIER_API_SKIP=$(grep -c '^SKIP|' "$TIER_API_OUT" 2>/dev/null || true)
        : "${TIER_API_PASS:=0}" "${TIER_API_FAIL:=0}" "${TIER_API_SKIP:=0}"

        PASS=$((PASS + TIER_API_PASS))
        FAIL=$((FAIL + TIER_API_FAIL))

        if [[ "$TIER_API_FAIL" -eq 0 ]]; then
            pass "JupyterHub API tier enforcement: $TIER_API_PASS assertions pass ($TIER_API_SKIP skipped)"
        else
            fail "JupyterHub API tier enforcement: $TIER_API_FAIL failures out of $((TIER_API_PASS + TIER_API_FAIL)) assertions"
        fi
    else
        warn "jupyterhub_tier_test.py not found at $TIER_API_SCRIPT"
    fi
fi

# ══════════════════════════════════════════════════════════════
# LAYER 5: DARK FOREST — Adversarial Pen Test + Protocol Fuzz
# ══════════════════════════════════════════════════════════════
if [[ "$LAYER" == "all" || "$LAYER" == "darkforest" ]]; then
    log ""
    log "══ Layer 5: Dark Forest ══"

    # 5a: Adversarial pen test
    log ""
    log "── 5a: Dark Forest Pen Test ──"

    PENTEST_SCRIPT="$SCRIPT_DIR/darkforest_pentest.sh"
    if [[ -x "$PENTEST_SCRIPT" ]]; then
        PENTEST_OUT="$RESULTS_DIR/darkforest_pentest.txt"
        TUNNEL_ARG=""
        [[ -n "$TUNNEL_URL" ]] && TUNNEL_ARG="--tunnel-url $TUNNEL_URL"
        bash "$PENTEST_SCRIPT" --suite all $TUNNEL_ARG 2>&1 | tee "$PENTEST_OUT" || true

        DF_PEN_PASS=$(grep -c '^PASS|' "$PENTEST_OUT" 2>/dev/null || true)
        DF_PEN_FAIL=$(grep -c '^FAIL|' "$PENTEST_OUT" 2>/dev/null || true)
        DF_PEN_GAP=$(grep -c '^KNOWN_GAP|' "$PENTEST_OUT" 2>/dev/null || true)
        DF_PEN_DF=$(grep -c '^DARK_FOREST|' "$PENTEST_OUT" 2>/dev/null || true)
        : "${DF_PEN_PASS:=0}" "${DF_PEN_FAIL:=0}" "${DF_PEN_GAP:=0}" "${DF_PEN_DF:=0}"

        PASS=$((PASS + DF_PEN_PASS))
        FAIL=$((FAIL + DF_PEN_FAIL))

        if [[ "$DF_PEN_FAIL" -eq 0 ]]; then
            pass "Dark Forest pen test: $DF_PEN_PASS pass, $DF_PEN_GAP gaps, $DF_PEN_DF dark forest findings"
        else
            fail "Dark Forest pen test: $DF_PEN_FAIL failures out of $((DF_PEN_PASS + DF_PEN_FAIL)) assertions"
        fi
    else
        warn "darkforest_pentest.sh not found at $PENTEST_SCRIPT"
    fi

    # 5b: Protocol fuzz
    log ""
    log "── 5b: Protocol Fuzz ──"

    FUZZ_SCRIPT="$SCRIPT_DIR/darkforest_fuzz.py"
    if [[ -f "$FUZZ_SCRIPT" ]]; then
        FUZZ_OUT="$RESULTS_DIR/darkforest_fuzz.txt"
        timeout 900 python3 -u "$FUZZ_SCRIPT" --rounds 2 2>&1 | tee "$FUZZ_OUT" || true

        DF_FUZZ_PASS=$(grep -c '^PASS|' "$FUZZ_OUT" 2>/dev/null || true)
        DF_FUZZ_FAIL=$(grep -c '^FAIL|' "$FUZZ_OUT" 2>/dev/null || true)
        DF_FUZZ_DF=$(grep -c '^DARK_FOREST|' "$FUZZ_OUT" 2>/dev/null || true)
        : "${DF_FUZZ_PASS:=0}" "${DF_FUZZ_FAIL:=0}" "${DF_FUZZ_DF:=0}"

        PASS=$((PASS + DF_FUZZ_PASS))
        FAIL=$((FAIL + DF_FUZZ_FAIL))

        if [[ "$DF_FUZZ_FAIL" -eq 0 ]]; then
            pass "Protocol fuzz: $DF_FUZZ_PASS pass, $DF_FUZZ_DF dark forest findings"
        else
            fail "Protocol fuzz: $DF_FUZZ_FAIL failures out of $((DF_FUZZ_PASS + DF_FUZZ_FAIL)) assertions"
        fi
    else
        warn "darkforest_fuzz.py not found at $FUZZ_SCRIPT"
    fi
fi

# ══════════════════════════════════════════════════════════════
# SKUNKBAT METRICS — Post-scan
# ══════════════════════════════════════════════════════════════
log ""
log "══ skunkBat Observation ══"

SCAN_END=$(rpc_skunkbat '{"jsonrpc":"2.0","method":"security.metrics","params":{},"id":2}')
if [[ -n "$SCAN_END" ]]; then
    echo "$SCAN_END" | python3 -m json.tool > "$RESULTS_DIR/skunkbat_metrics.json" 2>/dev/null

    THREATS=$(echo "$SCAN_END" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['threats_detected'])" 2>/dev/null || echo "?")
    QUARANTINED=$(echo "$SCAN_END" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['connections_quarantined'])" 2>/dev/null || echo "?")
    ALERTS=$(echo "$SCAN_END" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['alerts_sent'])" 2>/dev/null || echo "?")

    info "skunkBat metrics after scan:"
    info "  Threats detected: $THREATS"
    info "  Connections quarantined: $QUARANTINED"
    info "  Alerts sent: $ALERTS"
else
    warn "Could not reach skunkBat for post-scan metrics"
fi

DETECT_END=$(rpc_skunkbat '{"jsonrpc":"2.0","method":"security.detect","params":{},"id":3}')
echo "$DETECT_END" | python3 -m json.tool > "$RESULTS_DIR/skunkbat_detections.json" 2>/dev/null

# ══════════════════════════════════════════════════════════════
# RESULTS
# ══════════════════════════════════════════════════════════════
log ""
log "═══════════════════════════════════════════════════════════"
log "  Security Validation Complete"
log "  PASS: $PASS"
log "  FAIL: $FAIL"
log "  WARN: $WARN"
log "  INFO: $INFO"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

cat > "$RESULTS_DIR/SECURITY_RESULTS.md" << EOF
# Security Validation — $(date -Iseconds)

**Target**: $TARGET_HOST
**Layer**: $LAYER
**Composition**: 13 primals (full NUCLEUS)
$(if [[ -n "$TUNNEL_URL" ]]; then echo "**Tunnel**: $TUNNEL_URL"; fi)

## Summary

| Metric | Count |
|--------|-------|
| PASS | $PASS |
| FAIL | $FAIL |
| WARN | $WARN |
| INFO | $INFO |

## skunkBat Observations

- Threats detected: $THREATS
- Connections quarantined: $QUARANTINED
- Alerts sent: $ALERTS

## Layer Coverage

| Layer | Scope | Tests |
|-------|-------|-------|
| Below (OS/Network) | Port exposure, firewall, file permissions | $(if [[ "$LAYER" == "all" || "$LAYER" == "below" ]]; then echo "Included"; else echo "Skipped"; fi) |
| At (Primal APIs) | Auth probes, input fuzzing, method enumeration, BTSP | $(if [[ "$LAYER" == "all" || "$LAYER" == "at" ]]; then echo "Included"; else echo "Skipped"; fi) |
| Above (Application) | JupyterHub headers, auth, path traversal, tunnel TLS | $(if [[ "$LAYER" == "all" || "$LAYER" == "above" ]]; then echo "Included"; else echo "Skipped"; fi) |
| Tiers (ABG Enforcement) | Filesystem, network, process, JupyterHub API per tier | $(if [[ "$LAYER" == "all" || "$LAYER" == "tiers" ]]; then echo "Included"; else echo "Skipped"; fi) |
| Dark Forest | Adversarial pen test, protocol fuzz, timing analysis | $(if [[ "$LAYER" == "all" || "$LAYER" == "darkforest" ]]; then echo "Included"; else echo "Skipped"; fi) |

## Environment

- System: $(uname -n) ($(uname -m))
- Kernel: $(uname -r)
- Date: $(date -Iseconds)

## Files

- \`security.log\` — full test output
- \`listening_ports.txt\` — all listening sockets
- \`hub_headers.txt\` — JupyterHub response headers
- \`skunkbat_metrics.json\` — skunkBat post-scan metrics
- \`skunkbat_detections.json\` — skunkBat detections during scan
- \`tier_os_results.txt\` — OS-level tier enforcement test output
- \`tier_api_results.txt\` — JupyterHub API tier enforcement test output
- \`darkforest_pentest.txt\` — adversarial pen test output
- \`darkforest_fuzz.txt\` — protocol fuzz output
EOF

log "  Report: $RESULTS_DIR/SECURITY_RESULTS.md"

if [[ $FAIL -gt 0 ]]; then
    exit 1
fi
exit 0
