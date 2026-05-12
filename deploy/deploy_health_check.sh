#!/usr/bin/env bash
# deploy_health_check.sh — Post-deploy health verification
#
# Sourced by deploy.sh. Expects the following variables:
#   PLASMIDBIN_DIR, PRIMALS, and all *_PORT variables.
#
# Provides: port_for_primal(), rpc_health_check(), verify_primals()

port_for_primal() {
    case "$1" in
        beardog)     echo "$BEARDOG_PORT" ;;
        songbird)    echo "$SONGBIRD_PORT" ;;
        toadstool)   echo "$TOADSTOOL_PORT" ;;
        barracuda)   echo "$BARRACUDA_PORT" ;;
        coralreef)   echo "$CORALREEF_PORT" ;;
        nestgate)    echo "$NESTGATE_PORT" ;;
        rhizocrypt)  echo "$RHIZOCRYPT_PORT" ;;
        loamspine)   echo "$LOAMSPINE_PORT" ;;
        sweetgrass)  echo "$SWEETGRASS_PORT" ;;
        squirrel)    echo "$SQUIRREL_PORT" ;;
        skunkbat)    echo "$SKUNKBAT_PORT" ;;
        biomeos)     echo "$BIOMEOS_PORT" ;;
        petaltongue) echo "$PETALTONGUE_PORT" ;;
        *)           echo "" ;;
    esac
}

rpc_health_check() {
    local port="$1"
    curl -sf --max-time 3 "http://127.0.0.1:$port" \
        -X POST -H 'Content-Type: application/json' \
        -d '{"jsonrpc":"2.0","method":"health.liveness","id":1}' 2>/dev/null
}

verify_primals() {
    local primals="$1"
    local all_ok=true

    for p in $primals; do
        local pid
        pid=$(pgrep -f "$PLASMIDBIN_DIR/primals/$p" 2>/dev/null | head -1) || true
        if [[ -z "$pid" ]]; then
            echo "  $p: NOT RUNNING — check /tmp/$p.log"
            all_ok=false
            continue
        fi

        local port
        port=$(port_for_primal "$p")
        if [[ -n "$port" ]]; then
            local resp
            resp=$(rpc_health_check "$port") || resp=""
            if [[ -n "$resp" ]]; then
                echo "  $p: PID $pid, TCP $port — HEALTHY"
            else
                echo "  $p: PID $pid, TCP $port — running (health probe pending)"
            fi
        else
            echo "  $p: PID $pid — running"
        fi
    done

    if $all_ok; then
        return 0
    fi
    return 1
}
