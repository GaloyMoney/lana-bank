#!/usr/bin/env bash
set -euo pipefail

# Determine the correct podman socket to use
SYSTEM_SOCKET="/run/podman/podman.sock"
USER_SOCKET="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/podman/podman.sock"

if [ -S "$SYSTEM_SOCKET" ] && CONTAINER_HOST="unix://$SYSTEM_SOCKET" timeout 3s podman version >/dev/null 2>&1; then
    echo "unix://$SYSTEM_SOCKET"
elif [ -S "$USER_SOCKET" ] && CONTAINER_HOST="unix://$USER_SOCKET" timeout 3s podman version >/dev/null 2>&1; then
    echo "unix://$USER_SOCKET"
else
    # Default fallback (will likely fail, but provides a reasonable default)
    echo "unix://$SYSTEM_SOCKET"
fi 