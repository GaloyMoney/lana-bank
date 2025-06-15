#!/usr/bin/env bash
set -euo pipefail

echo "--- Configuring Podman ---"

if [ "$(uname)" = "Linux" ]; then
    echo "Applying Linux-specific podman configuration..."
    
    # Only try to configure if we have write permissions or if running as root
    if [ -w /etc/containers ] || [ "$(id -u)" -eq 0 ]; then
        mkdir -p /etc/containers
        echo '{ "default": [{"type": "insecureAcceptAnything"}]}' > /etc/containers/policy.json || echo "Warning: Could not write policy.json"
        echo 'unqualified-search-registries = ["docker.io"]' > /etc/containers/registries.conf || echo "Warning: Could not write registries.conf"
    else
        echo "Skipping system-level container configuration (no write permissions)"
    fi
    
    # Only try to modify /etc/hosts if we have write permissions
    if [ -w /etc/hosts ] || [ "$(id -u)" -eq 0 ]; then
        grep -q "host.containers.internal" /etc/hosts || echo "127.0.0.1 host.containers.internal" >> /etc/hosts || echo "Warning: Could not modify /etc/hosts"
    else
        echo "Skipping /etc/hosts modification (no write permissions)"
    fi
else
    echo "Non-Linux system detected, skipping container configuration"
fi

echo "--- Podman configuration done ---" 