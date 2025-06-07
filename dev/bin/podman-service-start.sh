#!/usr/bin/env bash
set -euo pipefail

echo "--- Starting Podman service ---"

if [ "$(uname)" = "Linux" ]; then
    echo "Checking if podman socket is working..."
    if [ -S /run/podman/podman.sock ] && CONTAINER_HOST=unix:///run/podman/podman.sock timeout 3s podman version >/dev/null 2>&1; then
        echo "Podman socket already working!"
    else
        echo "Starting podman system service..."
        mkdir -p /run/podman
        podman system service --time=0 unix:///run/podman/podman.sock &
        echo "Waiting for socket to be created..."
        for i in 1 2 3 4 5; do
            if [ -S /run/podman/podman.sock ] && CONTAINER_HOST=unix:///run/podman/podman.sock timeout 3s podman version >/dev/null 2>&1; then
                echo "Socket created and working!"
                break
            fi
            echo "Waiting... ($i/5)"
            sleep 2
        done
    fi
fi

echo "--- Podman service ready ---" 