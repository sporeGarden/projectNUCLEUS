#!/usr/bin/env bash
# forgejo_mirror.sh — Create Forgejo repos and configure dual-push remotes
#
# Usage:
#   bash forgejo_mirror.sh [--dry-run]     # Preview what would be done
#   bash forgejo_mirror.sh                 # Execute mirror setup
#   bash forgejo_mirror.sh --push-all      # Push all repos to Forgejo
#
# Requires: .netrc with Forgejo credentials, curl, git

set -uo pipefail

FORGEJO_URL="${FORGEJO_URL:-http://127.0.0.1:3000}"
FORGEJO_TOKEN="${FORGEJO_TOKEN:-}"
ECOPRIMALS_ROOT="${ECOPRIMALS_ROOT:-$HOME/Development/ecoPrimals}"
DRY_RUN=false
PUSH_ALL=false

if [[ -z "$FORGEJO_TOKEN" ]]; then
  echo "ERROR: FORGEJO_TOKEN env var required for API operations"
  echo "Usage: FORGEJO_TOKEN=<token> bash $0 [--dry-run] [--push-all]"
  exit 1
fi
AUTH_HEADER="Authorization: token $FORGEJO_TOKEN"

for arg in "$@"; do
  case "$arg" in
    --dry-run) DRY_RUN=true ;;
    --push-all) PUSH_ALL=true ;;
  esac
done

declare -A REPO_MAP
# sporeGarden org (already exists in Forgejo)
REPO_MAP["sporeGarden/projectNUCLEUS"]="sporeGarden/projectNUCLEUS"
REPO_MAP["sporeGarden/cellMembrane"]="sporeGarden/cellMembrane"
REPO_MAP["sporeGarden/foundation"]="sporeGarden/foundation"
REPO_MAP["sporeGarden/lithoSpore"]="sporeGarden/lithoSpore"
REPO_MAP["sporeGarden/esotericWebb"]="sporeGarden/esotericWebb"

# ecoPrimals org
REPO_MAP["infra/plasmidBin"]="ecoPrimals/plasmidBin"
REPO_MAP["infra/wateringHole"]="ecoPrimals/wateringHole"
REPO_MAP["infra/sporePrint"]="ecoPrimals/sporePrint"
REPO_MAP["infra/whitePaper"]="ecoPrimals/whitePaper"
REPO_MAP["primals/bearDog"]="ecoPrimals/bearDog"
REPO_MAP["primals/songBird"]="ecoPrimals/songBird"
REPO_MAP["primals/toadStool"]="ecoPrimals/toadStool"
REPO_MAP["primals/nestGate"]="ecoPrimals/nestGate"
REPO_MAP["primals/squirrel"]="ecoPrimals/squirrel"
REPO_MAP["primals/rhizoCrypt"]="ecoPrimals/rhizoCrypt"
REPO_MAP["primals/loamSpine"]="ecoPrimals/loamSpine"
REPO_MAP["primals/sweetGrass"]="ecoPrimals/sweetGrass"
REPO_MAP["primals/biomeOS"]="ecoPrimals/biomeOS"
REPO_MAP["primals/petalTongue"]="ecoPrimals/petalTongue"
REPO_MAP["primals/skunkBat"]="ecoPrimals/skunkBat"
REPO_MAP["primals/barraCuda"]="ecoPrimals/barraCuda"
REPO_MAP["primals/coralReef"]="ecoPrimals/coralReef"
REPO_MAP["primals/bingoCube"]="ecoPrimals/bingoCube"
REPO_MAP["primals/sourDough"]="ecoPrimals/sourDough"

# syntheticChemistry org
REPO_MAP["springs/primalSpring"]="syntheticChemistry/primalSpring"
REPO_MAP["springs/wetSpring"]="syntheticChemistry/wetSpring"
REPO_MAP["springs/hotSpring"]="syntheticChemistry/hotSpring"
REPO_MAP["springs/groundSpring"]="syntheticChemistry/groundSpring"
REPO_MAP["springs/airSpring"]="syntheticChemistry/airSpring"
REPO_MAP["springs/neuralSpring"]="syntheticChemistry/neuralSpring"
REPO_MAP["springs/ludoSpring"]="syntheticChemistry/ludoSpring"
REPO_MAP["springs/healthSpring"]="syntheticChemistry/healthSpring"

created=0
skipped=0
remote_added=0
pushed=0
failed=0

for local_path in "${!REPO_MAP[@]}"; do
  forgejo_path="${REPO_MAP[$local_path]}"
  org="${forgejo_path%%/*}"
  repo="${forgejo_path##*/}"
  full_local="$ECOPRIMALS_ROOT/$local_path"

  if [[ ! -d "$full_local/.git" ]]; then
    echo "SKIP  $local_path — not a git repo"
    ((skipped++))
    continue
  fi

  # Create repo in Forgejo if it doesn't exist
  exists=$(curl -sf -o /dev/null -w '%{http_code}' -H "$AUTH_HEADER" "$FORGEJO_URL/api/v1/repos/$forgejo_path" 2>/dev/null || echo "000")
  if [[ "$exists" != "200" ]]; then
    if $DRY_RUN; then
      echo "WOULD CREATE  $forgejo_path"
    else
      result=$(curl -sf -X POST "$FORGEJO_URL/api/v1/orgs/$org/repos" \
        -H "$AUTH_HEADER" \
        -H "Content-Type: application/json" \
        -d "{\"name\": \"$repo\", \"private\": true, \"default_branch\": \"main\"}" 2>&1 || echo '{"message":"failed"}')
      if echo "$result" | grep -q '"full_name"' 2>/dev/null; then
        echo "CREATED  $forgejo_path"
        ((created++))
      else
        msg=$(echo "$result" | python3 -c "import json,sys; print(json.load(sys.stdin).get('message','?'))" 2>/dev/null || echo "unknown error")
        echo "FAIL  $forgejo_path — $msg"
        ((failed++))
        continue
      fi
    fi
  else
    echo "EXISTS  $forgejo_path"
  fi

  # Add/update forgejo remote
  current_remote=$(git -C "$full_local" remote get-url forgejo 2>/dev/null || echo "")
  target_url="$FORGEJO_URL/$forgejo_path.git"

  if [[ -z "$current_remote" ]]; then
    if $DRY_RUN; then
      echo "  WOULD ADD remote forgejo -> $target_url"
    else
      git -C "$full_local" remote add forgejo "$target_url" 2>/dev/null || true
      echo "  REMOTE ADDED  forgejo -> $target_url"
      ((remote_added++))
    fi
  elif [[ "$current_remote" != "$target_url" ]]; then
    if $DRY_RUN; then
      echo "  WOULD UPDATE remote forgejo: $current_remote -> $target_url"
    else
      git -C "$full_local" remote set-url forgejo "$target_url"
      echo "  REMOTE UPDATED  forgejo -> $target_url"
      ((remote_added++))
    fi
  fi

  # Push if --push-all
  if $PUSH_ALL && ! $DRY_RUN; then
    default_branch=$(git -C "$full_local" rev-parse --abbrev-ref HEAD 2>/dev/null || echo "main")
    push_output=$(git -C "$full_local" push forgejo "$default_branch" 2>&1) && push_ok=true || push_ok=false
    if $push_ok; then
      echo "  PUSHED  $default_branch"
      ((pushed++))
    else
      echo "  PUSH FAILED  $forgejo_path: $push_output"
      ((failed++))
    fi
  fi
done

echo ""
echo "Summary: $created created, $remote_added remotes added, $pushed pushed, $skipped skipped, $failed failed"
