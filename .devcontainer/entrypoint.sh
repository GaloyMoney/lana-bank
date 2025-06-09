#!/bin/bash
# Entrypoint script for dev container with automatic Nix environment

set -e

# Source nix environment
if [ -f ~/.nix-profile/etc/profile.d/nix.sh ]; then
    source ~/.nix-profile/etc/profile.d/nix.sh
fi

# Change to workspace directory
cd /workspaces/lana-bank 2>/dev/null || true

# Load nix development environment if available
if [ -f flake.nix ] && [ -z "$NIX_SHELL" ]; then
    echo "🔧 Loading Nix development environment..."
    eval "$(nix print-dev-env 2>/dev/null || echo 'echo "Failed to load nix environment"')"
fi

# If no arguments provided, start an interactive shell
if [ $# -eq 0 ]; then
    echo "✅ Development environment ready!"
    echo "Available tools:"
    echo "  - Rust/Cargo: $(cargo --version 2>/dev/null || echo 'Not available')"
    echo "  - Node.js: $(node --version 2>/dev/null || echo 'Not available')"
    echo "  - PostgreSQL: $(psql --version 2>/dev/null || echo 'Not available')"
    echo ""
    exec bash
else
    # Execute the provided command in the environment
    exec "$@"
fi 