#!/usr/bin/env bash
set -euo pipefail

echo "=== Lana Bank Dev Container Post-Start ==="

# Source Nix so we have cargo and other tools
if [[ -f "$HOME/.nix-profile/etc/profile.d/nix.sh" ]]; then
    . "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi

# Allow direnv (loads the nix flake environment)
cd /workspaces/lana-bank
direnv allow 2>/dev/null || true
eval "$(direnv export bash 2>/dev/null)" || true

# Wait for PostgreSQL to be ready
echo "Waiting for PostgreSQL..."
for i in $(seq 1 30); do
    if pg_isready -h core-pg -p 5432 -U user -d pg >/dev/null 2>&1; then
        echo "PostgreSQL is ready."
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo "Warning: PostgreSQL not ready after 30s, skipping migrations."
        exit 0
    fi
    sleep 1
done

# Run database migrations
echo "Running database migrations..."
cd /workspaces/lana-bank/lana/app && cargo sqlx migrate run
echo "Migrations complete."
