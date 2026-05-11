#!/usr/bin/env bash
# publish_sporeprint.sh — Publish sporePrint site to NestGate content pipeline
#
# Builds sporePrint with Zola, then pushes rendered HTML/CSS/JS to NestGate
# via content.put RPC. Each file stored by its BLAKE3 hash. Creates a
# versioned collection manifest for petalTongue to serve.
#
# Prerequisites:
#   - NestGate running with content.put support (H2-05)
#   - Zola installed and sporePrint repo available
#   - petalTongue configured in web mode (H2-06)
#
# Usage:
#   bash publish_sporeprint.sh [--dry-run] [--version TAG]
#
# Status: STUB — awaiting NestGate content.put implementation (H2-05).
#   When NestGate ships content.put, this script becomes the publish pipeline.
#   See specs/TUNNEL_EVOLUTION.md Step 3a for the full design.

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/nucleus_config.sh"

DRY_RUN=false
VERSION="$(date +%Y%m%d-%H%M%S)"

while [[ $# -gt 0 ]]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --version) VERSION="$2"; shift 2 ;;
        *) echo "Unknown arg: $1" >&2; exit 1 ;;
    esac
done

SPOREPRINT_BUILD="$SPOREPRINT_REPO/public"

log() { echo "[publish_sporeprint] $(date '+%H:%M:%S') $*"; }

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$NESTGATE_PORT" 2>/dev/null
}

check_nestgate_content_support() {
    local response
    response=$(rpc_nestgate '{"jsonrpc":"2.0","method":"content.put","params":{"test":true},"id":1}')
    if echo "$response" | grep -q '"error".*Method not found'; then
        echo "ERROR: NestGate does not support content.put yet (H2-05 not shipped)." >&2
        echo "  NestGate is alive on port $NESTGATE_PORT but lacks the content pipeline." >&2
        echo "  See: specs/TUNNEL_EVOLUTION.md Step 3a" >&2
        echo "  See: specs/EVOLUTION_GAPS.md H2-05" >&2
        exit 1
    fi
}

build_sporeprint() {
    log "Building sporePrint with Zola..."
    if [[ ! -d "$SPOREPRINT_REPO" ]]; then
        echo "ERROR: sporePrint repo not found at $SPOREPRINT_REPO" >&2
        exit 1
    fi
    cd "$SPOREPRINT_REPO"
    zola build
    log "  Built to $SPOREPRINT_BUILD ($(find "$SPOREPRINT_BUILD" -type f | wc -l) files)"
}

publish_to_nestgate() {
    log "Publishing to NestGate content pipeline..."
    local count=0
    local manifest=""

    while IFS= read -r -d '' file; do
        local relpath="${file#$SPOREPRINT_BUILD/}"
        local b3hash
        b3hash=$(b3sum "$file" | cut -d' ' -f1)

        if $DRY_RUN; then
            echo "  [dry-run] content.put: $relpath ($b3hash)"
        else
            local data
            data=$(base64 < "$file")
            rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"content.put\",\"params\":{\"path\":\"$relpath\",\"data\":\"$data\",\"blake3\":\"$b3hash\"},\"id\":$count}"
        fi

        manifest+="$b3hash $relpath\n"
        count=$((count + 1))
    done < <(find "$SPOREPRINT_BUILD" -type f -print0)

    log "  Published $count files"
    echo -e "$manifest" > "/tmp/sporeprint-manifest-$VERSION.txt"
    log "  Manifest: /tmp/sporeprint-manifest-$VERSION.txt"
}

create_collection() {
    log "Creating collection: sporeprint-$VERSION"
    if $DRY_RUN; then
        echo "  [dry-run] Would create collection sporeprint-$VERSION"
        return
    fi
    # collection.create wraps the manifest into a named, versioned set
    rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"collection.create\",\"params\":{\"name\":\"sporeprint\",\"version\":\"$VERSION\",\"manifest\":\"/tmp/sporeprint-manifest-$VERSION.txt\"},\"id\":1}"
    log "  Collection created"
}

main() {
    log "=== sporePrint → NestGate publish pipeline ==="
    log "Version: $VERSION"
    log "Dry run: $DRY_RUN"
    log ""

    check_nestgate_content_support
    build_sporeprint
    publish_to_nestgate
    create_collection

    log ""
    log "=== Publish complete ==="
    log "Next: run nestgate_content_parity.sh to verify shadow parity"
    log "  infra/benchScale/scenarios/nestgate_content_parity.sh \\"
    log "    --ghpages-url https://primals.eco \\"
    log "    --nestgate-url http://127.0.0.1:9901"
}

main
