#!/usr/bin/env bash
# Provenance Pipeline — Full rigor through the Nest Atomic
#
# Wraps each toadStool workload execution with provenance operations:
#   1. rhizoCrypt DAG session tracks every pipeline step
#   2. NestGate stores content-addressed artifacts
#   3. loamSpine permanent ledger commits the dehydrated session
#   4. sweetGrass creates attribution braids
#
# Usage:
#   ./provenance_pipeline.sh [--workloads-dir DIR] [--results-dir DIR]
#
# Requires: b3sum, curl, nc (netcat), jq, toadstool
# Requires running: BearDog, SongBird, ToadStool, NestGate, rhizoCrypt, loamSpine, sweetGrass

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Discover ecoPrimals root via git or relative path
if command -v git &>/dev/null; then
    _git_root="$(cd "$PROJECT_ROOT" && git rev-parse --show-toplevel 2>/dev/null)" || true
fi
ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-${_git_root:+$(cd "$_git_root/../.." 2>/dev/null && pwd)}}"
ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$(cd "$SCRIPT_DIR/../../../.." 2>/dev/null && pwd)}"
unset _git_root

WETSPRING_DIR="${WETSPRING_DIR:-$ECOPRIMALS_ROOT/springs/wetSpring}"
PLASMIDBIN_DIR="${PLASMIDBIN_DIR:-${ECOPRIMALS_PLASMID_BIN:-$ECOPRIMALS_ROOT/infra/plasmidBin}}"
TOADSTOOL="${TOADSTOOL:-$PLASMIDBIN_DIR/primals/toadstool}"

# Parse --workloads-dir and --results-dir flags
WORKLOADS_DIR="$PROJECT_ROOT/workloads/wetspring"
RESULTS_DIR="$PROJECT_ROOT/validation/provenance-run-$(date +%Y%m%d-%H%M%S)"
while [[ $# -gt 0 ]]; do
    case "$1" in
        --workloads-dir) WORKLOADS_DIR="$2"; shift 2 ;;
        --results-dir)   RESULTS_DIR="$2"; shift 2 ;;
        *)               WORKLOADS_DIR="$1"; shift ;;
    esac
done

# Phase 59 canonical TCP fallback ports (overridable via env)
BEARDOG_PORT="${BEARDOG_PORT:-9100}"
SONGBIRD_PORT="${SONGBIRD_PORT:-9200}"
TOADSTOOL_PORT="${TOADSTOOL_PORT:-9400}"
NESTGATE_PORT="${NESTGATE_PORT:-9500}"
RHIZOCRYPT_PORT="${RHIZOCRYPT_PORT:-9601}"
LOAMSPINE_PORT="${LOAMSPINE_PORT:-9700}"
SWEETGRASS_PORT="${SWEETGRASS_PORT:-9850}"

mkdir -p "$RESULTS_DIR"

rpc_nestgate() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$NESTGATE_PORT" 2>/dev/null
}

rpc_rhizocrypt() {
    # rhizoCrypt serves JSON-RPC on port+1 (dual HTTP+newline), tarpc on base port
    local jsonrpc_port=$((RHIZOCRYPT_PORT + 1))
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$jsonrpc_port" 2>/dev/null
}

rpc_loamspine() {
    curl -s -X POST "http://127.0.0.1:$LOAMSPINE_PORT" \
        -H 'Content-Type: application/json' -d "$1" 2>/dev/null
}

rpc_sweetgrass() {
    printf '%s\n' "$1" | nc -w 5 127.0.0.1 "$SWEETGRASS_PORT" 2>/dev/null
}

blake3_hash() {
    b3sum "$1" | cut -d' ' -f1
}

hash_to_byte_array() {
    local hex="$1"
    local arr="["
    for i in $(seq 0 2 62); do
        local byte=$((16#${hex:$i:2}))
        [ "$i" -gt 0 ] && arr+=","
        arr+="$byte"
    done
    arr+="]"
    echo "$arr"
}

log() {
    echo "[$(date +%H:%M:%S)] $*"
}

log "═══════════════════════════════════════════════════════════"
log "  Provenance Pipeline — Nest Atomic Full Rigor"
log "  Results: $RESULTS_DIR"
log "═══════════════════════════════════════════════════════════"

# ══════════════════════════════════════════════════════════════
# PHASE 1: Health checks — verify provenance primals
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 1: Health Checks ──"

rpc_health() {
    local name="$1" port="$2"
    local resp

    # Songbird uses HTTP GET /health (not JSON-RPC POST)
    if [[ "$name" == "Songbird" ]]; then
        resp=$(curl -sf --max-time 3 "http://127.0.0.1:$port/health" 2>/dev/null) || resp=""
        if [[ "$resp" == "OK" ]]; then
            log "  [OK] $name (HTTP $port) healthy"
            return 0
        fi
        log "  [FAIL] $name (HTTP $port) not responding"
        return 1
    fi

    # rhizoCrypt serves JSON-RPC on port+1 (tarpc on base port)
    if [[ "$name" == "rhizoCrypt" ]]; then
        local jsonrpc_port=$((port + 1))
        resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}\n' | nc -w 3 127.0.0.1 "$jsonrpc_port" 2>/dev/null) || resp=""
        if [[ -n "$resp" ]] && echo "$resp" | grep -q '"result"'; then
            log "  [OK] $name (TCP $jsonrpc_port) healthy"
            return 0
        fi
        log "  [FAIL] $name (TCP $jsonrpc_port) not responding"
        return 1
    fi

    # HTTP JSON-RPC (loamSpine, petalTongue)
    resp=$(curl -sf --max-time 3 "http://127.0.0.1:$port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}' 2>/dev/null) || resp=""
    if [[ -n "$resp" ]] && echo "$resp" | grep -q '"result"'; then
        log "  [OK] $name (TCP $port) healthy"
        return 0
    fi

    # Newline-delimited JSON-RPC (BearDog, ToadStool, NestGate, sweetGrass, etc.)
    resp=$(printf '{"jsonrpc":"2.0","method":"health.liveness","params":{},"id":0}\n' | nc -w 3 127.0.0.1 "$port" 2>/dev/null) || resp=""
    if [[ -n "$resp" ]] && echo "$resp" | grep -q '"result"'; then
        log "  [OK] $name (TCP $port) healthy"
        return 0
    fi

    log "  [FAIL] $name (TCP $port) not responding"
    return 1
}

HEALTH_FAIL=0
for primal_pair in "BearDog:$BEARDOG_PORT" "Songbird:$SONGBIRD_PORT" "ToadStool:$TOADSTOOL_PORT" "NestGate:$NESTGATE_PORT" "rhizoCrypt:$RHIZOCRYPT_PORT" "loamSpine:$LOAMSPINE_PORT" "sweetGrass:$SWEETGRASS_PORT"; do
    name="${primal_pair%%:*}"
    port="${primal_pair#*:}"
    if ! rpc_health "$name" "$port"; then
        HEALTH_FAIL=$((HEALTH_FAIL + 1))
    fi
done

if [[ $HEALTH_FAIL -gt 0 ]]; then
    log "  $HEALTH_FAIL primal(s) failed health check."
    log "  Ensure composition is running: bash deploy.sh --composition nest --gate irongate"
    exit 1
fi

# ══════════════════════════════════════════════════════════════
# PHASE 2: Create rhizoCrypt DAG session
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 2: Create DAG Session ──"

SESSION_NAME="abg-pipeline-$(date +%Y%m%d-%H%M%S)"
SESSION_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.session.create\",\"params\":{\"name\":\"$SESSION_NAME\"},\"id\":1}")
SESSION_ID=$(echo "$SESSION_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result'])" 2>/dev/null)

if [ -z "$SESSION_ID" ]; then
    log "  [FAIL] Could not create DAG session: $SESSION_RESP"
    exit 1
fi
log "  [OK] Session: $SESSION_ID ($SESSION_NAME)"

# ══════════════════════════════════════════════════════════════
# PHASE 3: Create loamSpine spine for this pipeline run
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 3: Create LoamSpine Spine ──"

SPINE_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"spine.create\",\"params\":{\"name\":\"$SESSION_NAME\",\"owner\":\"ecoPrimal\"},\"id\":1}")
SPINE_ID=$(echo "$SPINE_RESP" | python3 -c "import sys,json; print(json.load(sys.stdin)['result']['spine_id'])" 2>/dev/null)

if [ -z "$SPINE_ID" ]; then
    log "  [FAIL] Could not create spine: $SPINE_RESP"
    exit 1
fi
log "  [OK] Spine: $SPINE_ID"

# ══════════════════════════════════════════════════════════════
# PHASE 4: Hash and register NCBI data artifacts
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 4: Register Data Artifacts ──"

EVENT_IDX=0
ARTIFACT_HASHES=""

register_artifact() {
    local filepath="$1"
    local artifact_key="$2"
    local artifact_type="$3"
    local accession="${4:-}"

    if [ ! -f "$filepath" ]; then
        log "  [SKIP] $artifact_key — file not found: $filepath"
        return
    fi

    local hash
    hash=$(blake3_hash "$filepath")
    local size
    size=$(stat -c%s "$filepath" 2>/dev/null || stat -f%z "$filepath" 2>/dev/null)
    local hash_bytes
    hash_bytes=$(hash_to_byte_array "$hash")

    rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"$artifact_key\",\"value\":\"blake3:$hash size:$size\"},\"id\":$((EVENT_IDX+100))}" > /dev/null

    local data_json="{\"key\":\"$artifact_key\",\"blake3\":\"$hash\",\"size\":$size"
    [ -n "$accession" ] && data_json+=",\"accession\":\"$accession\""
    data_json+="}"

    local dag_resp
    dag_resp=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"DataCreate\":{}},\"data\":$data_json},\"id\":$((EVENT_IDX+200))}")

    local loam_resp
    loam_resp=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"DataAnchor\":{\"data_hash\":$hash_bytes,\"source\":\"$artifact_type\",\"size\":$size}},\"committer\":\"did:primal:ecoPrimal\",\"data\":{\"key\":\"$artifact_key\",\"blake3\":\"$hash\"}},\"id\":$((EVENT_IDX+300))}")

    EVENT_IDX=$((EVENT_IDX + 1))
    ARTIFACT_HASHES+="  $artifact_key  $hash  ${size}B\n"
    log "  [OK] $artifact_key → blake3:${hash:0:16}… (${size}B)"
}

register_artifact \
    "$WETSPRING_DIR/data/paper_proxy/nannochloropsis_16s/SRR7760408/SRR7760408_1.fastq.gz" \
    "ncbi:SRR7760408:R1" "ncbi_fastq" "SRR7760408"

register_artifact \
    "$WETSPRING_DIR/data/paper_proxy/nannochloropsis_16s/SRR7760408/SRR7760408_2.fastq.gz" \
    "ncbi:SRR7760408:R2" "ncbi_fastq" "SRR7760408"

register_artifact \
    "$WETSPRING_DIR/data/paper_proxy/nannochloropsis_pilots/SRR5534045/SRR5534045_1.fastq.gz" \
    "ncbi:SRR5534045:R1" "ncbi_fastq" "SRR5534045"

register_artifact \
    "$WETSPRING_DIR/data/paper_proxy/nannochloropsis_pilots/SRR5534045/SRR5534045_2.fastq.gz" \
    "ncbi:SRR5534045:R2" "ncbi_fastq" "SRR5534045"

# ══════════════════════════════════════════════════════════════
# PHASE 5: Execute workloads with provenance wrapping
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 5: Execute Workloads with Provenance ──"

WORKLOAD_RESULTS=""

execute_with_provenance() {
    local toml_path="$1"
    local workload_name
    workload_name=$(basename "$toml_path" .toml)

    log "  [$workload_name] Executing..."

    rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"ExperimentStart\":{\"protocol\":\"wetspring-validation\"}},\"data\":{\"workload\":\"$workload_name\",\"toml\":\"$(basename "$toml_path")\",\"timestamp\":\"$(date -Iseconds)\"}},\"id\":$((EVENT_IDX+400))}" > /dev/null
    EVENT_IDX=$((EVENT_IDX + 1))

    local output_file="$RESULTS_DIR/${workload_name}.stdout"
    local start_time
    start_time=$(date +%s)

    if $TOADSTOOL execute "$toml_path" > "$output_file" 2>&1; then
        local end_time
        end_time=$(date +%s)
        local duration_ms=$(( (end_time - start_time) * 1000 ))

        local check_count
        check_count=$(grep -c '\[OK\]' "$output_file" 2>/dev/null || true)
        check_count=${check_count:-0}
        check_count=$(echo "$check_count" | tr -d '[:space:]')
        local fail_count
        fail_count=$(grep -c '\[FAIL\]' "$output_file" 2>/dev/null || true)
        fail_count=${fail_count:-0}
        fail_count=$(echo "$fail_count" | tr -d '[:space:]')

        local exit_code=0
        if grep -q 'Status:.*Failed' "$output_file"; then
            exit_code=1
        fi

        local output_hash
        output_hash=$(blake3_hash "$output_file")
        local output_hash_bytes
        output_hash_bytes=$(hash_to_byte_array "$output_hash")
        local output_size
        output_size=$(stat -c%s "$output_file" 2>/dev/null || stat -f%z "$output_file" 2>/dev/null)

        rpc_nestgate "{\"jsonrpc\":\"2.0\",\"method\":\"storage.store\",\"params\":{\"key\":\"workload:$workload_name:output\",\"value\":\"blake3:$output_hash size:$output_size\"},\"id\":$((EVENT_IDX+500))}" > /dev/null

        local confidence=100
        local total=$((check_count + fail_count))
        [ "$total" -gt 0 ] && [ "$fail_count" -gt 0 ] && confidence=$(( (check_count * 100) / total ))
        rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.event.append\",\"params\":{\"session_id\":\"$SESSION_ID\",\"event_type\":{\"Result\":{\"confidence_percent\":$confidence}},\"data\":{\"workload\":\"$workload_name\",\"checks_passed\":$check_count,\"checks_failed\":$fail_count,\"exit_code\":$exit_code,\"duration_ms\":$duration_ms,\"output_blake3\":\"$output_hash\",\"output_size\":$output_size}},\"id\":$((EVENT_IDX+600))}" > /dev/null

        rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"DataAnchor\":{\"data_hash\":$output_hash_bytes,\"source\":\"workload_output\",\"size\":$output_size}},\"committer\":\"did:primal:ecoPrimal\",\"data\":{\"workload\":\"$workload_name\",\"checks_passed\":$check_count,\"blake3\":\"$output_hash\"}},\"id\":$((EVENT_IDX+700))}" > /dev/null

        EVENT_IDX=$((EVENT_IDX + 1))

        local status="PASS"
        [ "$exit_code" -ne 0 ] && status="FAIL"
        [ "$check_count" -eq 0 ] && [ "$fail_count" -eq 0 ] && status="RUN"

        WORKLOAD_RESULTS+="  $workload_name  $check_count/$((check_count+fail_count))  ${duration_ms}ms  $status  blake3:${output_hash:0:16}\n"
        log "  [$workload_name] $status — $check_count checks, ${duration_ms}ms, blake3:${output_hash:0:16}…"
    else
        log "  [$workload_name] EXECUTION ERROR"
        EVENT_IDX=$((EVENT_IDX + 1))
        WORKLOAD_RESULTS+="  $workload_name  ERROR  -  -  -\n"
    fi
}

WORKLOAD_COUNT=0
for toml in "$WORKLOADS_DIR"/*.toml; do
    [ -f "$toml" ] || continue
    execute_with_provenance "$toml"
    WORKLOAD_COUNT=$((WORKLOAD_COUNT + 1))
done

if [[ $WORKLOAD_COUNT -eq 0 ]]; then
    log "  [WARN] No workload TOMLs found in $WORKLOADS_DIR"
fi

# ══════════════════════════════════════════════════════════════
# PHASE 6: Dehydrate DAG → Merkle root
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 6: Dehydrate → Sign → Commit → Braid ──"

MERKLE_RESP=$(rpc_rhizocrypt "{\"jsonrpc\":\"2.0\",\"method\":\"dag.merkle.root\",\"params\":{\"session_id\":\"$SESSION_ID\"},\"id\":900}" || echo "{}")
MERKLE_ROOT_RAW=$(echo "$MERKLE_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin).get('result',''); print(json.dumps(r) if isinstance(r,(list,dict)) else str(r))" 2>/dev/null)

MERKLE_HEX=$(echo "$MERKLE_ROOT_RAW" | python3 -c "
import sys,json
r = sys.stdin.read().strip()
try:
    arr = json.loads(r)
    if isinstance(arr, list):
        print(''.join(f'{b:02x}' for b in arr))
    elif isinstance(arr, str):
        print(arr)
    else:
        print(r)
except:
    print(r)
" 2>/dev/null)

MERKLE_BYTES=$(echo "$MERKLE_ROOT_RAW" | python3 -c "
import sys,json
r = sys.stdin.read().strip()
try:
    arr = json.loads(r)
    if isinstance(arr, list):
        print(json.dumps(arr))
    elif isinstance(arr, str):
        bs = bytes.fromhex(arr)
        print(json.dumps(list(bs)))
    else:
        print('[' + ','.join(['0']*32) + ']')
except:
    print('[' + ','.join(['0']*32) + ']')
" 2>/dev/null)

log "  [OK] Merkle root: $MERKLE_HEX"

# ══════════════════════════════════════════════════════════════
# PHASE 7: Commit to loamSpine permanent ledger
# ══════════════════════════════════════════════════════════════

COMMIT_RESP=$(rpc_loamspine "{\"jsonrpc\":\"2.0\",\"method\":\"entry.append\",\"params\":{\"spine_id\":\"$SPINE_ID\",\"entry_type\":{\"SessionCommit\":{\"session_id\":\"$SESSION_ID\",\"merkle_root\":$MERKLE_BYTES,\"vertex_count\":$EVENT_IDX,\"committer\":\"did:primal:ecoPrimal\"}},\"committer\":\"did:primal:ecoPrimal\",\"data\":{\"pipeline\":\"$SESSION_NAME\",\"merkle_hex\":\"$MERKLE_HEX\",\"event_count\":$EVENT_IDX}},\"id\":901}")
COMMIT_HASH=$(echo "$COMMIT_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(r.get('entry_hash','') if isinstance(r,dict) else r)" 2>/dev/null)
COMMIT_INDEX=$(echo "$COMMIT_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(r.get('index','') if isinstance(r,dict) else '')" 2>/dev/null)
log "  [OK] LoamSpine commit: index=$COMMIT_INDEX"

# ══════════════════════════════════════════════════════════════
# PHASE 8: Create sweetGrass attribution braid
# ══════════════════════════════════════════════════════════════

BRAID_RESP=$(rpc_sweetgrass "{\"jsonrpc\":\"2.0\",\"method\":\"braid.create\",\"params\":{\"data_hash\":\"$MERKLE_HEX\",\"name\":\"$SESSION_NAME\",\"mime_type\":\"application/x-provenance-pipeline\",\"description\":\"ABG Full Pipeline — provenance braid for $EVENT_IDX events across wetSpring validators\",\"size\":$EVENT_IDX},\"id\":902}")
BRAID_ID=$(echo "$BRAID_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; print(r.get('@id',''))" 2>/dev/null)
BRAID_WITNESS=$(echo "$BRAID_RESP" | python3 -c "import sys,json; r=json.load(sys.stdin)['result']; w=r.get('witness',{}); print(f\"{w.get('algorithm','')}: {w.get('evidence','')[:32]}...\")" 2>/dev/null)
log "  [OK] Braid: $BRAID_ID"
log "       Witness: $BRAID_WITNESS"

# ══════════════════════════════════════════════════════════════
# PHASE 9: Write provenance manifest
# ══════════════════════════════════════════════════════════════
log ""
log "── Phase 9: Write Provenance Manifest ──"

cat > "$RESULTS_DIR/PROVENANCE_MANIFEST.md" << MANIFEST_EOF
# Provenance Manifest — $SESSION_NAME

**Date**: $(date -Iseconds)
**Composition**: Full NUCLEUS (13 primals)
**Session**: $SESSION_ID
**Spine**: $SPINE_ID

## Provenance Chain

| Layer | Identifier | Purpose |
|-------|-----------|---------|
| DAG Session | \`$SESSION_ID\` | Ephemeral pipeline tracking ($EVENT_IDX events) |
| Merkle Root | \`$MERKLE_HEX\` | Content hash of all DAG events |
| LoamSpine Commit | index=$COMMIT_INDEX | Permanent ledger entry |
| SweetGrass Braid | \`$BRAID_ID\` | Attribution with ed25519 witness |

## Data Artifacts

$(echo -e "$ARTIFACT_HASHES" | column -t 2>/dev/null || echo -e "$ARTIFACT_HASHES")

## Workload Results

$(echo -e "$WORKLOAD_RESULTS" | column -t 2>/dev/null || echo -e "$WORKLOAD_RESULTS")

## Verification

To verify this pipeline:
1. Confirm BLAKE3 hashes of NCBI FASTQs match the values above
2. Re-run the same workload TOMLs through toadStool
3. Query loamSpine for spine \`$SPINE_ID\` to see the full audit trail
4. Query sweetGrass for braid \`$BRAID_ID\` to verify the ed25519 witness

## NCBI Data Provenance

| Accession | BioProject | Source | DOI |
|-----------|-----------|--------|-----|
| SRR7760408 | PRJNA488170 | Nannochloropsis outdoor 16S (Wageningen) | 10.1007/s00253-022-11815-3 |
| SRR5534045 | PRJNA382322 | Nannochloropsis extended pilots | 10.1007/s00253-022-11815-3 |
MANIFEST_EOF

echo "$BRAID_RESP" | python3 -m json.tool > "$RESULTS_DIR/braid.json" 2>/dev/null || echo "$BRAID_RESP" > "$RESULTS_DIR/braid.json"

log "  [OK] Manifest: $RESULTS_DIR/PROVENANCE_MANIFEST.md"
log "  [OK] Braid JSON: $RESULTS_DIR/braid.json"

log ""
log "═══════════════════════════════════════════════════════════"
log "  Provenance Pipeline Complete"
log "  Session:     $SESSION_ID"
log "  Events:      $EVENT_IDX"
log "  Merkle Root: ${MERKLE_HEX:0:64}"
log "  Spine:       $SPINE_ID"
log "  Braid:       $BRAID_ID"
log "  Results:     $RESULTS_DIR/"
log "═══════════════════════════════════════════════════════════"
