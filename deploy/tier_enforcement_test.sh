#!/usr/bin/env bash
# Tier Enforcement Test Suite — Validates ABG access boundaries
#
# Runs as root. Probes each tier via sudo -u to verify OS-level enforcement.
# Every assertion is independently testable and machine-readable.
#
# Usage:
#   sudo bash tier_enforcement_test.sh [--tier compute|reviewer|observer|admin|all]
#
# Output format per assertion:
#   PASS|<tier>|<capability>|<detail>
#   FAIL|<tier>|<capability>|<detail>
#
# Exit code: number of FAILs (0 = all pass)

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

TIER_FILTER="${1:-all}"
[[ "$TIER_FILTER" == "--tier" ]] && TIER_FILTER="${2:-all}"

PASS_COUNT=0
FAIL_COUNT=0
KNOWN_GAP_COUNT=0

# --- Test users (one per tier) ---
declare -A TIER_USERS=(
    [admin]=kmok
    [compute]=tamison
    [reviewer]=abgreviewer
    [observer]=abg-test
)

pass() {
    local tier="$1" cap="$2" detail="$3"
    echo "PASS|$tier|$cap|$detail"
    PASS_COUNT=$((PASS_COUNT + 1))
}

fail() {
    local tier="$1" cap="$2" detail="$3"
    echo "FAIL|$tier|$cap|$detail"
    FAIL_COUNT=$((FAIL_COUNT + 1))
}

known_gap() {
    local tier="$1" cap="$2" detail="$3"
    echo "KNOWN_GAP|$tier|$cap|$detail (JH-0)"
    KNOWN_GAP_COUNT=$((KNOWN_GAP_COUNT + 1))
}

assert_succeeds() {
    local tier="$1" cap="$2" detail="$3" user="$4"
    shift 4
    if sudo -u "$user" bash -c "$*" >/dev/null 2>&1; then
        pass "$tier" "$cap" "$detail"
    else
        fail "$tier" "$cap" "EXPECTED SUCCESS: $detail"
    fi
}

assert_fails() {
    local tier="$1" cap="$2" detail="$3" user="$4"
    shift 4
    if sudo -u "$user" bash -c "$*" >/dev/null 2>&1; then
        fail "$tier" "$cap" "EXPECTED FAILURE: $detail"
    else
        pass "$tier" "$cap" "$detail"
    fi
}

# ═══════════════════════════════════════════════════
# Prerequisite check
# ═══════════════════════════════════════════════════

if [[ $EUID -ne 0 ]]; then
    echo "ERROR: Must run as root (sudo)" >&2
    exit 1
fi

for tier in admin compute reviewer observer; do
    user="${TIER_USERS[$tier]}"
    if ! id "$user" &>/dev/null; then
        echo "ERROR: Test user '$user' for tier '$tier' does not exist" >&2
        exit 1
    fi
done

echo "═══════════════════════════════════════════════════"
echo "  ABG Tier Enforcement Test Suite"
echo "  Date: $(date -Iseconds)"
echo "  Filter: $TIER_FILTER"
echo "═══════════════════════════════════════════════════"
echo ""

run_tier_tests() {
    local tier="$1"
    local user="${TIER_USERS[$tier]}"
    local user_home="/home/$user"

    echo "── Tier: $tier (user: $user) ──"

    # --- Filesystem: write to own notebooks/ ---
    if [[ "$tier" == "admin" || "$tier" == "compute" ]]; then
        assert_succeeds "$tier" "fs_write_notebooks" "Can write to ~/notebooks/" "$user" \
            "touch '$user_home/notebooks/.tier_test_$$' && rm -f '$user_home/notebooks/.tier_test_$$'"
    else
        assert_fails "$tier" "fs_write_notebooks" "Cannot write to ~/notebooks/ (550 root-owned)" "$user" \
            "touch '$user_home/notebooks/.tier_test_$$'"
    fi

    # --- Filesystem: write to shared commons ---
    assert_fails "$tier" "fs_write_shared_commons" "Cannot write to shared/commons/" "$user" \
        "touch '$ABG_SHARED/commons/.tier_test_$$'"

    # --- Filesystem: read /etc/shadow ---
    assert_fails "$tier" "fs_read_shadow" "Cannot read /etc/shadow" "$user" \
        "cat /etc/shadow"

    # --- Filesystem: read other users' homes ---
    local other_user
    if [[ "$tier" == "compute" ]]; then other_user="abgreviewer"; else other_user="tamison"; fi
    assert_fails "$tier" "fs_read_other_home" "Cannot read /home/$other_user/" "$user" \
        "ls /home/$other_user/"

    # --- Filesystem: read shared content ---
    assert_succeeds "$tier" "fs_read_shared_commons" "Can read shared/commons/" "$user" \
        "ls '$ABG_SHARED/commons/'"

    assert_succeeds "$tier" "fs_read_shared_showcase" "Can read shared/showcase/" "$user" \
        "ls '$ABG_SHARED/showcase/'"

    # --- Network: outbound internet ---
    # All ABG users (UID 1001-1099) are blocked by iptables regardless of tier.
    # Only irongate (system owner, UID 1000) has outbound internet access.
    local user_uid
    user_uid=$(id -u "$user" 2>/dev/null)
    if [[ "$user_uid" -ge "$ABG_UID_MIN" && "$user_uid" -le "$ABG_UID_MAX" ]]; then
        assert_fails "$tier" "net_outbound_internet" "Cannot reach internet (iptables DROP, UID $user_uid in ABG range)" "$user" \
            "curl -sf --max-time 5 https://github.com -o /dev/null"
    else
        assert_succeeds "$tier" "net_outbound_internet" "Can reach internet (UID $user_uid outside ABG range)" "$user" \
            "curl -sf --max-time 5 https://github.com -o /dev/null"
    fi

    # --- Network: localhost primals (JH-0 known gap) ---
    known_gap "$tier" "net_primal_rpc" "Can call primal RPC on localhost — no capability check (JH-0)"

    # --- Process: visibility ---
    local proc_count
    proc_count=$(sudo -u "$user" ps aux 2>/dev/null | wc -l)
    if [[ "$proc_count" -lt 20 ]]; then
        pass "$tier" "proc_isolation" "Sees $proc_count processes (hidepid=2 working)"
    else
        fail "$tier" "proc_isolation" "Sees $proc_count processes — hidepid=2 may not be active"
    fi

    # --- Filesystem: /tmp write (should work for all) ---
    assert_succeeds "$tier" "fs_write_tmp" "Can write to /tmp" "$user" \
        "touch '/tmp/.tier_test_$$' && rm -f '/tmp/.tier_test_$$'"

    # --- Filesystem: read irongate development dirs ---
    assert_fails "$tier" "fs_read_irongate_dev" "Cannot read irongate development dirs" "$user" \
        "ls /home/irongate/Development/"

    # --- Filesystem: read cloudflared config ---
    assert_fails "$tier" "fs_read_tunnel_config" "Cannot read tunnel credentials" "$user" \
        "cat /home/irongate/.cloudflared/config.yml"

    echo ""
}

# ═══════════════════════════════════════════════════
# Run tests
# ═══════════════════════════════════════════════════

for tier in admin compute reviewer observer; do
    if [[ "$TIER_FILTER" == "all" || "$TIER_FILTER" == "$tier" ]]; then
        run_tier_tests "$tier"
    fi
done

# ═══════════════════════════════════════════════════
# Summary
# ═══════════════════════════════════════════════════

echo "═══════════════════════════════════════════════════"
echo "  Results: $PASS_COUNT PASS, $FAIL_COUNT FAIL, $KNOWN_GAP_COUNT KNOWN_GAP"
echo "═══════════════════════════════════════════════════"

if [[ $FAIL_COUNT -gt 0 ]]; then
    echo ""
    echo "FAILURES DETECTED — tier boundaries are not enforced correctly."
    echo "Review FAIL lines above and fix before allowing ABG user access."
fi

exit "$FAIL_COUNT"
