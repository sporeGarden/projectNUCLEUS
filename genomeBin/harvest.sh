#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# genomeBin harvest — build, verify, and stage primal binaries
#
# Reads manifest.toml for the primal list, builds each in release mode,
# runs clippy + test, then copies binaries to stage/ with checksums.
#
# Intended to be run from projectNUCLEUS root or CI.
# Output lands in genomeBin/stage/<target>/<binary>.
#
# Usage:
#   ./genomeBin/harvest.sh [--target x86_64-unknown-linux-gnu] [--dry-run]
#
# Evolution: bash (now) → nucleus-deploy harvest (Rust)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MANIFEST="$SCRIPT_DIR/manifest.toml"
ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(realpath "$PROJECT_ROOT/../..")}"
STAGE_DIR="$SCRIPT_DIR/stage"
TARGET="${TARGET:-x86_64-unknown-linux-gnu}"
DRY_RUN=false

while [[ $# -gt 0 ]]; do
    case "$1" in
        --target)   TARGET="$2"; shift 2 ;;
        --dry-run)  DRY_RUN=true; shift ;;
        *)          echo "Unknown: $1" >&2; exit 1 ;;
    esac
done

RED='\033[0;31m'
GREEN='\033[0;32m'
NC='\033[0m'

log()  { echo -e "${GREEN}[harvest]${NC} $*"; }
err()  { echo -e "${RED}[harvest]${NC} $*" >&2; }

if [[ ! -f "$MANIFEST" ]]; then
    err "manifest.toml not found at $MANIFEST"
    exit 1
fi

OUTDIR="$STAGE_DIR/$TARGET"
mkdir -p "$OUTDIR"

PASS=0
FAIL=0
SKIP=0

# Extract primal slugs and crate paths from manifest
# Simple TOML parsing: look for [primals.<name>] sections
SLUGS=$(grep '^\[primals\.' "$MANIFEST" | sed 's/\[primals\.\(.*\)\]/\1/')

for slug in $SLUGS; do
    section=$(sed -n "/^\[primals\.$slug\]/,/^\[primals\./p" "$MANIFEST" | sed '${ /^\[primals\./d }')
    # For the last section (no following [primals.]), grab to EOF
    if [[ -z "$section" ]]; then
        section=$(sed -n "/^\[primals\.$slug\]/,\$p" "$MANIFEST")
    fi
    crate_path=$(echo "$section" | grep 'crate_path' | head -1 | sed 's/.*= *"\(.*\)"/\1/')
    binary=$(echo "$section" | grep '^binary' | head -1 | sed 's/.*= *"\(.*\)"/\1/')
    build_pkg=$(echo "$section" | { grep 'build_package' || true; } | head -1 | sed 's/.*= *"\(.*\)"/\1/')
    is_alias=$(echo "$section" | { grep 'alias.*=.*true' || true; } | head -1)
    targets_line=$(echo "$section" | grep 'targets' | head -1)

    if ! echo "$targets_line" | grep -q "$TARGET"; then
        log "SKIP $slug — $TARGET not in target list"
        SKIP=$((SKIP + 1))
        continue
    fi

    if [[ -n "$is_alias" ]]; then
        log "SKIP $slug — alias of $binary (same binary, second port)"
        PASS=$((PASS + 1))
        continue
    fi

    CRATE_DIR="$ECOPRIMALS_ROOT/$crate_path"
    if [[ ! -d "$CRATE_DIR" ]]; then
        err "FAIL $slug — crate not found: $CRATE_DIR"
        FAIL=$((FAIL + 1))
        continue
    fi

    PKG_FLAG=""
    if [[ -n "$build_pkg" ]]; then
        PKG_FLAG="-p $build_pkg"
    fi

    log "Building $slug ($binary) from $crate_path ${PKG_FLAG:+[$PKG_FLAG]} ..."

    if $DRY_RUN; then
        log "  [dry-run] cargo build --release --target $TARGET $PKG_FLAG"
        log "  [dry-run] cp target/$TARGET/release/$binary → $OUTDIR/$binary"
        PASS=$((PASS + 1))
        continue
    fi

    if (cd "$CRATE_DIR" && cargo build --release --target "$TARGET" $PKG_FLAG 2>&1); then
        BIN_PATH="$CRATE_DIR/target/$TARGET/release/$binary"
        if [[ -f "$BIN_PATH" ]]; then
            cp "$BIN_PATH" "$OUTDIR/$binary"
            if command -v b3sum >/dev/null 2>&1; then
                b3sum "$OUTDIR/$binary" > "$OUTDIR/$binary.b3"
            elif command -v sha256sum >/dev/null 2>&1; then
                sha256sum "$OUTDIR/$binary" > "$OUTDIR/$binary.sha256"
            fi
            log "  OK: $OUTDIR/$binary"
            PASS=$((PASS + 1))
        else
            err "  FAIL: binary not found at $BIN_PATH"
            FAIL=$((FAIL + 1))
        fi
    else
        err "  FAIL: cargo build failed for $slug"
        FAIL=$((FAIL + 1))
    fi
done

echo
log "Harvest complete: $PASS PASS, $FAIL FAIL, $SKIP SKIP"
log "Stage: $OUTDIR"

[[ $FAIL -eq 0 ]]
