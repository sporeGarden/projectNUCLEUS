#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# gate_trust_validator.sh — Deploy Graph Trust Validation
#
# Cross-validates gate TOMLs, gate_manifest.toml, and deploy graph
# TOMLs to ensure consistency. gate.toml is the authority for what
# primals run on each gate.
#
# Usage:
#   ./validation/gate_trust_validator.sh
#
# Evolution: bash (now) → nucleus-deploy verify --trust (Rust)

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MANIFEST="$PROJECT_ROOT/deploy/gate_manifest.toml"
REGISTRY="$PROJECT_ROOT/deploy/nucleus-primals/src/lib.rs"

GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'
NC='\033[0m'

PASS=0
FAIL=0
WARN=0

pass() { echo -e "  ${GREEN}PASS${NC} $*"; PASS=$((PASS + 1)); }
fail() { echo -e "  ${RED}FAIL${NC} $*"; FAIL=$((FAIL + 1)); }
warn() { echo -e "  ${YELLOW}WARN${NC} $*"; WARN=$((WARN + 1)); }

# Extract primal slugs from a TOML primals = [...] array.
# Handles both single-line and multi-line arrays. Stops at next section or field.
extract_primals() {
    local file="$1"
    sed -n '/^\[composition\]/,/^\[/p' "$file" \
        | sed -n '/primals/,/\]/p' \
        | grep -oP '"[a-z][a-z0-9-]*"' \
        | tr -d '"' \
        | sort
}

# Extract primal names from a deploy graph.
# Handles [primals.*] sections and [[graph.nodes]] / [[fragment.nodes]] name fields.
extract_graph_primals() {
    local file="$1"
    local primals=""
    # [primals.*] sections
    primals=$(grep '^\[primals\.' "$file" 2>/dev/null | sed 's/\[primals\.\(.*\)\]/\1/' | sort || true)
    if [[ -n "$primals" ]]; then
        echo "$primals"
        return
    fi
    # [[graph.nodes]] or [[fragment.nodes]] name fields
    grep -A2 '^\[\[.*nodes\]\]' "$file" 2>/dev/null \
        | grep '^name' \
        | sed 's/.*= *"\(.*\)"/\1/' \
        | sort -u || true
}

echo "=== Deploy Graph Trust Validation ==="
echo "Manifest: $MANIFEST"
echo

GATES=$(grep '^\[\[gates\]\]' -A1 "$MANIFEST" | grep 'name' | sed 's/.*= *"\(.*\)"/\1/')

for gate in $GATES; do
    section=$(sed -n "/name = \"$gate\"/,/^\[\[gates\]\]/p" "$MANIFEST" | sed '${ /^\[\[gates\]\]/d }')
    if [[ -z "$section" ]]; then
        section=$(sed -n "/name = \"$gate\"/,\$p" "$MANIFEST")
    fi

    manifest_count=$(echo "$section" | { grep '^primals' || true; } | head -1 | sed 's/.*= *//')
    manifest_status=$(echo "$section" | { grep '^status' || true; } | head -1 | sed 's/.*= *"\(.*\)"/\1/')
    manifest_graph=$(echo "$section" | { grep '^graph' || true; } | head -1 | sed 's/.*= *"\(.*\)"/\1/')
    manifest_config=$(echo "$section" | { grep '^config' || true; } | head -1 | sed 's/.*= *"\(.*\)"/\1/')

    echo "--- $gate (status: $manifest_status) ---"

    # Resolve paths relative to deploy/
    gate_toml="$PROJECT_ROOT/deploy/$manifest_config"
    if [[ ! -f "$gate_toml" ]]; then
        fail "gate TOML not found: $manifest_config"
        echo
        continue
    fi
    pass "gate TOML exists"

    graph_toml=""
    if [[ -n "$manifest_graph" ]]; then
        graph_toml="$PROJECT_ROOT/deploy/$manifest_graph"
        if [[ -f "$graph_toml" ]]; then
            pass "deploy graph exists"
        else
            fail "deploy graph not found: $manifest_graph"
        fi
    fi

    # Extract actual primals from gate TOML
    gate_primals=$(extract_primals "$gate_toml")
    gate_count=$(echo "$gate_primals" | grep -c . || true)

    # Validate manifest count vs gate TOML count
    if [[ "$manifest_count" == "$gate_count" ]]; then
        pass "primal count: manifest=$manifest_count, gate=$gate_count"
    else
        fail "primal count mismatch: manifest=$manifest_count, gate=$gate_count"
    fi

    # Validate gate primals exist in nucleus-primals registry
    for primal in $gate_primals; do
        if grep -q "slug: \"$primal\"" "$REGISTRY"; then
            pass "'$primal' in registry"
        else
            fail "'$primal' NOT in nucleus-primals registry"
        fi
    done

    # Cross-validate gate primals against deploy graph
    if [[ -n "$graph_toml" && -f "$graph_toml" ]]; then
        graph_primals=$(extract_graph_primals "$graph_toml")

        if [[ -n "$graph_primals" ]]; then
            # Gate primal should appear in graph
            for primal in $gate_primals; do
                if echo "$graph_primals" | grep -qx "$primal"; then
                    pass "'$primal' authorized in deploy graph"
                else
                    case "$primal" in
                        biomeos) pass "'$primal' (orchestrator, implicit)" ;;
                        *) warn "'$primal' in gate TOML but not in deploy graph" ;;
                    esac
                fi
            done

            # Graph primals must be authorized by gate TOML
            for gp in $graph_primals; do
                case "$gp" in
                    biomeos_neural_api) continue ;;
                esac
                if ! echo "$gate_primals" | grep -qx "$gp"; then
                    fail "graph primal '$gp' NOT authorized by gate TOML"
                fi
            done
        fi
    fi

    echo
done

echo "=== Summary ==="
echo -e "${GREEN}PASS${NC}: $PASS  ${RED}FAIL${NC}: $FAIL  ${YELLOW}WARN${NC}: $WARN"

if [[ $FAIL -gt 0 ]]; then
    echo -e "\n${RED}TRUST VALIDATION: FAILED${NC}"
    exit 1
else
    echo -e "\n${GREEN}TRUST VALIDATION: PASSED${NC}"
fi
