#!/usr/bin/env bash
# Cloudflare Access setup — gates /hub/* behind Zero Trust policy
#
# Prerequisites:
#   1. CF_API_TOKEN env var with Account:Access:Edit + Zone:Read permissions
#   2. Account ID from tunnel credentials
#
# This script will be replaced by `tunnelKeeper access setup` once the Rust crate is live.

set -euo pipefail

ACCOUNT_ID="${CF_ACCOUNT_ID:-5a7d7fef1355b283ad8b3a8a6582e291}"
ZONE_NAME="primals.eco"
APP_DOMAIN="lab.primals.eco"

if [ -z "${CF_API_TOKEN:-}" ]; then
    echo "CF_API_TOKEN not set. Generate one at:"
    echo "  https://dash.cloudflare.com/profile/api-tokens"
    echo "  Permissions needed: Account > Access: Apps and Policies > Edit"
    echo ""
    echo "Then: export CF_API_TOKEN=<token>"
    exit 1
fi

API="https://api.cloudflare.com/client/v4"
AUTH="Authorization: Bearer ${CF_API_TOKEN}"

echo "--- Creating Access Application for ${APP_DOMAIN}/hub/* ---"

# Create self-hosted Access Application
APP_RESPONSE=$(curl -s -X POST "${API}/accounts/${ACCOUNT_ID}/access/apps" \
    -H "${AUTH}" \
    -H "Content-Type: application/json" \
    --data '{
        "name": "ABG JupyterHub",
        "domain": "'"${APP_DOMAIN}"'",
        "path": "hub",
        "type": "self_hosted",
        "session_duration": "24h",
        "auto_redirect_to_identity": false,
        "app_launcher_visible": true
    }')

APP_ID=$(echo "${APP_RESPONSE}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',{}).get('id','FAILED'))")
echo "Access App ID: ${APP_ID}"

if [ "${APP_ID}" = "FAILED" ]; then
    echo "Failed to create Access App:"
    echo "${APP_RESPONSE}" | python3 -m json.tool
    exit 1
fi

# Create OTP policy (email-based one-time pin)
echo "--- Creating email OTP policy ---"
POLICY_RESPONSE=$(curl -s -X POST "${API}/accounts/${ACCOUNT_ID}/access/apps/${APP_ID}/policies" \
    -H "${AUTH}" \
    -H "Content-Type: application/json" \
    --data '{
        "name": "ABG Members (email OTP)",
        "decision": "allow",
        "include": [
            {
                "email_domain": {
                    "domain": "primals.eco"
                }
            }
        ],
        "require": [],
        "exclude": []
    }')

POLICY_ID=$(echo "${POLICY_RESPONSE}" | python3 -c "import sys,json; print(json.load(sys.stdin).get('result',{}).get('id','FAILED'))")
echo "Policy ID: ${POLICY_ID}"

echo ""
echo "--- Cloudflare Access configured ---"
echo "App: ${APP_DOMAIN}/hub/*"
echo "Auth: Email OTP for @primals.eco"
echo ""
echo "To add individual emails, update the policy include list:"
echo "  curl -X PUT '${API}/accounts/${ACCOUNT_ID}/access/apps/${APP_ID}/policies/${POLICY_ID}'"
echo ""
echo "tunnelKeeper will manage this once the Rust crate is live."
