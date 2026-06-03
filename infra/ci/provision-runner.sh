#!/usr/bin/env bash
# SPDX-License-Identifier: AGPL-3.0-or-later
#
# provision-runner.sh — Register a Forgejo Actions self-hosted runner
#
# Installs forgejo-runner on ironGate (or any sovereign gate) and
# registers it with git.primals.eco. Runs as a systemd user service.
#
# Prerequisites:
#   - Forgejo instance at git.primals.eco with Actions enabled
#   - Runner registration token from Forgejo admin UI:
#     Site Admin → Actions → Runners → Create new runner
#   - Rust toolchain installed (via rustup)
#
# Usage:
#   ./provision-runner.sh --token <RUNNER_TOKEN> [--name irongate-runner]
#
# The runner registers with labels: self-hosted,linux,x86_64,rust
# so CI workflows can target it via:
#   runs-on: [self-hosted, linux, x86_64, rust]

set -euo pipefail

FORGEJO_INSTANCE="https://git.primals.eco"
RUNNER_VERSION="6.3.1"
RUNNER_NAME="${HOSTNAME:-irongate}-runner"
RUNNER_LABELS="self-hosted,linux,x86_64,rust"
RUNNER_DIR="$HOME/.forgejo-runner"
TOKEN=""

while [[ $# -gt 0 ]]; do
    case "$1" in
        --token) TOKEN="$2"; shift 2 ;;
        --name)  RUNNER_NAME="$2"; shift 2 ;;
        --version) RUNNER_VERSION="$2"; shift 2 ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

if [[ -z "$TOKEN" ]]; then
    echo "Error: --token required (get from Forgejo admin → Actions → Runners)"
    exit 1
fi

ARCH=$(uname -m)
case "$ARCH" in
    x86_64)  RUNNER_ARCH="amd64" ;;
    aarch64) RUNNER_ARCH="arm64" ;;
    *) echo "Unsupported arch: $ARCH"; exit 1 ;;
esac

echo "=== Forgejo Actions Runner Provisioning ==="
echo "Instance:  $FORGEJO_INSTANCE"
echo "Runner:    $RUNNER_NAME"
echo "Labels:    $RUNNER_LABELS"
echo "Directory: $RUNNER_DIR"
echo

mkdir -p "$RUNNER_DIR"
cd "$RUNNER_DIR"

RUNNER_BIN="forgejo-runner"
if ! command -v "$RUNNER_BIN" >/dev/null 2>&1; then
    echo "--- Downloading forgejo-runner v${RUNNER_VERSION} ---"
    DOWNLOAD_URL="https://code.forgejo.org/forgejo/runner/releases/download/v${RUNNER_VERSION}/forgejo-runner-${RUNNER_VERSION}-linux-${RUNNER_ARCH}"
    curl -fsSL -o "$RUNNER_BIN" "$DOWNLOAD_URL"
    chmod +x "$RUNNER_BIN"
    echo "Downloaded: $RUNNER_BIN"
else
    echo "forgejo-runner already installed: $(command -v $RUNNER_BIN)"
fi

echo
echo "--- Registering runner ---"
./"$RUNNER_BIN" register \
    --instance "$FORGEJO_INSTANCE" \
    --token "$TOKEN" \
    --name "$RUNNER_NAME" \
    --labels "$RUNNER_LABELS" \
    --no-interactive

echo
echo "--- Creating systemd user service ---"
SERVICE_DIR="$HOME/.config/systemd/user"
mkdir -p "$SERVICE_DIR"

cat > "$SERVICE_DIR/forgejo-runner.service" << UNIT
[Unit]
Description=Forgejo Actions Runner ($RUNNER_NAME)
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
WorkingDirectory=$RUNNER_DIR
ExecStart=$RUNNER_DIR/$RUNNER_BIN daemon
Restart=on-failure
RestartSec=10
Environment=HOME=$HOME
Environment=PATH=$HOME/.cargo/bin:/usr/local/bin:/usr/bin:/bin

[Install]
WantedBy=default.target
UNIT

systemctl --user daemon-reload
systemctl --user enable forgejo-runner.service
systemctl --user start forgejo-runner.service

echo
echo "--- Runner status ---"
systemctl --user status forgejo-runner.service --no-pager || true

echo
echo "=== Provisioning complete ==="
echo "Runner registered: $RUNNER_NAME"
echo "Labels: $RUNNER_LABELS"
echo "Service: systemctl --user {start|stop|status} forgejo-runner"
echo
echo "Workflows can now use:"
echo "  runs-on: [self-hosted, linux, x86_64, rust]"
