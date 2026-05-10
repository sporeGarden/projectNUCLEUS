#!/usr/bin/env bash
# sporeprint_verify.sh — Verify sporePrint health on both origins
#
# Checks GitHub Pages (external) and local gate server (sovereign)
# for primals.eco, validates key pages, compares build freshness.
#
# Usage:
#   sporeprint_verify.sh              # full check, human-readable
#   sporeprint_verify.sh --json       # machine-readable summary
#   sporeprint_verify.sh --tier-test  # pipe-delimited for tier_test_all.sh
#
# Evolution: bash (now) → Rust (sporeGuard absorbs) → primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

SPOREPRINT_REPO="${SPOREPRINT_REPO}"
LOCAL_PORT="${SPOREPRINT_LOCAL_PORT}"
LOCAL_ADDR="${NUCLEUS_BIND_ADDRESS}"
ZOLA="${ZOLA:-/usr/local/bin/zola}"

EXTERNAL_URL="${SITE_URL}"
LOCAL_URL="http://${LOCAL_ADDR}:${LOCAL_PORT}"

KEY_PATHS=(
    "/"
    "/science/"
    "/architecture/"
    "/lab/"
    "/glossary/"
    "/primals/"
    "/springs/"
)

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

MODE="human"
for arg in "$@"; do
    case "$arg" in
        --json) MODE="json" ;;
        --tier-test) MODE="tier-test" ;;
    esac
done

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
RESULTS=()

pass() {
    PASS_COUNT=$((PASS_COUNT + 1))
    local msg="$1"
    RESULTS+=("PASS|$msg")
    if [[ "$MODE" == "human" ]]; then
        echo -e "  ${GREEN}PASS${NC}  $msg"
    elif [[ "$MODE" == "tier-test" ]]; then
        echo "PASS|sporePrint|$msg"
    fi
}

fail() {
    FAIL_COUNT=$((FAIL_COUNT + 1))
    local msg="$1"
    RESULTS+=("FAIL|$msg")
    if [[ "$MODE" == "human" ]]; then
        echo -e "  ${RED}FAIL${NC}  $msg"
    elif [[ "$MODE" == "tier-test" ]]; then
        echo "FAIL|sporePrint|$msg"
    fi
}

warn() {
    WARN_COUNT=$((WARN_COUNT + 1))
    local msg="$1"
    RESULTS+=("WARN|$msg")
    if [[ "$MODE" == "human" ]]; then
        echo -e "  ${YELLOW}WARN${NC}  $msg"
    elif [[ "$MODE" == "tier-test" ]]; then
        echo "WARN|sporePrint|$msg"
    fi
}

LAST_HTTP_CODE=""

check_url() {
    local url="$1"
    local label="$2"
    LAST_HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 "$url" 2>/dev/null || echo "000")
    if [[ "$LAST_HTTP_CODE" == "200" ]]; then
        pass "${label} → ${LAST_HTTP_CODE}"
    elif [[ "$LAST_HTTP_CODE" == "000" ]]; then
        fail "${label} → unreachable"
    else
        fail "${label} → ${LAST_HTTP_CODE}"
    fi
}

# ── Section 1: Local server health ─────────────────────────────────────

[[ "$MODE" == "human" ]] && echo -e "\n${GREEN}── Local preview server (dev only) ──${NC}"

LOCAL_SERVICE=$(systemctl --user is-active sporeprint-local.service 2>/dev/null || echo "inactive")
if [[ "$LOCAL_SERVICE" == "active" ]]; then
    pass "sporeprint-local.service active (dev preview)"
else
    # Not a failure in production — primals.eco served by GitHub Pages CDN
    warn "sporeprint-local.service ${LOCAL_SERVICE} (dev preview only, not required)"
fi

check_url "${LOCAL_URL}/" "local:8880 /"
LOCAL_STATUS="$LAST_HTTP_CODE"

for path in "${KEY_PATHS[@]}"; do
    [[ "$path" == "/" ]] && continue
    check_url "${LOCAL_URL}${path}" "local:8880 ${path}"
done

# ── Section 2: External (GitHub Pages) health ──────────────────────────

[[ "$MODE" == "human" ]] && echo -e "\n${GREEN}── External (GitHub Pages) ──${NC}"

check_url "${EXTERNAL_URL}/" "external /"
EXT_STATUS="$LAST_HTTP_CODE"

for path in "${KEY_PATHS[@]}"; do
    [[ "$path" == "/" ]] && continue
    check_url "${EXTERNAL_URL}${path}" "external ${path}"
done

# ── Section 3: Zola build validation ───────────────────────────────────

[[ "$MODE" == "human" ]] && echo -e "\n${GREEN}── Build validation ──${NC}"

if [[ -x "$ZOLA" && -d "$SPOREPRINT_REPO" ]]; then
    BUILD_OUT=$(cd "$SPOREPRINT_REPO" && $ZOLA build 2>&1)
    if [[ $? -eq 0 ]]; then
        page_count=$(echo "$BUILD_OUT" | rg -o 'Creating [0-9]+ pages' | rg -o '[0-9]+' || echo "?")
        pass "zola build OK (${page_count} pages)"
    else
        fail "zola build failed"
    fi
else
    warn "zola or sporePrint repo not available for build check"
fi

# ── Section 4: Freshness comparison ────────────────────────────────────

[[ "$MODE" == "human" ]] && echo -e "\n${GREEN}── Freshness ──${NC}"

if [[ -d "$SPOREPRINT_REPO/.git" ]]; then
    REPO_HEAD=$(cd "$SPOREPRINT_REPO" && git rev-parse --short HEAD 2>/dev/null || echo "unknown")
    REPO_DATE=$(cd "$SPOREPRINT_REPO" && git log -1 --format='%ci' 2>/dev/null || echo "unknown")
    pass "repo HEAD: ${REPO_HEAD} (${REPO_DATE})"

    BEHIND=$(cd "$SPOREPRINT_REPO" && git fetch origin main 2>/dev/null && git rev-list --count HEAD..origin/main 2>/dev/null || echo "?")
    if [[ "$BEHIND" == "0" ]]; then
        pass "repo up to date with origin/main"
    elif [[ "$BEHIND" == "?" ]]; then
        warn "could not check freshness against origin"
    else
        warn "repo is ${BEHIND} commit(s) behind origin/main"
    fi
fi

# ── Section 5: Tunnel health ───────────────────────────────────────────

[[ "$MODE" == "human" ]] && echo -e "\n${GREEN}── Cloudflare tunnel ──${NC}"

TUNNEL_STATUS=$(systemctl --user is-active cloudflared-tunnel.service 2>/dev/null || echo "inactive")
if [[ "$TUNNEL_STATUS" == "active" ]]; then
    pass "cloudflared-tunnel.service active"
else
    fail "cloudflared-tunnel.service ${TUNNEL_STATUS}"
fi

# Cell membrane model: primals.eco is extracellular (GitHub Pages CDN),
# tunnel only routes lab/git subdomains (membrane channels)
if rg -q 'hostname: lab.primals.eco' "${CLOUDFLARED_DIR}/config.yml" 2>/dev/null; then
    pass "lab.primals.eco in tunnel ingress config (membrane)"
else
    fail "lab.primals.eco NOT in tunnel ingress config"
fi

# ── Summary ────────────────────────────────────────────────────────────

if [[ "$MODE" == "human" ]]; then
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

    FAILOVER_READY="no"
    if [[ "$LOCAL_STATUS" == "200" && "$TUNNEL_STATUS" == "active" ]]; then
        FAILOVER_READY="yes"
    fi

    echo -e "  Pass: ${PASS_COUNT}  Fail: ${FAIL_COUNT}  Warn: ${WARN_COUNT}"
    echo -e "  External (GH Pages): $([ "$EXT_STATUS" = "200" ] && echo "${GREEN}UP${NC}" || echo "${RED}DOWN${NC}")"
    echo -e "  Local sovereign:     $([ "$LOCAL_STATUS" = "200" ] && echo "${GREEN}UP${NC}" || echo "${RED}DOWN${NC}")"
    echo -e "  Failover ready:      $([ "$FAILOVER_READY" = "yes" ] && echo "${GREEN}YES${NC}" || echo "${RED}NO${NC}")"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
elif [[ "$MODE" == "json" ]]; then
    cat <<JSONEOF
{
  "timestamp": "$(date -Iseconds)",
  "pass": $PASS_COUNT,
  "fail": $FAIL_COUNT,
  "warn": $WARN_COUNT,
  "external_status": "$EXT_STATUS",
  "local_status": "$LOCAL_STATUS",
  "tunnel_active": $([ "$TUNNEL_STATUS" = "active" ] && echo "true" || echo "false"),
  "failover_ready": $([ "$LOCAL_STATUS" = "200" ] && [ "$TUNNEL_STATUS" = "active" ] && echo "true" || echo "false")
}
JSONEOF
fi

exit $(( FAIL_COUNT > 125 ? 125 : FAIL_COUNT ))
