#!/usr/bin/env bash
set -euo pipefail

# Start Cypress Test Stack
LOG_FILE="cypress-stack.log"
CORE_PID_FILE=".core.pid"
ADMIN_PANEL_PID_FILE=".admin-panel.pid"

# Cleanup function
cleanup() {
    echo "Cleaning up..."
    
    # Kill background processes
    if [[ -f "$CORE_PID_FILE" ]]; then
        CORE_PID=$(cat "$CORE_PID_FILE")
        kill "$CORE_PID" 2>/dev/null || true
        rm -f "$CORE_PID_FILE"
    fi
    
    if [[ -f "$ADMIN_PANEL_PID_FILE" ]]; then
        ADMIN_PANEL_PID=$(cat "$ADMIN_PANEL_PID_FILE")
        kill "$ADMIN_PANEL_PID" 2>/dev/null || true
        rm -f "$ADMIN_PANEL_PID_FILE"
    fi
    
    # Kill any remaining processes
    pkill -f "lana-cli" || true
    pkill -f "admin-panel.*pnpm.*dev" || true
    
    # Stop podman services
    make clean-deps || true
}

# Set up trap for cleanup only on interruption, not normal exit
trap cleanup INT TERM

echo "Starting Cypress test stack..."

# Start dependencies
echo "Starting dependencies..."
make start-deps

# Diagnostic info
echo "Checking dependency status..."
sleep 5
${ENGINE_DEFAULT:-docker} ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || true

# Setup database
echo "Setting up database..."
make setup-db

# Start core server
echo "Starting core server..."
export LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
nix build .
nohup nix run . -- --config ./bats/lana.yml > "$LOG_FILE" 2>&1 &
echo $! > "$CORE_PID_FILE"

# Wait for core server
wait4x http http://localhost:5253/health --timeout 60s

# Start admin panel
echo "Starting admin panel..."
export NEXT_PUBLIC_CORE_ADMIN_URL="/graphql"

cd apps/admin-panel
echo "Building admin panel..."
nix develop -c bash -c "pnpm install --frozen-lockfile && pnpm build" || { echo "Admin panel build failed"; exit 1; }

echo "Starting admin panel server..."
nohup nix develop -c pnpm start --port 3001 > "../../admin-panel.log" 2>&1 &
echo $! > "../../$ADMIN_PANEL_PID_FILE"
cd ../..

# Wait for admin panel services
wait4x http http://localhost:3001/api/health --timeout 200s || { echo "Admin panel failed - check admin-panel.log"; exit 1; }
wait4x http http://admin.localhost:4455/api/health --timeout 30s || { echo "Proxy access failed"; exit 1; }

echo "All services are ready!"
exit 0