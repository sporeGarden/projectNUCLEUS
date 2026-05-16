#!/usr/bin/env bash
# gate_switch.sh — Switch the primary gate for lab.primals.eco
#
# Transfers the compute surface (JupyterHub, observer, pappusCast) from
# the current primary gate to a target gate. The cloudflared tunnel
# replica on this gate is NOT stopped — it continues serving the static
# sporePrint site as a replica. Only compute services are migrated.
#
# In a multi-gate replica setup, all gates run cloudflared replicas
# serving primals.eco. This script only moves the "primary" designation
# (JupyterHub, observer, pappusCast) to a new gate.
#
# Prerequisites:
#   - SSH key access to target gate
#   - cloudflared installed on target gate (gate_provision.sh)
#   - ABG_SHARED available on target (via NestGate rsync or local mount)
#   - deploy.sh present on target gate
#
# Usage:
#   gate_switch.sh <target-gate-hostname> [--dry-run]
#
# Evolution: bash (now) -> Rust (tunnelKeeper absorbs) -> primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
MANIFEST="$SCRIPT_DIR/gate_manifest.toml"
CF_CONFIG="${CLOUDFLARED_DIR}/config.yml"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log()  { echo -e "${GREEN}[gate_switch]${NC} $*"; }
warn() { echo -e "${YELLOW}[gate_switch]${NC} $*"; }
err()  { echo -e "${RED}[gate_switch]${NC} $*" >&2; }

usage() {
    echo "Usage: gate_switch.sh <target-gate> [--dry-run]"
    echo
    echo "Switches the active gate for lab.primals.eco compute services."
    echo "The static observer (HTML exports, sporePrint) remains always-on."
    echo
    echo "Arguments:"
    echo "  target-gate    Hostname or IP of the gate to activate"
    echo "  --dry-run      Show what would happen without executing"
    exit 1
}

DRY_RUN=false
TARGET=""

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN=true ;;
        --help|-h) usage ;;
        *) TARGET="$arg" ;;
    esac
done

[[ -z "$TARGET" ]] && usage

run_or_print() {
    if $DRY_RUN; then
        echo "  [dry-run] $*"
    else
        "$@"
    fi
}

# ── Phase 1: Pre-flight checks ──────────────────────────────────────────

log "Phase 1: Pre-flight checks"

if ! ssh -o ConnectTimeout=5 -o BatchMode=yes "$TARGET" true 2>/dev/null; then
    err "Cannot SSH to $TARGET — check key access"
    exit 1
fi
log "  SSH connectivity: OK"

REMOTE_NUCLEUS="Development/ecoPrimals/gardens/projectNUCLEUS"
REMOTE_DEPLOY=$(ssh "$TARGET" "ls ~/${REMOTE_NUCLEUS}/deploy/deploy.sh 2>/dev/null" || true)
if [[ -z "$REMOTE_DEPLOY" ]]; then
    warn "  deploy.sh not found on $TARGET — services must be set up manually"
fi

REMOTE_CF=$(ssh "$TARGET" "which cloudflared 2>/dev/null" || true)
if [[ -z "$REMOTE_CF" ]]; then
    err "cloudflared not installed on $TARGET"
    exit 1
fi
log "  cloudflared on target: OK"

# ── Phase 2: Export static HTML snapshot before switch ───────────────────

log "Phase 2: Exporting static HTML snapshot (always-on fallback)"

ABG_SHARED="${ABG_SHARED}"
PAPPUSCAST="$SCRIPT_DIR/pappusCast.py"

if [[ -f "$PAPPUSCAST" ]]; then
    run_or_print python3 "$PAPPUSCAST" once --force
    log "  pappusCast full sync + HTML export: done"
else
    warn "  pappusCast.py not found — skipping pre-switch export"
fi

# ── Phase 3: Stop local compute services ─────────────────────────────────

log "Phase 3: Stopping local compute services"

for svc in jupyterhub observer-static pappusCast; do
    if systemctl is-active --quiet "$svc" 2>/dev/null; then
        run_or_print sudo systemctl stop "$svc"
        log "  Stopped $svc"
    fi
done

# ── Phase 4: Sync workspace to target gate ───────────────────────────────

log "Phase 4: Syncing ABG shared workspace to $TARGET"

run_or_print rsync -az --delete \
    --exclude='.pappusCast/' \
    --exclude='envs/' \
    --exclude='wheelhouse/' \
    --exclude='.ipynb_checkpoints/' \
    "$ABG_SHARED/" \
    "$TARGET:$ABG_SHARED/"

log "  Workspace sync: done"

# ── Phase 5: Deploy services on target gate ──────────────────────────────

log "Phase 5: Deploying services on $TARGET"

if [[ -n "$REMOTE_DEPLOY" ]]; then
    run_or_print ssh "$TARGET" "cd ~/${REMOTE_NUCLEUS}/deploy && bash deploy.sh"
    log "  Services deployed via deploy.sh"
else
    warn "  Manual service startup required on $TARGET"
    warn "  Start: jupyterhub, observer-static, pappusCast"
fi

# ── Phase 6: Ensure target gate has full tunnel config ────────────────────

log "Phase 6: Tunnel replica configuration on target"

CF_CRED_DIR="${CLOUDFLARED_DIR}"
if [[ -d "$CF_CRED_DIR" ]]; then
    run_or_print rsync -az "$CF_CRED_DIR/" "$TARGET:$CF_CRED_DIR/"
    log "  Tunnel credentials synced"
fi

# Ensure cloudflared is running on target with full config (all routes)
run_or_print ssh "$TARGET" "sudo systemctl enable --now cloudflared-replica 2>/dev/null || cloudflared tunnel run nucleus-lab &"
log "  Target gate has cloudflared replica running"

# Local cloudflared stays running as a membrane replica — NOT stopped.
# It continues routing lab/git.primals.eco in the tunnel pool.
log "  Local cloudflared replica: kept running (membrane failover)"

# ── Phase 7: Trigger pappusCast full sync on target ──────────────────────

log "Phase 7: Triggering pappusCast full sync on $TARGET"

run_or_print ssh "$TARGET" "cd ~/${REMOTE_NUCLEUS}/deploy && python3 pappusCast.py once --force"
log "  Remote pappusCast sync: done"

# ── Phase 8: Verify ─────────────────────────────────────────────────────

log "Phase 8: Verification"

if ! $DRY_RUN; then
    sleep 5
    HTTP_CODE=$(curl -s -o /dev/null -w '%{http_code}' --max-time 10 "https://lab.primals.eco/" 2>/dev/null || echo "000")
    if [[ "$HTTP_CODE" == "200" ]]; then
        log "  lab.primals.eco responds 200: ${GREEN}OK${NC}"
    else
        warn "  lab.primals.eco returned $HTTP_CODE — may need DNS propagation time"
    fi
else
    echo "  [dry-run] Would verify lab.primals.eco responds 200"
fi

# ── Summary ──────────────────────────────────────────────────────────────

echo
log "Gate switch complete:"
log "  Source gate: $(hostname) (now membrane-only replica)"
log "  Target gate: $TARGET (now primary — compute + membrane)"
log "  Static HTML: preserved in .pappusCast/html_export/"
log "  primals.eco: extracellular (GitHub Pages CDN, no gate dependency)"
log "  Membrane:    lab/git.primals.eco served by all tunnel replicas"
log "  Compute:     now active on $TARGET"

if $DRY_RUN; then
    echo
    warn "DRY RUN — no changes were made"
fi
