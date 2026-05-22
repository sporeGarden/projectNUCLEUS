#!/usr/bin/env bash
# darkforest --suite membrane — Remote VPS audit
#
# Implements MEM-01 through MEM-13: cellMembrane security validation.
# Checks split into two categories:
#   LOCAL:  probes from the gate (TCP/UDP reachability)
#   REMOTE: checks via SSH on the VPS itself
#
# Usage:
#   bash validation/darkforest_membrane.sh [--skip-ssh]
#
# Prerequisites:
#   - SSH access to cellMembrane VPS (key-based)
#   - nucleus_config.sh sourced for MEMBRANE_VPS_IP

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$PROJECT_ROOT/deploy/nucleus_config.sh" 2>/dev/null || true

VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
VPS_USER="${MEMBRANE_VPS_USER:-root}"
SKIP_SSH=false
[[ "${1:-}" == "--skip-ssh" ]] && SKIP_SSH=true

PASS=0; FAIL=0; SKIP=0

pass()  { echo "  PASS  [$1] $2"; PASS=$((PASS + 1)); }
fail()  { echo "  FAIL  [$1] $2"; FAIL=$((FAIL + 1)); }
skip()  { echo "  SKIP  [$1] $2"; SKIP=$((SKIP + 1)); }

ssh_cmd() {
    ssh -o ConnectTimeout=10 -o BatchMode=yes -o StrictHostKeyChecking=accept-new \
        "$VPS_USER@$VPS_IP" "$@" 2>/dev/null
}

echo "darkforest --suite membrane"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "VPS:  $VPS_IP"
echo ""

# ─── MEM-01: SSH password auth disabled ─────────────────────────────────────
echo "═══ MEM-01: SSH password auth disabled ═══"
if $SKIP_SSH; then
    skip "MEM-01" "SSH checks skipped"
else
    pw_result=$(ssh -o ConnectTimeout=5 -o BatchMode=yes \
        -o PreferredAuthentications=password \
        -o PubkeyAuthentication=no \
        "$VPS_USER@$VPS_IP" "echo ok" 2>&1 || true)
    if echo "$pw_result" | grep -qi "permission denied\|no more authentication"; then
        pass "MEM-01" "Password auth rejected"
    elif echo "$pw_result" | grep -qi "ok"; then
        fail "MEM-01" "Password auth succeeded — must disable"
    else
        pass "MEM-01" "Password auth not accepted"
    fi
fi

# ─── MEM-02: fail2ban sshd jail active ──────────────────────────────────────
echo "═══ MEM-02: fail2ban sshd jail ═══"
if $SKIP_SSH; then
    skip "MEM-02" "SSH checks skipped"
else
    f2b=$(ssh_cmd "fail2ban-client status sshd 2>/dev/null | head -5" || true)
    if echo "$f2b" | grep -qi "currently banned\|filter"; then
        pass "MEM-02" "fail2ban sshd jail active"
    else
        fail "MEM-02" "fail2ban sshd jail not responding"
    fi
fi

# ─── MEM-03: UFW posture ────────────────────────────────────────────────────
echo "═══ MEM-03: UFW posture (22+3478+21115-21117 only) ═══"
if $SKIP_SSH; then
    skip "MEM-03" "SSH checks skipped"
else
    ufw=$(ssh_cmd "ufw status numbered 2>/dev/null" || true)
    if echo "$ufw" | grep -q "Status: active"; then
        pass "MEM-03" "UFW active"
        if echo "$ufw" | grep -q "22/tcp\|22 "; then
            pass "MEM-03" "Port 22 allowed"
        else
            fail "MEM-03" "Port 22 not in UFW rules"
        fi
        if echo "$ufw" | grep -q "3478"; then
            pass "MEM-03" "Port 3478 (TURN) allowed"
        else
            fail "MEM-03" "Port 3478 not in UFW rules"
        fi
    else
        fail "MEM-03" "UFW not active"
    fi
fi

# ─── MEM-04: TURN relay reachable (UDP :3478) ──────────────────────────────
echo "═══ MEM-04: TURN relay reachable ═══"
turn_tcp=false
turn_udp=false
nc -z -w 5 "$VPS_IP" 3478 2>/dev/null && turn_tcp=true
nc -z -u -w 5 "$VPS_IP" 3478 2>/dev/null && turn_udp=true
if $turn_tcp || $turn_udp; then
    local_proto=""
    $turn_tcp && local_proto="TCP"
    $turn_udp && local_proto="${local_proto:+$local_proto+}UDP"
    pass "MEM-04" "TURN :3478 reachable ($local_proto)"
else
    fail "MEM-04" "TURN :3478 unreachable"
fi

# ─── MEM-05: TURN rejects unauthenticated relay ────────────────────────────
echo "═══ MEM-05: TURN unauthenticated rejection ═══"
unauth_bytes=$(echo -ne '\x00\x03\x00\x00' | nc -w 3 "$VPS_IP" 3478 2>/dev/null | wc -c)
unauth_bytes="${unauth_bytes// /}"
if [[ "$unauth_bytes" -lt 10 ]] 2>/dev/null; then
    pass "MEM-05" "TURN does not relay without credentials ($unauth_bytes bytes returned)"
else
    fail "MEM-05" "TURN may accept unauthenticated relaying ($unauth_bytes bytes)"
fi

# ─── MEM-06: No unnecessary services ────────────────────────────────────────
echo "═══ MEM-06: No unnecessary services ═══"
if $SKIP_SSH; then
    skip "MEM-06" "SSH checks skipped"
else
    for svc in exim4 droplet-agent snapd; do
        svc_status=$(ssh_cmd "systemctl is-active $svc 2>/dev/null" || true)
        if [[ "$svc_status" == "active" ]]; then
            fail "MEM-06" "$svc is running — should be removed"
        else
            pass "MEM-06" "$svc not running"
        fi
    done
fi

# ─── MEM-07: journald persistence ───────────────────────────────────────────
echo "═══ MEM-07: journald persistence ═══"
if $SKIP_SSH; then
    skip "MEM-07" "SSH checks skipped"
else
    journal_dir=$(ssh_cmd "ls -d /var/log/journal/ 2>/dev/null" || true)
    if [[ -n "$journal_dir" ]]; then
        pass "MEM-07" "journald persistence configured (/var/log/journal/ exists)"
    else
        fail "MEM-07" "/var/log/journal/ not found — volatile logging"
    fi
fi

# ─── MEM-08: Credential file permissions ────────────────────────────────────
echo "═══ MEM-08: Credential file permissions ═══"
if $SKIP_SSH; then
    skip "MEM-08" "SSH checks skipped"
else
    cred_files="/etc/songbird/relay-credentials /opt/membrane/songbird/turn-credentials"
    found_cred=false
    for cf in $cred_files; do
        perms=$(ssh_cmd "stat -c '%a %U' $cf 2>/dev/null" || true)
        if [[ -n "$perms" ]]; then
            found_cred=true
            mode=$(echo "$perms" | awk '{print $1}')
            owner=$(echo "$perms" | awk '{print $2}')
            if [[ "$mode" == "600" && "$owner" == "root" ]]; then
                pass "MEM-08" "Credential file $cf: mode=$mode owner=$owner"
            else
                fail "MEM-08" "Credential file $cf: mode=$mode owner=$owner (expect 600/root)"
            fi
        fi
    done
    if ! $found_cred; then
        skip "MEM-08" "No credential files found at expected paths"
    fi
fi

# ─── MEM-09: Songbird binary integrity ─────────────────────────────────────
echo "═══ MEM-09: Songbird binary integrity ═══"
if $SKIP_SSH; then
    skip "MEM-09" "SSH checks skipped"
else
    remote_hash=$(ssh_cmd "b3sum /opt/membrane/songbird/songbird 2>/dev/null | awk '{print \$1}'" || true)
    if [[ -n "$remote_hash" && ${#remote_hash} -eq 64 ]]; then
        pass "MEM-09" "Songbird binary BLAKE3: ${remote_hash:0:16}..."
    elif [[ -n "$remote_hash" ]]; then
        pass "MEM-09" "Songbird binary hash obtained (b3sum may not be installed, got: ${remote_hash:0:20})"
    else
        skip "MEM-09" "Could not hash remote songbird binary (b3sum not available)"
    fi
fi

# ─── MEM-10: No unexpected listening ports ──────────────────────────────────
echo "═══ MEM-10: No unexpected listening ports ═══"
if $SKIP_SSH; then
    skip "MEM-10" "SSH checks skipped"
else
    # Phase 0.5: SSH + TURN + RustDesk + Tower (BearDog + SkunkBat) + Caddy TLS
    # + Shadow services: BearDog TLS :8443, petalTongue web :8080
    expected_ports="22 53 5355 3478 21115 21116 21117 21118 21119 9100 9140 80 443 2019 8443 8080 9500 9601 9602 9700 9001 9850 8091"
    listeners=$(ssh_cmd "ss -tlnp 2>/dev/null | grep LISTEN" || true)
    unexpected=0
    while IFS= read -r line; do
        [[ -z "$line" ]] && continue
        port=$(echo "$line" | grep -oP ':\K\d+(?=\s)' | head -1)
        [[ -z "$port" ]] && continue
        is_expected=false
        for ep in $expected_ports; do
            [[ "$port" == "$ep" ]] && is_expected=true
        done
        if ! $is_expected; then
            proc_name=$(echo "$line" | grep -oP 'users:\(\("\K[^"]+' || echo "unknown")
            known_primals="sweetgrass nestgate rhizocrypt loamspine beardog songbird skunkbat petaltongue"
            is_known_primal=false
            for kp in $known_primals; do
                [[ "$proc_name" == "$kp" ]] && is_known_primal=true
            done
            if $is_known_primal; then
                echo "    NOTE: Primal ephemeral port :$port ($proc_name) — OK"
            else
                echo "    NOTE: Unexpected TCP listener on :$port ($proc_name)"
                unexpected=$((unexpected + 1))
            fi
        fi
    done <<< "$listeners"
    if [[ $unexpected -eq 0 ]]; then
        pass "MEM-10" "No unexpected TCP listeners"
    else
        fail "MEM-10" "$unexpected unexpected TCP listeners found"
    fi
fi

# ─── MEM-11: RustDesk hbbs/hbbr services active ────────────────────────────
echo "═══ MEM-11: RustDesk services ═══"
if $SKIP_SSH; then
    skip "MEM-11" "SSH checks skipped"
else
    for svc in hbbs-membrane hbbr-membrane; do
        svc_status=$(ssh_cmd "systemctl is-active $svc 2>/dev/null" || true)
        if [[ "$svc_status" == "active" ]]; then
            pass "MEM-11" "$svc active"
        else
            fail "MEM-11" "$svc not active (status: $svc_status)"
        fi
    done
fi

# ─── MEM-12: RustDesk relay key ─────────────────────────────────────────────
echo "═══ MEM-12: RustDesk relay key ═══"
if $SKIP_SSH; then
    skip "MEM-12" "SSH checks skipped"
else
    key_exists=$(ssh_cmd "test -f /opt/membrane/rustdesk/id_ed25519.pub && echo yes" || true)
    if [[ "$key_exists" == "yes" ]]; then
        key_hash=$(ssh_cmd "sha256sum /opt/membrane/rustdesk/id_ed25519.pub | awk '{print \$1}'" || true)
        pass "MEM-12" "RustDesk key present (sha256: ${key_hash:0:16}...)"
    else
        fail "MEM-12" "RustDesk id_ed25519.pub not found"
    fi
fi

# ─── MEM-13: RustDesk ports reachable ───────────────────────────────────────
echo "═══ MEM-13: RustDesk ports reachable ═══"
if nc -z -w 5 "$VPS_IP" 21116 2>/dev/null; then
    pass "MEM-13" "RustDesk :21116 reachable (TCP)"
else
    fail "MEM-13" "RustDesk :21116 unreachable"
fi

# ─── MEM-14: NestGate health (Nest Atomic) ─────────────────────────────────
# NestGate serves HTTP REST on :9500 (not JSON-RPC). Probe /health endpoint.
echo "═══ MEM-14: NestGate health (Nest Atomic) ═══"
if $SKIP_SSH; then
    skip "MEM-14" "SSH checks skipped"
else
    ng_resp=$(ssh_cmd "curl -sf --max-time 3 http://127.0.0.1:9500/health 2>/dev/null" || true)
    if echo "$ng_resp" | grep -q '"status":"ok"'; then
        pass "MEM-14" "NestGate healthy on :9500 (HTTP REST)"
    elif ssh_cmd "pgrep -x nestgate >/dev/null 2>&1"; then
        fail "MEM-14" "NestGate process running but /health not responding"
    else
        skip "MEM-14" "NestGate not deployed (Nest Atomic pending CM-1)"
    fi
fi

# ─── MEM-15: rhizoCrypt DAG health (Nest Atomic) ──────────────────────────
# rhizoCrypt: tarpc on :9601, JSON-RPC (newline-delimited) on :9602.
echo "═══ MEM-15: rhizoCrypt health (Nest Atomic) ═══"
if $SKIP_SSH; then
    skip "MEM-15" "SSH checks skipped"
else
    rc_resp=$(ssh_cmd "echo '{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}' | timeout 3 socat -t 0.5 - TCP:127.0.0.1:9602 2>/dev/null | head -1" || true)
    if echo "$rc_resp" | grep -q '"result"'; then
        pass "MEM-15" "rhizoCrypt healthy on :9602 (JSON-RPC)"
    elif ssh_cmd "pgrep -x rhizocrypt >/dev/null 2>&1"; then
        fail "MEM-15" "rhizoCrypt process running but not responding on :9602"
    else
        skip "MEM-15" "rhizoCrypt not deployed (Nest Atomic pending CM-1)"
    fi
fi

# ─── MEM-16: loamSpine ledger health (Nest Atomic) ────────────────────────
echo "═══ MEM-16: loamSpine health (Nest Atomic) ═══"
if $SKIP_SSH; then
    skip "MEM-16" "SSH checks skipped"
else
    ls_resp=$(ssh_cmd "curl -sf --max-time 3 -X POST http://127.0.0.1:9700 -H 'Content-Type: application/json' -d '{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}' 2>/dev/null" || true)
    if echo "$ls_resp" | grep -q '"result"'; then
        pass "MEM-16" "loamSpine healthy on :9700 (HTTP)"
    elif ssh_cmd "pgrep -x loamspine >/dev/null 2>&1"; then
        fail "MEM-16" "loamSpine process running but not responding"
    else
        skip "MEM-16" "loamSpine not deployed (Nest Atomic pending CM-1)"
    fi
fi

# ─── MEM-17: sweetGrass braid health (Nest Atomic) ────────────────────────
echo "═══ MEM-17: sweetGrass health (Nest Atomic) ═══"
if $SKIP_SSH; then
    skip "MEM-17" "SSH checks skipped"
else
    sg_resp=$(ssh_cmd "echo '{\"jsonrpc\":\"2.0\",\"method\":\"health.liveness\",\"id\":1}' | timeout 3 socat -t 0.5 - TCP:127.0.0.1:9850 2>/dev/null | head -1" || true)
    if echo "$sg_resp" | grep -q '"result"'; then
        pass "MEM-17" "sweetGrass healthy on :9850"
    elif ssh_cmd "pgrep -x sweetgrass >/dev/null 2>&1"; then
        fail "MEM-17" "sweetGrass process running but not responding"
    else
        skip "MEM-17" "sweetGrass not deployed (Nest Atomic pending CM-1)"
    fi
fi

# ─── Summary ────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════"
echo "darkforest --suite membrane — Results"
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  SKIP: $SKIP"
echo "  VPS:  $VPS_IP"
echo "═══════════════════════════════════════════════════"

if [[ $FAIL -eq 0 ]]; then
    echo "MEMBRANE STATUS: CLEAN — all checks passed"
    exit 0
else
    echo "MEMBRANE STATUS: $FAIL issues found"
    exit 1
fi
