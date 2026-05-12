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
