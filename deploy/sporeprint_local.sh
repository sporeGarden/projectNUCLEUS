#!/usr/bin/env bash
# sporeprint_local.sh — Pull, build, and serve sporePrint locally
#
# Development and preview tool for the sporePrint static site.
# NOT in the production path — primals.eco is served by GitHub Pages
# + Cloudflare CDN (extracellular). This script is for local preview,
# testing Zola builds, and verifying content before pushing to git.
#
# Modes:
#   build   — git pull + zola build
#   serve   — serve public/ on 127.0.0.1:8880 (local preview)
#   once    — pull + build + serve (one-shot for testing)
#
# Evolution: bash (now) → Rust (petalTongue absorbs) → primal

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "${SCRIPT_DIR}/nucleus_config.sh" 2>/dev/null \
  || { echo "ERROR: Cannot find nucleus_config.sh" >&2; exit 1; }

SPOREPRINT_REPO="${SPOREPRINT_REPO}"
ZOLA="${ZOLA:-/usr/local/bin/zola}"
SERVE_PORT="${SPOREPRINT_LOCAL_PORT}"
SERVE_ADDR="${NUCLEUS_BIND_ADDRESS}"
PID_FILE="/tmp/sporeprint-local.pid"
LOG_TAG="sporeprint-local"

log()  { echo "[${LOG_TAG}] $(date '+%H:%M:%S') $*"; }
err()  { echo "[${LOG_TAG}] $(date '+%H:%M:%S') ERROR: $*" >&2; }

check_deps() {
    if [[ ! -x "$ZOLA" ]]; then
        err "Zola not found at $ZOLA"
        exit 1
    fi
    if [[ ! -d "$SPOREPRINT_REPO/.git" ]]; then
        err "sporePrint repo not found at $SPOREPRINT_REPO"
        exit 1
    fi
}

do_build() {
    check_deps
    log "Pulling latest sporePrint..."
    cd "$SPOREPRINT_REPO"

    if git fetch origin main 2>/dev/null; then
        local behind
        behind=$(git rev-list --count HEAD..origin/main 2>/dev/null || echo 0)
        if [[ "$behind" -gt 0 ]]; then
            log "  $behind commit(s) behind — pulling"
            git pull --ff-only origin main 2>/dev/null || {
                err "Pull failed (dirty tree?) — building from current state"
            }
        else
            log "  Already up to date"
        fi
    else
        log "  Fetch failed (offline?) — building from current state"
    fi

    log "Building site with Zola..."
    if $ZOLA build 2>&1; then
        local page_count
        page_count=$(find public/ -name '*.html' | wc -l)
        log "Build complete: $page_count HTML pages in public/"
    else
        err "Zola build failed"
        return 1
    fi
}

do_serve() {
    check_deps
    local serve_dir="${SPOREPRINT_REPO}/public"

    if [[ ! -d "$serve_dir" ]]; then
        log "No public/ directory — running build first"
        do_build
    fi

    log "Serving on ${SERVE_ADDR}:${SERVE_PORT} from ${serve_dir}"
    cd "$serve_dir"
    exec python3 -m http.server "$SERVE_PORT" --bind "$SERVE_ADDR"
}

do_once() {
    do_build
    do_serve
}

case "${1:-once}" in
    build) do_build ;;
    serve) do_serve ;;
    once)  do_once ;;
    *)
        echo "Usage: sporeprint_local.sh {build|serve|once}"
        exit 1
        ;;
esac
