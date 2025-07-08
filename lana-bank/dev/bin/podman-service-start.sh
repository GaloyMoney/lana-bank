#!/usr/bin/env bash
set -euo pipefail

echo "--- Configuring Podman ---"

# Ensure we're working with a clean environment
unset DOCKER_HOST CONTAINER_HOST

configure_podman_linux() {
    echo "Applying Linux-specific podman configuration..."
    
    # Create system containers directory if possible, fall back to user config
    if sudo mkdir -p /etc/containers 2>/dev/null; then
        echo '{"default":[{"type":"insecureAcceptAnything"}]}' | sudo tee /etc/containers/policy.json >/dev/null
        echo 'unqualified-search-registries = ["docker.io"]' | sudo tee /etc/containers/registries.conf >/dev/null
    else
        echo "Cannot write to /etc/containers, using user configuration..."
        mkdir -p ~/.config/containers
        echo '{"default":[{"type":"insecureAcceptAnything"}]}' > ~/.config/containers/policy.json
        echo 'unqualified-search-registries = ["docker.io"]' > ~/.config/containers/registries.conf
    fi
    
    # Add host.containers.internal entry
    if ! grep -q "host.containers.internal" /etc/hosts 2>/dev/null; then
        echo "127.0.0.1 host.containers.internal" | sudo tee -a /etc/hosts >/dev/null || {
            echo "Warning: Could not add host.containers.internal to /etc/hosts"
        }
    fi
    
    # Set up subuid/subgid for rootless containers if not already configured
    if ! grep -q "^$(whoami):" /etc/subuid 2>/dev/null; then
        echo "Setting up subuid/subgid for rootless containers..."
        sudo usermod --add-subuids 100000-165535 --add-subgids 100000-165535 "$(whoami)" || {
            echo "Warning: Could not configure subuid/subgid"
        }
    fi
}

start_podman_service() {
    echo "--- Starting Podman service ---"
    
    # Define socket paths
    local system_socket="/run/podman/podman.sock"
    local user_socket="${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/podman/podman.sock"
    
    # Function to test if a socket works
    test_socket() {
        local socket_path="$1"
        [ -S "$socket_path" ] && CONTAINER_HOST="unix://$socket_path" timeout 3s podman version >/dev/null 2>&1
    }
    
    # Check if any socket is already working
    if test_socket "$system_socket"; then
        echo "System podman socket already working!"
        export CONTAINER_HOST="unix://$system_socket"
        return 0
    elif test_socket "$user_socket"; then
        echo "User podman socket already working!"
        export CONTAINER_HOST="unix://$user_socket"
        return 0
    fi
    
    echo "Starting podman system service..."
    
    # Try to start system service, fall back to user service
    if sudo mkdir -p /run/podman 2>/dev/null; then
        echo "Using system socket at $system_socket"
        podman system service --time=0 "unix://$system_socket" &
        local socket_path="$system_socket"
    else
        echo "Cannot create system socket directory, using user socket at $user_socket"
        mkdir -p "$(dirname "$user_socket")"
        podman system service --time=0 "unix://$user_socket" &
        local socket_path="$user_socket"
    fi
    
    # Wait for socket to be ready
    echo "Waiting for socket to be created..."
    local max_attempts=10
    for i in $(seq 1 $max_attempts); do
        if test_socket "$socket_path"; then
            echo "Socket created and working!"
            export CONTAINER_HOST="unix://$socket_path"
            return 0
        fi
        echo "Waiting... ($i/$max_attempts)"
        sleep 2
    done
    
    echo "ERROR: Failed to start podman service after $max_attempts attempts"
    return 1
}

# Main execution
if [ "$(uname)" = "Linux" ]; then
    configure_podman_linux
    start_podman_service
else
    echo "Non-Linux system detected, skipping podman configuration"
    echo "Assuming podman is already configured properly"
fi

echo "--- Podman service ready ---"

# Verify final state
if command -v podman >/dev/null 2>&1; then
    echo "Podman version: $(podman version --format '{{.Client.Version}}')"
    if [ -n "${CONTAINER_HOST:-}" ]; then
        echo "Using socket: $CONTAINER_HOST"
    fi
else
    echo "ERROR: podman command not found in PATH"
    exit 1
fi