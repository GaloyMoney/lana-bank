#!/usr/bin/env bash
set -euo pipefail

echo "--- Configuring Podman ---"

if [ "$(uname)" = "Linux" ]; then
    echo "Applying Linux-specific podman configuration..."
    mkdir -p /etc/containers
    echo '{ "default": [{"type": "insecureAcceptAnything"}]}' > /etc/containers/policy.json || true
    echo 'unqualified-search-registries = ["docker.io"]' > /etc/containers/registries.conf || true
    grep -q "host.containers.internal" /etc/hosts || echo "127.0.0.1 host.containers.internal" >> /etc/hosts || true
else
    echo "Non-Linux system detected, skipping container configuration"
fi

echo "--- Podman configuration done ---" 