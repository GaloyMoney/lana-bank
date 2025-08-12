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

# Step 1: Start dependencies (databases, auth services, etc.)
echo "Starting dependencies..."
make start-deps

# Add diagnostic info after starting dependencies
echo "Checking dependency startup status..."
sleep 5
${ENGINE_DEFAULT:-docker} ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || true

# Step 2: Setup database
echo "Setting up database..."
make setup-db

# Step 3: Start core backend server in background
echo "Starting core server..."

# Start server in background and capture PID using nix
nix build .
nohup nix run . -- --config ./bats/lana.yml > "$LOG_FILE" 2>&1 &
echo $! > "$CORE_PID_FILE"

# Step 4: Wait for core server to be ready
echo "Waiting for core server to be ready..."
wait4x http http://localhost:5253/health --timeout 60s

# Step 5: Start admin panel in background
echo "Starting admin panel..."
export NEXT_PUBLIC_CORE_ADMIN_URL="/graphql"

cd apps/admin-panel

# Build first (synchronously to catch build errors)
echo "Installing dependencies and building admin panel..."
if ! nix develop -c bash -c "pnpm install --frozen-lockfile && pnpm build"; then
    echo "ERROR: Admin panel build failed"
    exit 1
fi

# Then start server in background
echo "Starting admin panel server..."
nohup nix develop -c pnpm start --port 3001 > "../../admin-panel.log" 2>&1 &
echo $! > "../../$ADMIN_PANEL_PID_FILE"
cd ../..

# Step 6: Wait for admin panel to be ready
echo "Waiting for admin panel to build and start on port 3001..."

# First wait for admin panel to be ready on port 3001 directly
if ! wait4x http http://localhost:3001/api/health --timeout 200s; then
    echo "ERROR: Admin panel not starting on port 3001"
    echo "Check admin-panel.log for build/start errors"
    exit 1
fi

echo "Admin panel is ready, now checking proxy access..."

# Then check proxy access
if ! wait4x http http://admin.localhost:4455/api/health --timeout 30s; then
    echo "ERROR: Admin panel proxy not accessible"
    echo "Oathkeeper may have networking issues"
    exit 1
fi

echo "All services are ready!"
exit 0