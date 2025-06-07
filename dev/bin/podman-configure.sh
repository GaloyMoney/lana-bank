#!/usr/bin/env bash
set -euo pipefail

echo "--- Configuring Podman ---"

if [ "$(uname)" = "Linux" ]; then
    echo "Applying Linux-specific podman configuration..."
    
    # Try to configure system-wide containers first, fall back to user config
    if sudo mkdir -p /etc/containers 2>/dev/null; then
        echo '{ "default": [{"type": "insecureAcceptAnything"}]}' | sudo tee /etc/containers/policy.json >/dev/null || true
        echo 'unqualified-search-registries = ["docker.io"]' | sudo tee /etc/containers/registries.conf >/dev/null || true
        sudo grep -q "host.containers.internal" /etc/hosts 2>/dev/null || echo "127.0.0.1 host.containers.internal" | sudo tee -a /etc/hosts >/dev/null || true
    else
        echo "Cannot write to system directories, configuring user-space only..."
        # Configure containers in user space as fallback
        mkdir -p ~/.config/containers
        echo '{"default":[{"type":"insecureAcceptAnything"}]}' > ~/.config/containers/policy.json
        echo 'unqualified-search-registries = ["docker.io"]' > ~/.config/containers/registries.conf
    fi
else
    echo "Non-Linux system detected, skipping container configuration"
fi

echo "--- Podman configuration done ---" 