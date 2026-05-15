#!/usr/bin/env bash
# deploy_graph.sh — Graph-driven primal deployment
#
# Reads a NUCLEUS graph TOML and starts primals in dependency order,
# using the graph's node metadata for ports, binaries, and capabilities.
#
# This replaces the hardcoded nohup-per-primal loop in deploy.sh with
# the composition.deploy(graph) pattern that biomeOS uses internally.
#
# Usage:
#   source deploy_graph.sh
#   deploy_from_graph "$GRAPH_FILE" "$PLASMIDBIN_DIR" "$RUNTIME_DIR" "$BIND_ADDRESS"
#
# The graph TOML format (from primalSpring) defines:
#   [[graph.nodes]] — name, binary, order, depends_on, tcp_fallback_port, capabilities
#
# Prerequisites: nucleus_config.sh sourced, PLASMIDBIN_DIR set

set -euo pipefail

parse_graph_nodes() {
    local graph_file="$1"

    python3 -c "
import sys
try:
    import tomllib
except ImportError:
    import tomli as tomllib

with open('$graph_file', 'rb') as f:
    graph = tomllib.load(f)

nodes = graph.get('graph', {}).get('nodes', [])
if not nodes:
    nodes = graph.get('fragment', {}).get('nodes', [])
nodes.sort(key=lambda n: n.get('order', 999))

for node in nodes:
    name = node.get('name', '')
    binary = node.get('binary', name)
    order = node.get('order', 999)
    port = node.get('tcp_fallback_port', 0)
    required = 'true' if node.get('required', False) else 'false'
    spawn = 'true' if node.get('spawn', True) else 'false'
    deps = ','.join(node.get('depends_on', []))
    caps = ','.join(node.get('capabilities', []))
    health = node.get('health_method', 'health.liveness')
    print(f'{name}|{binary}|{order}|{port}|{required}|{spawn}|{deps}|{caps}|{health}')
"
}

wait_for_health() {
    local port="$1"
    local name="$2"
    local method="${3:-health.liveness}"
    local max_attempts=10
    local attempt=0

    while [[ $attempt -lt $max_attempts ]]; do
        if curl -sf --max-time 2 "http://127.0.0.1:$port" \
            -X POST -H 'Content-Type: application/json' \
            -d "{\"jsonrpc\":\"2.0\",\"method\":\"$method\",\"id\":1}" \
            >/dev/null 2>&1; then
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done
    return 1
}

start_primal_from_graph() {
    local name="$1"
    local binary="$2"
    local port="$3"
    local plasmidbin_dir="$4"
    local runtime_dir="$5"
    local bind_address="$6"
    local family_id="$7"

    local bin_path="$plasmidbin_dir/primals/$binary"
    if [[ ! -x "$bin_path" ]]; then
        echo "    WARN: Binary not found: $bin_path"
        return 1
    fi

    local socket="$runtime_dir/biomeos/${binary}-${family_id}.sock"

    case "$binary" in
        beardog)
            nohup "$bin_path" server \
                --socket "$socket" \
                --family-id "$family_id" \
                --listen "$bind_address:$port" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        songbird)
            nohup "$bin_path" server \
                --port "$port" \
                --socket "$socket" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        toadstool)
            nohup "$bin_path" server \
                --port "$port" \
                --family-id "$family_id" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        nestgate)
            nohup "$bin_path" daemon \
                --socket-only \
                --port "$port" \
                --bind "$bind_address" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        biomeos)
            nohup "$bin_path" neural-api \
                --port "$port" \
                --family-id "$family_id" \
                --btsp-optional \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        petaltongue)
            nohup "$bin_path" server \
                --port "$port" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        coralreef)
            nohup "$bin_path" server \
                --rpc-bind "$bind_address:$port" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        barracuda)
            nohup "$bin_path" server \
                --bind "$bind_address:$port" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        rhizocrypt)
            nohup "$bin_path" server \
                --port "$port" \
                --host "$bind_address" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        loamspine)
            nohup "$bin_path" server \
                --port "$port" \
                --bind-address "$bind_address" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        sweetgrass)
            nohup "$bin_path" server \
                --port "$port" \
                --http-address "$bind_address:$((port + 1))" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
        *)
            nohup "$bin_path" server \
                --port "$port" \
                > "/tmp/${binary}.log" 2>&1 &
            ;;
    esac

    echo $!
}

shadow_deploy() {
    local graph_file="$1"
    local plasmidbin_dir="$2"
    local bind_address="${3:-127.0.0.1}"
    local mode="${4:-fresh}"

    if [[ ! -f "$graph_file" ]]; then
        echo "ERROR: Graph file not found: $graph_file" >&2
        return 1
    fi

    local mode_label="Dry-Run Validation"
    [[ "$mode" == "live" ]] && mode_label="Live Composition Validation"

    echo "╔══════════════════════════════════════════════════════╗"
    echo "║  composition.deploy.shadow — $mode_label"
    echo "╚══════════════════════════════════════════════════════╝"
    echo ""
    echo "  Graph: $graph_file"
    echo "  Mode:  $mode"
    echo ""

    local nodes_valid=0
    local nodes_missing=0
    local nodes_skipped=0
    local nodes_live=0

    local all_names=()

    while IFS='|' read -r name binary order port required spawn deps caps health; do
        all_names+=("$name")

        if [[ "$spawn" == "false" ]]; then
            echo "  [$order] $name — external (skip)"
            nodes_skipped=$((nodes_skipped + 1))
            continue
        fi

        local bin_path="$plasmidbin_dir/primals/$binary"
        local status="OK"
        local issues=()

        if [[ ! -x "$bin_path" ]]; then
            issues+=("binary missing: $bin_path")
            status="MISSING"
        fi

        if [[ "$port" -gt 0 ]]; then
            if ss -tlnp 2>/dev/null | grep -q ":$port "; then
                if [[ "$mode" == "live" ]]; then
                    local rpc_resp
                    rpc_resp=$(curl -sf --max-time 2 "http://$bind_address:$port" \
                        -X POST -H 'Content-Type: application/json' \
                        -d "{\"jsonrpc\":\"2.0\",\"method\":\"$health\",\"id\":1}" 2>&1 || true)
                    if echo "$rpc_resp" | grep -q '"result"' 2>/dev/null; then
                        status="LIVE"
                    elif echo "$rpc_resp" | grep -q 'Authentication required\|jsonrpc' 2>/dev/null; then
                        status="LIVE"
                    elif timeout 1 bash -c "echo > /dev/tcp/$bind_address/$port" 2>/dev/null; then
                        status="LIVE"
                    else
                        issues+=("port $port occupied but unreachable")
                        status="UNHEALTHY"
                    fi
                else
                    issues+=("port $port already in use")
                    status="CONFLICT"
                fi
            fi
        fi

        if [[ -n "$deps" ]]; then
            IFS=',' read -ra dep_list <<< "$deps"
            for dep in "${dep_list[@]}"; do
                local dep_found=false
                for prev in "${all_names[@]}"; do
                    [[ "$prev" == "$dep" ]] && dep_found=true
                done
                if [[ "$dep_found" == "false" ]]; then
                    issues+=("dependency '$dep' not in graph")
                    [[ "$required" == "true" ]] && status="DEP_FAIL"
                fi
            done
        fi

        case "$status" in
            OK)
                echo "  [$order] $name (TCP $port, caps: $caps) — VALID"
                nodes_valid=$((nodes_valid + 1))
                ;;
            LIVE)
                echo "  [$order] $name (TCP $port) — LIVE ✓"
                nodes_live=$((nodes_live + 1))
                ;;
            *)
                echo "  [$order] $name — $status"
                for issue in "${issues[@]}"; do
                    echo "    ! $issue"
                done
                nodes_missing=$((nodes_missing + 1))
                ;;
        esac

    done < <(parse_graph_nodes "$graph_file")

    # Validate workload TOMLs via toadstool.validate if toadStool is running
    local toadstool_port=9400
    if curl -sf --max-time 2 "http://$bind_address:$toadstool_port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","id":1}' >/dev/null 2>&1; then
        echo ""
        echo "  toadStool reachable on :$toadstool_port — validating workloads..."
        local workloads_dir
        workloads_dir="$(cd "$(dirname "$graph_file")/../workloads" 2>/dev/null && pwd || true)"
        if [[ -d "$workloads_dir" ]]; then
            local wl_valid=0 wl_fail=0
            for wl in "$workloads_dir"/**/*.toml; do
                [[ "$wl" == *templates* ]] && continue
                local resp
                resp=$(curl -sf --max-time 5 "http://$bind_address:$toadstool_port" \
                    -X POST -H 'Content-Type: application/json' \
                    -d "{\"jsonrpc\":\"2.0\",\"method\":\"toadstool.validate\",\"params\":{\"workload_path\":\"$wl\",\"dry_run\":true},\"id\":1}" 2>/dev/null || true)
                if echo "$resp" | grep -q '"valid":true' 2>/dev/null; then
                    wl_valid=$((wl_valid + 1))
                elif echo "$resp" | grep -q 'Authentication required' 2>/dev/null; then
                    wl_fail=$((wl_fail + 1))
                    echo "    AUTH: $(basename "$wl") — MethodGate requires BTSP token"
                else
                    wl_fail=$((wl_fail + 1))
                    echo "    WARN: $(basename "$wl") — validation failed or timeout"
                fi
            done
            echo "  Workloads: $wl_valid valid, $wl_fail auth/issues"
        fi
    else
        echo ""
        echo "  toadStool not reachable — skipping workload pre-flight"
    fi

    echo ""
    if [[ "$mode" == "live" ]]; then
        echo "  Live summary: $nodes_live LIVE, $nodes_valid ready, $nodes_missing issues, $nodes_skipped external"
    else
        echo "  Shadow summary: $nodes_valid valid, $nodes_missing issues, $nodes_skipped external"
    fi

    if [[ $nodes_missing -gt 0 ]]; then
        echo "  ⚠ Fix issues before live deploy"
        return 1
    fi

    if [[ "$mode" == "live" && $nodes_live -gt 0 ]]; then
        echo "  ✓ Composition validated — $nodes_live/$((nodes_live + nodes_valid + nodes_skipped)) nodes LIVE"
    else
        echo "  ✓ Graph ready for live deploy"
    fi
    return 0
}

deploy_from_graph() {
    local graph_file="$1"
    local plasmidbin_dir="$2"
    local runtime_dir="$3"
    local bind_address="$4"
    local family_id="${5:-}"

    if [[ ! -f "$graph_file" ]]; then
        echo "ERROR: Graph file not found: $graph_file" >&2
        return 1
    fi

    echo "=== Graph-Driven Deploy ==="
    echo "  Graph: $graph_file"
    echo ""

    mkdir -p "$runtime_dir/biomeos"
    for sock in "$runtime_dir"/biomeos/*.sock; do
        [[ -S "$sock" ]] && rm -f "$sock" 2>/dev/null || true
    done

    local nodes_started=0
    local nodes_failed=0
    local nodes_skipped=0

    while IFS='|' read -r name binary order port required spawn deps caps health; do
        if [[ "$spawn" == "false" ]]; then
            echo "  [$order] $name — spawn=false (external orchestrator)"
            nodes_skipped=$((nodes_skipped + 1))
            continue
        fi

        if [[ -n "$deps" ]]; then
            IFS=',' read -ra dep_list <<< "$deps"
            local deps_met=true
            for dep in "${dep_list[@]}"; do
                if ! pgrep -f "$plasmidbin_dir/primals/$dep" >/dev/null 2>&1; then
                    echo "  [$order] $name — dependency '$dep' not running"
                    if [[ "$required" == "true" ]]; then
                        deps_met=false
                    fi
                fi
            done
            if [[ "$deps_met" == "false" ]]; then
                echo "    FAIL: Required dependencies not met"
                nodes_failed=$((nodes_failed + 1))
                continue
            fi
        fi

        echo -n "  [$order] $name (TCP $port)..."

        local pid
        pid=$(start_primal_from_graph "$name" "$binary" "$port" \
              "$plasmidbin_dir" "$runtime_dir" "$bind_address" "$family_id") || {
            echo " FAILED (binary missing)"
            nodes_failed=$((nodes_failed + 1))
            continue
        }

        local wait_time=2
        [[ "$binary" == "beardog" || "$binary" == "biomeos" || "$binary" == "songbird" ]] && wait_time=3
        sleep "$wait_time"

        if kill -0 "$pid" 2>/dev/null; then
            echo " PID $pid"
            nodes_started=$((nodes_started + 1))

            if [[ "$port" -gt 0 ]]; then
                if wait_for_health "$port" "$name" "$health"; then
                    echo "    Health: OK ($health)"
                else
                    echo "    Health: pending (may need more time)"
                fi
            fi
        else
            echo " EXITED (check /tmp/$binary.log)"
            nodes_failed=$((nodes_failed + 1))
        fi

    done < <(parse_graph_nodes "$graph_file")

    echo ""
    echo "  Graph deploy complete: $nodes_started started, $nodes_failed failed, $nodes_skipped skipped"

    if [[ $nodes_failed -gt 0 ]]; then
        return 1
    fi
    return 0
}
