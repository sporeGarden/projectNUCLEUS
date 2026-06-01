#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
# DEPRECATED: Use `nucleus-deploy dns` (Rust) instead. Wave 64.
#
# deploy_knot_dns.sh — Deploy knot-dns authoritative for primals.eco
#
# Channel 1 (Signal): Sovereign DNS on cellMembrane VPS.
# H2-17: knot-dns authoritative, H2-18: NS transfer from Cloudflare.
#
# Installs knot-dns on the VPS, creates the primals.eco zone with DNSSEC
# (ECDSAP256SHA256), opens port 53, and validates resolution.
#
# Usage:
#   bash deploy/deploy_knot_dns.sh [--dry-run]
#   bash deploy/deploy_knot_dns.sh --status
#   bash deploy/deploy_knot_dns.sh --test
#
# Prerequisites:
#   - SSH access to cellMembrane VPS (key-based)
#   - nucleus_config.sh sourced for MEMBRANE_VPS_IP

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh" 2>/dev/null || true

VPS_IP="${MEMBRANE_VPS_IP:-157.230.3.183}"
VPS_USER="${MEMBRANE_VPS_USER:-root}"
ZONE_DOMAIN="primals.eco"
DRY_RUN=false
MODE="deploy"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --status)  MODE="status"; shift ;;
        --test)    MODE="test"; shift ;;
        *)         echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

log()  { echo "[knot-dns] $*"; }
warn() { echo "[knot-dns] WARNING: $*" >&2; }

ssh_cmd() {
    ssh -o ConnectTimeout=10 -o BatchMode=yes -o StrictHostKeyChecking=accept-new \
        "$VPS_USER@$VPS_IP" "$@" 2>/dev/null
}

# ── Zone file generation ─────────────────────────────────────────────────
# Zone records reflect the current architecture:
#   - VPS IP for authoritative NS (ns1.primals.eco)
#   - GitHub Pages IPs for the apex (primals.eco A records)
#   - VPS A records for membrane, lab, git subdomains
#   - CAA for Let's Encrypt only

generate_zone() {
    local vps_ip="$1"
    local serial
    serial=$(date +%Y%m%d%H)

    cat << ZONE
\$ORIGIN primals.eco.
\$TTL 300

@ IN SOA ns1.primals.eco. admin.primals.eco. (
    $serial   ; serial (YYYYMMDDHH)
    3600      ; refresh (1 hour)
    900       ; retry (15 min)
    1209600   ; expire (2 weeks)
    300       ; minimum TTL (5 min)
)

; Nameservers — sovereign on cellMembrane VPS
@ IN NS ns1.primals.eco.

; NS glue record
ns1 IN A $vps_ip

; Apex — GitHub Pages CDN (extracellular, always-on)
@ IN A 185.199.108.153
@ IN A 185.199.109.153
@ IN A 185.199.110.153
@ IN A 185.199.111.153

; cellMembrane VPS
membrane IN A $vps_ip

; Lab surface — pappusCast static content, future BTSP relay for JupyterHub
lab IN A $vps_ip

; golgiBody — periplasmic Forgejo (sovereign git forge)
git IN A $vps_ip

; CAA — only Let's Encrypt may issue certs
@ IN CAA 0 issue "letsencrypt.org"
@ IN CAA 0 issuewild "letsencrypt.org"

; TXT — domain verification and SPF
@ IN TXT "v=spf1 -all"
ZONE
}

do_deploy() {
    log "Deploying knot-dns authoritative for $ZONE_DOMAIN"
    log "  VPS: $VPS_IP"
    log "  Channel: 1 (Signal)"
    log ""

    if $DRY_RUN; then
        log "[dry-run] Would install knot-dns from Debian repos"
        log "[dry-run] Would generate /etc/knot/knot.conf"
        log "[dry-run] Would generate /etc/knot/zones/primals.eco.zone"
        log "[dry-run] Would enable DNSSEC (ECDSAP256SHA256)"
        log "[dry-run] Would open UDP/TCP 53 in UFW"
        log "[dry-run] Would start knot.service"
        echo ""
        log "Zone file preview:"
        generate_zone "$VPS_IP"
        return 0
    fi

    log "Phase 1: Installing knot-dns..."
    ssh_cmd bash -s << 'INSTALL'
set -euo pipefail
if command -v knotd >/dev/null 2>&1; then
    echo "  knot-dns already installed: $(knotd --version 2>&1 | head -1)"
else
    apt-get update -qq
    apt-get install -y -qq knot knot-dnsutils >/dev/null 2>&1
    echo "  Installed: $(knotd --version 2>&1 | head -1)"
fi
mkdir -p /etc/knot/zones /var/lib/knot
chown -R knot:knot /var/lib/knot
INSTALL

    log "Phase 2: Writing configuration..."
    ssh_cmd bash -c "cat > /etc/knot/knot.conf" << CONF
server:
    rundir: "/run/knot"
    user: knot:knot
    listen: ${VPS_IP}@53
    identity: "ns1.primals.eco"
    version: ""

log:
  - target: syslog
    any: info

database:
    storage: "/var/lib/knot"

policy:
  - id: ecdsap256
    algorithm: ECDSAP256SHA256
    ksk-lifetime: 365d
    zsk-lifetime: 90d
    nsec3: on
    nsec3-iterations: 0

template:
  - id: default
    storage: "/etc/knot/zones"
    file: "%s.zone"
    semantic-checks: on
    zonefile-sync: -1
    zonefile-load: difference-no-serial
    journal-content: all

zone:
  - domain: primals.eco
    dnssec-signing: on
    dnssec-policy: ecdsap256
CONF

    log "Phase 3: Writing zone file..."
    local zone_content
    zone_content=$(generate_zone "$VPS_IP")
    echo "$zone_content" | ssh_cmd "cat > /etc/knot/zones/primals.eco.zone"
    ssh_cmd "chown -R knot:knot /etc/knot/zones"

    log "Phase 4: Validating configuration..."
    ssh_cmd "knotc conf-check 2>&1" || {
        warn "Configuration check failed — review /etc/knot/knot.conf"
        return 1
    }

    log "Phase 5: Opening firewall..."
    ssh_cmd "ufw allow 53/tcp comment 'Channel 1: DNS (knot-dns)'" 2>/dev/null || true
    ssh_cmd "ufw allow 53/udp comment 'Channel 1: DNS (knot-dns)'" 2>/dev/null || true

    log "Phase 6: Starting knot-dns..."
    ssh_cmd "systemctl enable knot && systemctl restart knot"
    sleep 2

    local status
    status=$(ssh_cmd "systemctl is-active knot 2>/dev/null || echo 'failed'")
    if [[ "$status" == "active" ]]; then
        log "knot-dns ACTIVE"
    else
        warn "knot-dns failed to start — checking journal:"
        ssh_cmd "journalctl -u knot --no-pager -n 15" 2>&1
        return 1
    fi

    log ""
    log "Phase 7: Validation..."
    do_test

    log ""
    log "═══════════════════════════════════════════════"
    log "  Channel 1 (Signal): knot-dns DEPLOYED"
    log "  Zone: $ZONE_DOMAIN"
    log "  NS:   ns1.$ZONE_DOMAIN → $VPS_IP"
    log "  DNSSEC: ECDSAP256SHA256 (auto-signed)"
    log ""
    log "  Next steps:"
    log "    H2-18: Transfer NS from Cloudflare registrar"
    log "           to ns1.$ZONE_DOMAIN ($VPS_IP)"
    log "    Shadow: Both NS active (CF + sovereign) for 14 days"
    log "═══════════════════════════════════════════════"
}

do_status() {
    log "Channel 1 (Signal) status:"
    local active
    active=$(ssh_cmd "systemctl is-active knot 2>/dev/null || echo 'not-found'")
    log "  knot.service: $active"

    if [[ "$active" == "active" ]]; then
        local version
        version=$(ssh_cmd "knotd --version 2>&1 | head -1")
        log "  Version: $version"
        log ""
        log "  Zone status:"
        ssh_cmd "knotc zone-status primals.eco 2>&1" | while IFS= read -r line; do
            log "    $line"
        done

        log ""
        log "  DNSSEC keys:"
        ssh_cmd "keymgr primals.eco list 2>&1" | while IFS= read -r line; do
            log "    $line"
        done

        log ""
        log "  Listeners:"
        ssh_cmd "ss -ulnp 2>/dev/null | grep ':53 '" | while IFS= read -r line; do
            log "    $line"
        done
    fi
}

do_test() {
    log "Testing resolution against $VPS_IP..."

    local domains=("primals.eco" "ns1.primals.eco" "membrane.primals.eco" "lab.primals.eco" "git.primals.eco")
    local pass=0
    local fail=0

    for domain in "${domains[@]}"; do
        local result
        result=$(ssh_cmd "khost $domain localhost 2>/dev/null | head -3" || true)
        if [[ -z "$result" ]]; then
            result=$(dig +short "@$VPS_IP" "$domain" A 2>/dev/null | head -1 || true)
        fi
        if [[ -n "$result" ]]; then
            log "  PASS: $domain → $(echo "$result" | head -1)"
            pass=$((pass + 1))
        else
            log "  FAIL: $domain — no response"
            fail=$((fail + 1))
        fi
    done

    local dnssec_ok
    dnssec_ok=$(dig +dnssec +short "$ZONE_DOMAIN" A "@$VPS_IP" 2>/dev/null | grep -c "RRSIG" || echo 0)
    if [[ "$dnssec_ok" -gt 0 ]]; then
        log "  PASS: DNSSEC signatures present"
        pass=$((pass + 1))
    else
        log "  INFO: DNSSEC — dig not available or signatures not yet signed"
    fi

    local axfr_blocked
    axfr_blocked=$(dig AXFR "$ZONE_DOMAIN" "@$VPS_IP" 2>/dev/null | grep -c "Transfer failed" || echo 0)
    if [[ "$axfr_blocked" -gt 0 ]] || ! dig AXFR "$ZONE_DOMAIN" "@$VPS_IP" 2>/dev/null | grep -q "^${ZONE_DOMAIN}"; then
        log "  PASS: AXFR zone transfer blocked"
        pass=$((pass + 1))
    else
        log "  WARN: AXFR zone transfer may be open"
    fi

    log ""
    log "  DNS validation: $pass PASS, $fail FAIL"
}

case "$MODE" in
    deploy) do_deploy ;;
    status) do_status ;;
    test)   do_test ;;
esac
