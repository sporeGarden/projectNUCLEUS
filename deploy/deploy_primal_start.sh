#!/usr/bin/env bash
# deploy_primal_start.sh — Per-primal startup logic
#
# Sourced by deploy.sh. Expects the following variables:
#   PLASMIDBIN_DIR, RUNTIME_DIR, BIND_ADDRESS, FAMILY_ID, NODE_ID,
#   BEARDOG_SOCKET, BEACON_SEED, UDS_ONLY, and all *_PORT variables.
#
# Wave 56 standard: nucleus_launcher --uds-only is the preferred VPS path.
# This script supports --uds-only by suppressing TCP port arguments.
#
# Provides: start_primal()

start_primal() {
    local p="$1"

    # Wave 56 UDS-only: suppress TCP ports (VPS standard)
    if ${UDS_ONLY:-false}; then
        BEARDOG_PORT=0; SONGBIRD_PORT=0; TOADSTOOL_PORT=0; BARRACUDA_PORT=0
        CORALREEF_PORT=0; NESTGATE_PORT=0; RHIZOCRYPT_PORT=0; LOAMSPINE_PORT=0
        SWEETGRASS_PORT=0; SQUIRREL_PORT=0; SKUNKBAT_PORT=0; BIOMEOS_PORT=0
        PETALTONGUE_PORT=0
    fi

    case "$p" in
        beardog)
            local transport=$( (( BEARDOG_PORT > 0 )) && echo "UDS + TCP $BEARDOG_PORT" || echo "UDS-only" )
            echo "  Starting beardog ($transport)..."
            export BEARDOG_FAMILY_SEED="$BEACON_SEED"
            local bd_args=(server --socket "$BEARDOG_SOCKET" --family-id "$FAMILY_ID")
            (( BEARDOG_PORT > 0 )) && bd_args+=(--listen "$BIND_ADDRESS:$BEARDOG_PORT")
            nohup "$PLASMIDBIN_DIR/primals/beardog" "${bd_args[@]}" > /tmp/beardog.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        songbird)
            local transport=$( (( SONGBIRD_PORT > 0 )) && echo "HTTP $SONGBIRD_PORT" || echo "UDS-only" )
            echo "  Starting songbird ($transport)..."
            export BEARDOG_SOCKET="$BEARDOG_SOCKET"
            export BEARDOG_MODE=direct
            export SONGBIRD_SECURITY_PROVIDER=beardog
            local sb_args=(server --socket "$RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock")
            (( SONGBIRD_PORT > 0 )) && sb_args+=(--port "$SONGBIRD_PORT")
            nohup "$PLASMIDBIN_DIR/primals/songbird" "${sb_args[@]}" > /tmp/songbird.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        toadstool)
            local transport=$( (( TOADSTOOL_PORT > 0 )) && echo "TCP $TOADSTOOL_PORT" || echo "UDS-only" )
            echo "  Starting toadstool ($transport)..."
            export TOADSTOOL_FAMILY_ID="$FAMILY_ID"
            export TOADSTOOL_NODE_ID="$NODE_ID"
            export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1
            export SPRINGS_ROOT="${SPRINGS_ROOT:-${ECOPRIMALS_ROOT:-}/springs}"
            export GATE_HOME="${GATE_HOME:-$HOME}"
            local ts_args=(server --family-id "$FAMILY_ID")
            (( TOADSTOOL_PORT > 0 )) && ts_args+=(--port "$TOADSTOOL_PORT")
            nohup "$PLASMIDBIN_DIR/primals/toadstool" "${ts_args[@]}" > /tmp/toadstool.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        barracuda)
            local transport=$( (( BARRACUDA_PORT > 0 )) && echo "TCP $BARRACUDA_PORT" || echo "UDS-only" )
            echo "  Starting barracuda ($transport)..."
            local bc_args=(server)
            (( BARRACUDA_PORT > 0 )) && bc_args+=(--bind "$BIND_ADDRESS:$BARRACUDA_PORT")
            nohup "$PLASMIDBIN_DIR/primals/barracuda" "${bc_args[@]}" > /tmp/barracuda.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        coralreef)
            local transport=$( (( CORALREEF_PORT > 0 )) && echo "RPC $CORALREEF_PORT" || echo "UDS-only" )
            echo "  Starting coralreef ($transport)..."
            local cr_args=(server)
            (( CORALREEF_PORT > 0 )) && cr_args+=(--rpc-bind "$BIND_ADDRESS:$CORALREEF_PORT")
            nohup "$PLASMIDBIN_DIR/primals/coralreef" "${cr_args[@]}" > /tmp/coralreef.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        nestgate)
            local transport=$( (( NESTGATE_PORT > 0 )) && echo "UDS + TCP $NESTGATE_PORT" || echo "UDS-only" )
            echo "  Starting nestgate ($transport)..."
            export NESTGATE_FAMILY_ID="$FAMILY_ID"
            export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(head -c 32 /dev/urandom | base64)}"
            local ng_args=(daemon --socket-only)
            (( NESTGATE_PORT > 0 )) && ng_args+=(--port "$NESTGATE_PORT" --bind "$BIND_ADDRESS")
            nohup "$PLASMIDBIN_DIR/primals/nestgate" "${ng_args[@]}" > /tmp/nestgate.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        rhizocrypt)
            local transport=$( (( RHIZOCRYPT_PORT > 0 )) && echo "TCP $RHIZOCRYPT_PORT" || echo "UDS-only" )
            echo "  Starting rhizocrypt ($transport)..."
            export FAMILY_SEED="$BEACON_SEED"
            local rc_args=(server)
            (( RHIZOCRYPT_PORT > 0 )) && rc_args+=(--port "$RHIZOCRYPT_PORT" --host "$BIND_ADDRESS")
            nohup "$PLASMIDBIN_DIR/primals/rhizocrypt" "${rc_args[@]}" > /tmp/rhizocrypt.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        loamspine)
            local transport=$( (( LOAMSPINE_PORT > 0 )) && echo "TCP $LOAMSPINE_PORT" || echo "UDS-only" )
            echo "  Starting loamspine ($transport)..."
            local ls_args=(server)
            (( LOAMSPINE_PORT > 0 )) && ls_args+=(--port "$LOAMSPINE_PORT" --bind-address "$BIND_ADDRESS")
            nohup "$PLASMIDBIN_DIR/primals/loamspine" "${ls_args[@]}" > /tmp/loamspine.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        sweetgrass)
            local transport=$( (( SWEETGRASS_PORT > 0 )) && echo "TCP $SWEETGRASS_PORT" || echo "UDS-only" )
            echo "  Starting sweetgrass ($transport)..."
            local sg_args=(server)
            (( SWEETGRASS_PORT > 0 )) && sg_args+=(--port "$SWEETGRASS_PORT" --http-address "$BIND_ADDRESS:$((SWEETGRASS_PORT + 1))")
            nohup "$PLASMIDBIN_DIR/primals/sweetgrass" "${sg_args[@]}" > /tmp/sweetgrass.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        squirrel)
            local transport=$( (( SQUIRREL_PORT > 0 )) && echo "TCP $SQUIRREL_PORT" || echo "UDS-only" )
            echo "  Starting squirrel ($transport)..."
            export CAPABILITY_REGISTRY_SOCKET="$RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock"
            local sq_args=(server)
            (( SQUIRREL_PORT > 0 )) && sq_args+=(--port "$SQUIRREL_PORT" --bind "$BIND_ADDRESS")
            nohup "$PLASMIDBIN_DIR/primals/squirrel" "${sq_args[@]}" > /tmp/squirrel.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        skunkbat)
            local transport=$( (( SKUNKBAT_PORT > 0 )) && echo "TCP $SKUNKBAT_PORT" || echo "UDS-only" )
            echo "  Starting skunkbat ($transport)..."
            local sk_args=(server)
            (( SKUNKBAT_PORT > 0 )) && sk_args+=(--port "$SKUNKBAT_PORT")
            nohup "$PLASMIDBIN_DIR/primals/skunkbat" "${sk_args[@]}" > /tmp/skunkbat.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        biomeos)
            local transport=$( (( BIOMEOS_PORT > 0 )) && echo "TCP $BIOMEOS_PORT" || echo "UDS-only" )
            echo "  Starting biomeos neural-api ($transport)..."
            local bo_args=(neural-api --family-id "$FAMILY_ID" --btsp-optional)
            (( BIOMEOS_PORT > 0 )) && bo_args+=(--port "$BIOMEOS_PORT")
            nohup "$PLASMIDBIN_DIR/primals/biomeos" "${bo_args[@]}" > /tmp/biomeos.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        petaltongue)
            local transport=$( (( PETALTONGUE_PORT > 0 )) && echo "TCP $PETALTONGUE_PORT" || echo "UDS-only" )
            echo "  Starting petaltongue server ($transport)..."
            local pt_args=(server)
            (( PETALTONGUE_PORT > 0 )) && pt_args+=(--port "$PETALTONGUE_PORT")
            nohup "$PLASMIDBIN_DIR/primals/petaltongue" "${pt_args[@]}" > /tmp/petaltongue.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        *)
            local primal_bin="$PLASMIDBIN_DIR/primals/$p"
            if [[ -x "$primal_bin" ]]; then
                echo "  Starting $p (discovered, attempting server --port)..."
                nohup "$primal_bin" server --port 0 \
                    > "/tmp/$p.log" 2>&1 &
                echo "    PID: $! (port auto-assigned — check /tmp/$p.log)"
                sleep 1
            else
                echo "  SKIP $p — no binary found"
            fi
            ;;
    esac
}
