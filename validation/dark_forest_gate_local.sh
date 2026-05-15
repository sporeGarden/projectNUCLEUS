#!/usr/bin/env bash
# Dark Forest Glacial Gate — Local deployment graph validation
#
# Mirrors primalSpring's s_dark_forest_gate scenario (Tier::Rust) but runs
# against projectNUCLEUS's local deploy graphs. All 5 pillars are checked
# structurally — no live primals required.
#
# Usage:
#   bash validation/dark_forest_gate_local.sh
#
# Exit code: 0 if all pillars PASS, 1 if any FAIL.

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
GRAPHS_DIR="$PROJECT_ROOT/graphs"

PASS=0; FAIL=0; WARN=0

pass()  { echo "  PASS  $1"; PASS=$((PASS + 1)); }
fail()  { echo "  FAIL  $1"; FAIL=$((FAIL + 1)); }
warn()  { echo "  WARN  $1"; WARN=$((WARN + 1)); }

section() { echo ""; echo "═══ $1 ═══"; }

DEPLOY_GRAPHS=(
    "$GRAPHS_DIR/nucleus_complete.toml"
    "$GRAPHS_DIR/node_atomic_compute.toml"
    "$GRAPHS_DIR/ionic_capability_share.toml"
    "$GRAPHS_DIR/basement_hpc_covalent.toml"
    "$GRAPHS_DIR/friend_remote_covalent.toml"
)

FRAGMENT_GRAPHS=(
    "$GRAPHS_DIR/tower_atomic.toml"
    "$GRAPHS_DIR/node_atomic.toml"
    "$GRAPHS_DIR/nest_atomic.toml"
    "$GRAPHS_DIR/nucleus.toml"
)

echo "Dark Forest Glacial Gate — Local Validation"
echo "Date: $(date -u +%Y-%m-%dT%H:%M:%SZ)"
echo "Graphs: $GRAPHS_DIR"

# ─── Pillar 1: Zero Metadata Leakage ────────────────────────────────────────
section "Pillar 1: Zero Metadata Leakage"

tower="$GRAPHS_DIR/tower_atomic.toml"
if grep -q 'security_model = "btsp"' "$tower"; then
    pass "tower_atomic declares security_model = btsp"
else
    fail "tower_atomic missing security_model = btsp"
fi

if grep -q 'beardog' "$tower" && grep -q 'songbird' "$tower"; then
    pass "tower_atomic includes BearDog + Songbird"
else
    fail "tower_atomic missing BearDog or Songbird"
fi

for frag in "${FRAGMENT_GRAPHS[@]}"; do
    fname="$(basename "$frag")"
    if grep -q 'security_model = "btsp"' "$frag"; then
        pass "$fname has security_model = btsp"
    else
        fail "$fname missing security_model = btsp"
    fi
done

# ─── Pillar 2: Zero Port Exposure ───────────────────────────────────────────
section "Pillar 2: Zero Port Exposure"

tier5="${PRIMALSPRING_TCP_TIER5:-}"
if [[ -z "$tier5" || "$tier5" == "0" || "$tier5" == "false" ]]; then
    pass "PRIMALSPRING_TCP_TIER5 is unset/off (zero-port standard)"
else
    fail "PRIMALSPRING_TCP_TIER5 is set to '$tier5' — violates zero-port default"
fi

source "$PROJECT_ROOT/deploy/nucleus_config.sh" 2>/dev/null || true
port_collision=0
declare -A port_primal_map
for primal in "${!PRIMAL_PORTS[@]}"; do
    port="${PRIMAL_PORTS[$primal]}"
    if [[ -n "${port_primal_map[$port]:-}" ]]; then
        echo "  NOTE  Port $port assigned to both ${port_primal_map[$port]} and $primal"
        port_collision=1
    fi
    port_primal_map[$port]="$primal"
done

if [[ $port_collision -eq 0 ]]; then
    pass "No port collisions in PRIMAL_PORTS (${#PRIMAL_PORTS[@]} primals, ${#port_primal_map[@]} unique ports)"
else
    fail "Port collision detected in nucleus_config.sh PRIMAL_PORTS"
fi

for graph in "${DEPLOY_GRAPHS[@]}"; do
    fname="$(basename "$graph")"
    transport=$(grep -oP 'transport\s*=\s*"\K[^"]+' "$graph" 2>/dev/null | head -1)
    if [[ "$transport" == "uds_only" || "$transport" == "uds_preferred" ]]; then
        pass "$fname transport = $transport"
    elif [[ -z "$transport" ]]; then
        warn "$fname has no transport field"
    else
        fail "$fname transport = $transport (expected uds_only or uds_preferred)"
    fi
done

# ─── Pillar 3: Songbird as Sole Network Surface ─────────────────────────────
section "Pillar 3: Songbird as Sole Network Surface"

if grep -q 'songbird' "$tower"; then
    pass "Songbird present in tower_atomic"
else
    fail "Songbird missing from tower_atomic"
fi

songbird_has_http=0
songbird_has_discovery=0
while IFS= read -r line; do
    if echo "$line" | grep -qP 'http\.'; then
        songbird_has_http=1
    fi
    if echo "$line" | grep -qP 'discovery\.'; then
        songbird_has_discovery=1
    fi
done < <(grep -A20 'name = "songbird"' "$tower")

[[ $songbird_has_http -eq 1 ]] && pass "Songbird owns http.* capabilities" || fail "Songbird missing http capabilities"
[[ $songbird_has_discovery -eq 1 ]] && pass "Songbird owns discovery.* capabilities" || fail "Songbird missing discovery capabilities"

non_songbird_http=0
for frag in "${FRAGMENT_GRAPHS[@]}"; do
    fname="$(basename "$frag")"
    in_songbird=0
    while IFS= read -r line; do
        if echo "$line" | grep -qP 'name\s*=\s*"songbird"'; then
            in_songbird=1
        elif echo "$line" | grep -qP 'name\s*=\s*"'; then
            in_songbird=0
        fi
        if [[ $in_songbird -eq 0 ]] && echo "$line" | grep -qP '"(http\.[^"]*|tls\.[^"]*)"'; then
            echo "  NOTE  $fname: non-Songbird node has http/tls capability: $line"
            non_songbird_http=1
        fi
    done < "$frag"
done

if [[ $non_songbird_http -eq 0 ]]; then
    pass "No non-Songbird nodes advertise http/tls capabilities in fragments"
else
    fail "Non-Songbird nodes found with http/tls capabilities"
fi

# ─── Pillar 4: BTSP Crypto Integrity ────────────────────────────────────────
section "Pillar 4: BTSP Crypto Integrity"

for graph in "${DEPLOY_GRAPHS[@]}"; do
    fname="$(basename "$graph")"
    if grep -q 'secure_by_default = true' "$graph"; then
        pass "$fname has secure_by_default = true"
    else
        fail "$fname missing secure_by_default = true in metadata"
    fi
done

for graph in "${DEPLOY_GRAPHS[@]}"; do
    fname="$(basename "$graph")"
    if grep -q 'bonding.*btsp_required\|security_model = "btsp_enforced"' "$graph"; then
        pass "$fname enforces BTSP bonding"
    else
        warn "$fname: could not confirm BTSP bonding policy"
    fi
done

chacha_found=0
for frag in "${FRAGMENT_GRAPHS[@]}"; do
    if grep -q 'crypto.encrypt_chacha20_poly1305' "$frag"; then
        chacha_found=1
        break
    fi
done
[[ $chacha_found -eq 1 ]] && pass "ChaCha20-Poly1305 AEAD registered in fragments" || fail "ChaCha20-Poly1305 not found in any fragment"

# ─── Pillar 5: Enclave Computing ────────────────────────────────────────────
section "Pillar 5: Enclave Computing"

nest="$GRAPHS_DIR/nest_atomic.toml"
if grep -q 'nestgate' "$nest"; then
    pass "NestGate present in nest_atomic"
else
    fail "NestGate missing from nest_atomic"
fi

if grep -q 'by_capability = "storage"' "$nest"; then
    pass "NestGate owns storage capability in nest_atomic"
else
    fail "NestGate not assigned storage capability"
fi

prov_primals=0
for p in rhizocrypt loamspine sweetgrass; do
    if grep -q "$p" "$nest"; then
        prov_primals=$((prov_primals + 1))
    fi
done

if [[ $prov_primals -eq 3 ]]; then
    pass "Provenance trio (rhizoCrypt + loamSpine + sweetGrass) present in nest_atomic"
else
    fail "Provenance trio incomplete ($prov_primals/3 found in nest_atomic)"
fi

if grep -q 'by_capability = "dag"' "$nest"; then
    pass "DAG capability assigned (rhizoCrypt lineage)"
else
    fail "DAG capability not assigned in nest_atomic"
fi

if grep -q 'by_capability = "attribution"' "$nest"; then
    pass "Attribution capability assigned (sweetGrass opaque agents)"
else
    fail "Attribution capability not assigned in nest_atomic"
fi

# ─── Summary ────────────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════"
echo "Dark Forest Glacial Gate — Results"
echo "  PASS: $PASS"
echo "  FAIL: $FAIL"
echo "  WARN: $WARN"
echo "═══════════════════════════════════════════════════"

if [[ $FAIL -eq 0 ]]; then
    echo "GATE STATUS: PASS — all 5 pillars validated structurally"
    exit 0
else
    echo "GATE STATUS: FAIL — $FAIL checks failed"
    exit 1
fi
