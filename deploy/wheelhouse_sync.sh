#!/usr/bin/env bash
# Wheelhouse Sync — Admin tool for managing the local package cache
#
# ABG compute users cannot reach PyPI (outbound internet blocked for ABG UIDs).
# This script runs as irongate (internet-capable) to download wheels into the
# shared wheelhouse. Users then install from it via pip's --find-links/--no-index
# (configured automatically in their venv's pip.conf).
#
# Usage:
#   bash wheelhouse_sync.sh add <package> [package...]    Download packages
#   bash wheelhouse_sync.sh list                          Show cached packages
#   bash wheelhouse_sync.sh update                        Re-download all cached packages
#   bash wheelhouse_sync.sh remove <package>              Remove a package's wheels

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

WHEELHOUSE="$ABG_SHARED/wheelhouse"
BIOINFO_PIP="$ABG_SHARED/envs/bioinfo/bin/pip"
PY_VERSION="3.12"

mkdir -p "$WHEELHOUSE"

case "${1:-help}" in
    add)
        shift
        [[ $# -lt 1 ]] && { echo "Usage: bash $0 add <package> [package...]"; exit 1; }
        echo "Downloading packages to wheelhouse: $*"
        "$BIOINFO_PIP" download \
            --python-version "$PY_VERSION" \
            --only-binary=:all: \
            -d "$WHEELHOUSE" \
            "$@" 2>&1
        echo ""
        echo "Done. Users can now run: %pip install $*"
        ;;
    list)
        echo "=== Wheelhouse Contents ==="
        echo "Path: $WHEELHOUSE"
        echo ""
        if [[ -d "$WHEELHOUSE" ]]; then
            ls -1 "$WHEELHOUSE"/*.whl 2>/dev/null | while read -r f; do
                basename "$f" | sed 's/-[0-9].*//'
            done | sort -u
            echo ""
            echo "$(ls "$WHEELHOUSE"/*.whl 2>/dev/null | wc -l) wheel files, $(du -sh "$WHEELHOUSE" | cut -f1)"
        else
            echo "(empty)"
        fi
        ;;
    update)
        echo "Re-downloading all cached packages ..."
        # Extract package names from existing wheel filenames
        pkgs=$(ls "$WHEELHOUSE"/*.whl 2>/dev/null | while read -r f; do
            basename "$f" | sed 's/-[0-9].*//' | tr '_' '-'
        done | sort -u)
        if [[ -z "$pkgs" ]]; then
            echo "Wheelhouse is empty, nothing to update."
            exit 0
        fi
        echo "Packages: $pkgs"
        # shellcheck disable=SC2086
        "$BIOINFO_PIP" download \
            --python-version "$PY_VERSION" \
            --only-binary=:all: \
            -d "$WHEELHOUSE" \
            $pkgs 2>&1
        echo "Done."
        ;;
    remove)
        shift
        [[ $# -lt 1 ]] && { echo "Usage: bash $0 remove <package>"; exit 1; }
        for pkg in "$@"; do
            normalized=$(echo "$pkg" | tr '-' '_' | tr '[:upper:]' '[:lower:]')
            found=$(find "$WHEELHOUSE" -maxdepth 1 -iname "${normalized}-*" -o -iname "$(echo "$pkg" | tr '_' '-')-*" 2>/dev/null)
            if [[ -n "$found" ]]; then
                echo "Removing: $found"
                rm -f $found
            else
                echo "No wheels found for: $pkg"
            fi
        done
        ;;
    *)
        echo "Wheelhouse Sync — Local package cache for ABG compute users"
        echo ""
        echo "Usage:"
        echo "  bash $0 add <package> [package...]    Download packages to wheelhouse"
        echo "  bash $0 list                          Show cached packages"
        echo "  bash $0 update                        Re-download all cached packages"
        echo "  bash $0 remove <package>              Remove a package's wheels"
        echo ""
        echo "Wheelhouse: $WHEELHOUSE"
        echo ""
        echo "Users install from the wheelhouse via: %pip install <package>"
        echo "(Their venv pip.conf is pre-configured to use --find-links --no-index)"
        ;;
esac
