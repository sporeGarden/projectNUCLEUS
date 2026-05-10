#!/usr/bin/env bash
# sporeprint_dns.sh — Manage primals.eco DNS routing via Cloudflare API
#
# Cell membrane model: primals.eco lives permanently on GitHub Pages
# (extracellular). This script is retained for emergency use only —
# the 'sovereign' command can temporarily route primals.eco through
# the tunnel if GitHub Pages has an extended outage.
#
# Normal operation: primals.eco → GitHub Pages (always)
#                   lab/git.primals.eco → tunnel (membrane)
#
# Usage:
#   sporeprint_dns.sh status       — show current DNS state
#   sporeprint_dns.sh sovereign    — route through tunnel (emergency only)
#   sporeprint_dns.sh external     — route to GitHub Pages (normal state)
#   sporeprint_dns.sh verify       — check which origin is actually serving
#
# Requires: ~/.cloudflared/cf_api_token (Cloudflare API token with DNS edit)
#
# Evolution: bash (now) → Rust (tunnelKeeper absorbs) → primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

TOKEN_FILE="${CF_API_TOKEN_FILE}"
ZONE_NAME="${CF_ZONE_NAME}"
TUNNEL_ID="${CF_TUNNEL_ID}"
TUNNEL_CNAME="${CF_TUNNEL_CNAME}"
RECORD_NAME="${CF_ZONE_NAME}"

GHPAGES_IPS=("185.199.108.153" "185.199.109.153" "185.199.110.153" "185.199.111.153")

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

log()  { echo -e "${GREEN}[dns]${NC} $*"; }
warn() { echo -e "${YELLOW}[dns]${NC} $*"; }
err()  { echo -e "${RED}[dns]${NC} $*" >&2; }
info() { echo -e "${CYAN}[dns]${NC} $*"; }

# ── Auth ─────────────────────────────────────────────────────────────────

load_token() {
    if [[ ! -f "$TOKEN_FILE" ]]; then
        err "No API token found at $TOKEN_FILE"
        err "Create one at: Cloudflare Dashboard → My Profile → API Tokens"
        err "Permissions needed: Zone.DNS:Edit, scoped to primals.eco"
        exit 1
    fi
    CF_TOKEN=$(tr -d '\n' < "$TOKEN_FILE")
}

cf_api() {
    local method="$1"
    local endpoint="$2"
    shift 2
    curl -s -X "$method" \
        "https://api.cloudflare.com/client/v4${endpoint}" \
        -H "Authorization: Bearer $CF_TOKEN" \
        -H "Content-Type: application/json" \
        "$@"
}

# ── Zone lookup ──────────────────────────────────────────────────────────

get_zone_id() {
    local resp
    resp=$(cf_api GET "/zones?name=${ZONE_NAME}")
    ZONE_ID=$(echo "$resp" | python3 -c "
import sys, json
d = json.load(sys.stdin)
if d.get('success') and d['result']:
    print(d['result'][0]['id'])
else:
    print('ERROR')
" 2>/dev/null)

    if [[ "$ZONE_ID" == "ERROR" || -z "$ZONE_ID" ]]; then
        err "Could not find zone for $ZONE_NAME"
        err "Check that your API token has access to this zone"
        exit 1
    fi
}

# ── DNS record helpers ───────────────────────────────────────────────────

list_records() {
    cf_api GET "/zones/${ZONE_ID}/dns_records?name=${RECORD_NAME}&per_page=50"
}

delete_record() {
    local record_id="$1"
    cf_api DELETE "/zones/${ZONE_ID}/dns_records/${record_id}"
}

create_a_record() {
    local ip="$1"
    cf_api POST "/zones/${ZONE_ID}/dns_records" \
        -d "{\"type\":\"A\",\"name\":\"${RECORD_NAME}\",\"content\":\"${ip}\",\"proxied\":true,\"ttl\":1}"
}

create_cname_record() {
    local target="$1"
    cf_api POST "/zones/${ZONE_ID}/dns_records" \
        -d "{\"type\":\"CNAME\",\"name\":\"${RECORD_NAME}\",\"content\":\"${target}\",\"proxied\":true,\"ttl\":1}"
}

# ── Commands ─────────────────────────────────────────────────────────────

do_status() {
    log "Fetching DNS records for $RECORD_NAME..."
    local resp
    resp=$(list_records)

    echo "$resp" | python3 -c "
import sys, json
d = json.load(sys.stdin)
if not d.get('success'):
    print('API error:', json.dumps(d.get('errors', [])))
    sys.exit(1)

records = d['result']
if not records:
    print('  No DNS records found for ${RECORD_NAME}')
    sys.exit(0)

ghpages = set(['185.199.108.153','185.199.109.153','185.199.110.153','185.199.111.153'])
tunnel_cname = '${TUNNEL_CNAME}'

a_records = [r for r in records if r['type'] == 'A']
cname_records = [r for r in records if r['type'] == 'CNAME']

print()
for r in records:
    proxied = 'proxied' if r.get('proxied') else 'DNS only'
    print(f\"  {r['type']:6s} {r['name']:20s} → {r['content']:45s} ({proxied})\")

print()
a_ips = set(r['content'] for r in a_records)
if a_ips and a_ips.issubset(ghpages):
    print('  Routing: EXTERNAL (GitHub Pages)')
elif any(r['content'] == tunnel_cname for r in cname_records):
    print('  Routing: SOVEREIGN (tunnel → gate)')
else:
    print('  Routing: UNKNOWN (manual inspection needed)')
"
}

do_sovereign() {
    log "Switching $RECORD_NAME to sovereign (tunnel) routing..."
    warn "NOTE: Brief DNS gap is unavoidable (A records must be deleted before CNAME"
    warn "can be created). ISP resolvers may cache negative responses during this gap."
    warn "LAN devices using ISP DNS may need cache flush or resolver change to 1.1.1.1."

    local resp
    resp=$(list_records)

    local record_ids
    record_ids=$(echo "$resp" | python3 -c "
import sys, json
d = json.load(sys.stdin)
for r in d.get('result', []):
    if r['type'] in ('A', 'AAAA', 'CNAME'):
        print(r['id'])
")

    if [[ -n "$record_ids" ]]; then
        log "Removing existing records (fast batch)..."
        while IFS= read -r rid; do
            delete_record "$rid" > /dev/null &
        done <<< "$record_ids"
        wait
        info "  All records deleted"
    fi

    log "Creating CNAME → $TUNNEL_CNAME (proxied) immediately..."
    local create_resp
    create_resp=$(create_cname_record "$TUNNEL_CNAME")
    local create_ok
    create_ok=$(echo "$create_resp" | python3 -c "import sys,json; print(json.load(sys.stdin).get('success',''))" 2>/dev/null)

    if [[ "$create_ok" == "True" ]]; then
        log "DNS switched to sovereign routing"
        info "  $RECORD_NAME → tunnel → gate:8880"
        echo ""
        log "Verifying..."
        sleep 3
        do_verify
    else
        err "Failed to create CNAME record"
        echo "$create_resp" | python3 -m json.tool 2>/dev/null || echo "$create_resp"
        exit 1
    fi
}

do_external() {
    log "Switching $RECORD_NAME to external (GitHub Pages) routing..."

    local resp
    resp=$(list_records)

    local record_ids
    record_ids=$(echo "$resp" | python3 -c "
import sys, json
d = json.load(sys.stdin)
for r in d.get('result', []):
    if r['type'] in ('A', 'AAAA', 'CNAME'):
        print(r['id'])
")

    if [[ -n "$record_ids" ]]; then
        log "Removing existing records (fast batch)..."
        while IFS= read -r rid; do
            delete_record "$rid" > /dev/null &
        done <<< "$record_ids"
        wait
        info "  All records deleted"
    fi

    log "Creating A records for GitHub Pages..."
    for ip in "${GHPAGES_IPS[@]}"; do
        create_a_record "$ip" > /dev/null &
    done
    wait
    info "  Created 4 A records"

    log "DNS switched to external routing"
    info "  $RECORD_NAME → GitHub Pages CDN"
    echo ""
    log "Verifying..."
    sleep 3
    do_verify
}

do_verify() {
    info "Checking live origin..."

    local headers
    headers=$(curl -sI --max-time 10 "${SITE_URL}/" 2>/dev/null)

    local has_fastly
    has_fastly=$(echo "$headers" | grep -ci "fastly\|varnish\|x-served-by" || true)
    local has_cf
    has_cf=$(echo "$headers" | grep -ci "cf-ray" || true)

    local http_code
    http_code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 "${SITE_URL}/" 2>/dev/null || echo "000")

    echo ""
    info "  HTTP status: $http_code"

    if [[ "$has_fastly" -gt 0 ]]; then
        info "  Origin: GitHub Pages (Fastly CDN detected)"
    elif [[ "$has_cf" -gt 0 && "$has_fastly" -eq 0 ]]; then
        info "  Origin: Sovereign gate (Cloudflare tunnel, no Fastly)"
    else
        warn "  Origin: Unknown (no distinguishing headers)"
    fi

    local local_status
    local_status=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "http://${NUCLEUS_BIND_ADDRESS}:${SPOREPRINT_LOCAL_PORT}/" 2>/dev/null || echo "000")
    info "  Local server: $([[ "$local_status" == "200" ]] && echo "UP" || echo "DOWN")"
}

# ── Main ─────────────────────────────────────────────────────────────────

load_token
get_zone_id

case "${1:-status}" in
    status)    do_status ;;
    sovereign) do_sovereign ;;
    external)  do_external ;;
    verify)    do_verify ;;
    *)
        echo "Usage: sporeprint_dns.sh {status|sovereign|external|verify}"
        echo ""
        echo "  status     Show current DNS routing"
        echo "  sovereign  Switch to tunnel (gate serves primals.eco)"
        echo "  external   Switch to GitHub Pages"
        echo "  verify     Check which origin is actually serving"
        exit 1
        ;;
esac
