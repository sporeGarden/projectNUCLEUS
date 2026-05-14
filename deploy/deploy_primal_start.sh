#!/usr/bin/env bash
# deploy_primal_start.sh — Per-primal startup logic
#
# Sourced by deploy.sh. Expects the following variables:
#   PLASMIDBIN_DIR, RUNTIME_DIR, BIND_ADDRESS, FAMILY_ID, NODE_ID,
#   BEARDOG_SOCKET, BEACON_SEED, and all *_PORT variables.
#
# Provides: start_primal()

start_primal() {
    local p="$1"

    case "$p" in
        beardog)
            echo "  Starting beardog (UDS + TCP $BEARDOG_PORT)..."
            export BEARDOG_FAMILY_SEED="$BEACON_SEED"
            nohup "$PLASMIDBIN_DIR/primals/beardog" server \
                --socket "$BEARDOG_SOCKET" \
                --family-id "$FAMILY_ID" \
                --listen "$BIND_ADDRESS:$BEARDOG_PORT" \
                > /tmp/beardog.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        songbird)
            echo "  Starting songbird (HTTP $SONGBIRD_PORT)..."
            export BEARDOG_SOCKET="$BEARDOG_SOCKET"
            export BEARDOG_MODE=direct
            export SONGBIRD_SECURITY_PROVIDER=beardog
            nohup "$PLASMIDBIN_DIR/primals/songbird" server \
                --port "$SONGBIRD_PORT" \
                --socket "$RUNTIME_DIR/biomeos/songbird-$FAMILY_ID.sock" \
                > /tmp/songbird.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        toadstool)
            echo "  Starting toadstool (TCP $TOADSTOOL_PORT)..."
            export TOADSTOOL_FAMILY_ID="$FAMILY_ID"
            export TOADSTOOL_NODE_ID="$NODE_ID"
            export TOADSTOOL_SECURITY_WARNING_ACKNOWLEDGED=1
            export SPRINGS_ROOT="${SPRINGS_ROOT:-${ECOPRIMALS_ROOT:-}/springs}"
            nohup "$PLASMIDBIN_DIR/primals/toadstool" server \
                --port "$TOADSTOOL_PORT" \
                --family-id "$FAMILY_ID" \
                > /tmp/toadstool.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        barracuda)
            echo "  Starting barracuda (TCP $BARRACUDA_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/barracuda" server \
                --bind "$BIND_ADDRESS:$BARRACUDA_PORT" \
                > /tmp/barracuda.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        coralreef)
            echo "  Starting coralreef (RPC $CORALREEF_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/coralreef" server \
                --rpc-bind "$BIND_ADDRESS:$CORALREEF_PORT" \
                > /tmp/coralreef.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        nestgate)
            echo "  Starting nestgate (UDS + TCP $NESTGATE_PORT)..."
            export NESTGATE_FAMILY_ID="$FAMILY_ID"
            export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(head -c 32 /dev/urandom | base64)}"
            nohup "$PLASMIDBIN_DIR/primals/nestgate" daemon \
                --socket-only \
                --port "$NESTGATE_PORT" \
                --bind "$BIND_ADDRESS" \
                > /tmp/nestgate.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        rhizocrypt)
            echo "  Starting rhizocrypt (TCP $RHIZOCRYPT_PORT, JSON-RPC $((RHIZOCRYPT_PORT+1)))..."
            export FAMILY_SEED="$BEACON_SEED"
            nohup "$PLASMIDBIN_DIR/primals/rhizocrypt" server \
                --port "$RHIZOCRYPT_PORT" \
                --host "$BIND_ADDRESS" \
                > /tmp/rhizocrypt.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        loamspine)
            echo "  Starting loamspine (TCP $LOAMSPINE_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/loamspine" server \
                --port "$LOAMSPINE_PORT" \
                --bind-address "$BIND_ADDRESS" \
                > /tmp/loamspine.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        sweetgrass)
            echo "  Starting sweetgrass (TCP $SWEETGRASS_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/sweetgrass" server \
                --port "$SWEETGRASS_PORT" \
                --http-address "$BIND_ADDRESS:$((SWEETGRASS_PORT + 1))" \
                > /tmp/sweetgrass.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        squirrel)
            echo "  Starting squirrel (TCP $SQUIRREL_PORT)..."
            export CAPABILITY_REGISTRY_SOCKET="$RUNTIME_DIR/biomeos/neural-api-$FAMILY_ID.sock"
            nohup "$PLASMIDBIN_DIR/primals/squirrel" server \
                --port "$SQUIRREL_PORT" \
                --bind "$BIND_ADDRESS" \
                > /tmp/squirrel.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        skunkbat)
            echo "  Starting skunkbat (TCP $SKUNKBAT_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/skunkbat" server \
                --port "$SKUNKBAT_PORT" \
                > /tmp/skunkbat.log 2>&1 &
            echo "    PID: $!"
            sleep 1
            ;;

        biomeos)
            echo "  Starting biomeos neural-api (TCP $BIOMEOS_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/biomeos" neural-api \
                --port "$BIOMEOS_PORT" \
                --family-id "$FAMILY_ID" \
                --btsp-optional \
                > /tmp/biomeos.log 2>&1 &
            echo "    PID: $!"
            sleep 2
            ;;

        petaltongue)
            echo "  Starting petaltongue server (TCP $PETALTONGUE_PORT)..."
            nohup "$PLASMIDBIN_DIR/primals/petaltongue" server \
                --port "$PETALTONGUE_PORT" \
                > /tmp/petaltongue.log 2>&1 &
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
