#!/usr/bin/env bash
set -euo pipefail

echo "--- Setting up Podman for GitHub Actions ---"

# Install podman if not available
if ! command -v podman >/dev/null 2>&1; then
    echo "Installing podman..."
    sudo apt-get update
    sudo apt-get install -y podman podman-compose
fi

# Configure for GitHub Actions environment
echo "Configuring podman for GitHub Actions..."

# Set up subuid/subgid for rootless containers
sudo usermod --add-subuids 100000-165535 --add-subgids 100000-165535 $USER

# Configure containers in user space
mkdir -p ~/.config/containers
echo 'unqualified-search-registries = ["docker.io"]' > ~/.config/containers/registries.conf
echo '{"default":[{"type":"insecureAcceptAnything"}]}' > ~/.config/containers/policy.json

# For CI: also set up global configuration
sudo mkdir -p /etc/containers
echo '{ "default": [{"type": "insecureAcceptAnything"}]}' | sudo tee /etc/containers/policy.json
echo 'unqualified-search-registries = ["docker.io"]' | sudo tee /etc/containers/registries.conf

# Test podman
echo "Testing podman setup..."
podman version
podman info

echo "--- Podman setup for GitHub Actions complete ---" 