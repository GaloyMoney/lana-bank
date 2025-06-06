#!/usr/bin/env bash
set -euo pipefail

echo "--- Checking for Podman (via nix) ---"
command -v podman
echo "--- Podman check done ---"
command -v podman-compose
echo "--- Podman-compose check done ---"

echo "--- Testing Podman basic functionality ---"
podman info || echo "Warning: 'podman info' failed."
echo "--- Podman info done ---"

echo "--- Starting Podman service ---"
# Ensure DOCKER_HOST points to the standard rootful socket location
export DOCKER_HOST=unix:///run/podman/podman.sock 
podman system service --time=0 & # Start service in background
sleep 5 # Wait a bit for the socket to become active
echo "--- Podman service started (attempted) ---"

# @@
# -echo "--- Starting Podman service ---"
# -# Ensure DOCKER_HOST points to the standard rootful socket location
# -export DOCKER_HOST=unix:///run/podman/podman.sock 
# -podman system service --time=0 &     # Start service in background
# -sleep 5                              # Wait a bit for the socket to become active
# -echo "--- Podman service started (attempted) ---"
# +echo "--- Enabling *rootless* Podman socket ---"
# +# one-shot (CI) – omit ‘enable’ if you don’t want it after reboot
# +systemctl --user enable --now podman.socket
# +# Tell docker-compose / podman-compose where that socket is
# +export DOCKER_HOST="unix://${XDG_RUNTIME_DIR}/podman/podman.sock"
# +echo "--- Podman user socket is ready ---"


mkdir -p /etc/containers
echo '{ "default": [{"type": "insecureAcceptAnything"}]}' > /etc/containers/policy.json
echo 'unqualified-search-registries = ["docker.io"]' > /etc/containers/registries.conf

echo "--- Starting Dependencies with Podman Compose ---"
ENGINE_DEFAULT=podman bin/docker-compose-up.sh
echo "--- Podman-compose up done ---"

make setup-db

echo "--- Running Integration Tests ---"
cargo nextest run --verbose --locked
echo "--- Tests done ---"

echo "--- Cleaning up dependencies ---"
ENGINE_DEFAULT=podman bin/clean-deps.sh
echo "--- Cleanup done ---"

echo "--- All steps completed ---" 