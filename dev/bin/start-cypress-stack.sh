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
    make clean-deps-podman || true
}

# Set up trap for cleanup only on interruption, not normal exit
trap cleanup INT TERM

echo "Starting Cypress test stack..."

# Ensure proper podman setup for CI environment
echo "Setting up podman environment..."
export ENGINE_DEFAULT=podman

# Setup podman if not already configured
if [ "$(uname)" = "Linux" ] && [ "${CI:-}${CI_MODE:-}" = "true" ]; then
    echo "CI environment detected, setting up podman..."
    make podman-setup
else
    echo "Development environment, skipping podman setup"
fi

# Step 1: Start dependencies (databases, auth services, etc.)
echo "Starting dependencies with podman..."
make start-deps-podman

# Add diagnostic info after starting dependencies
echo "Checking dependency startup status..."
sleep 5
podman ps --filter "label=com.docker.compose.project=lana-bank" --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" || true

# Step 2: Setup database
echo "Setting up database..."
make setup-db

# Step 3: Start core backend server in background
echo "Starting core server..."

# Check if build was successful first
echo "Building server binary..."
if ! nix build . -L; then
    echo "ERROR: Failed to build server binary"
    echo "Build logs:"
    nix log . || true
    exit 1
fi

echo "Build successful, starting server..."

# Start server in background and capture PID using nix with better error handling
echo "Running: nix run . -- --config ./bats/lana-sim-time.yml"
if ! nohup nix run . -- --config ./bats/lana-sim-time.yml > "$LOG_FILE" 2>&1 & then
    echo "ERROR: Failed to start server"
    cat "$LOG_FILE" || true
    exit 1
fi

SERVER_PID=$!
echo $SERVER_PID > "$CORE_PID_FILE"
echo "Server started with PID: $SERVER_PID"

# Give the server a moment to start and check if it's still running
sleep 2
if ! kill -0 "$SERVER_PID" 2>/dev/null; then
    echo "ERROR: Server process died immediately after startup"
    echo "Server logs:"
    cat "$LOG_FILE" || true
    exit 1
fi

# Step 4: Wait for core server to be ready
echo "Waiting for core server to be ready..."
echo "Server logs during startup:"
tail -f "$LOG_FILE" &
TAIL_PID=$!

if ! wait4x http http://localhost:5253/health --timeout 60s; then
    echo "ERROR: Server failed to become healthy within 60 seconds"
    echo "Final server logs:"
    cat "$LOG_FILE" || true
    echo "Server process status:"
    if kill -0 "$SERVER_PID" 2>/dev/null; then
        echo "Server process is still running (PID: $SERVER_PID)"
    else
        echo "Server process has died"
    fi
    kill "$TAIL_PID" 2>/dev/null || true
    exit 1
fi

kill "$TAIL_PID" 2>/dev/null || true
echo "Core server is ready!"

# Step 5: Start admin panel in background
echo "Starting admin panel..."
export NEXT_PUBLIC_BASE_PATH="/admin"
export NEXT_PUBLIC_CORE_ADMIN_URL="/admin/graphql"

cd apps/admin-panel
echo "Installing admin panel dependencies..."
if ! nix develop -c pnpm install --frozen-lockfile; then
    echo "ERROR: Failed to install admin panel dependencies"
    cd ../..
    exit 1
fi

echo "Starting admin panel dev server..."
nohup nix develop -c pnpm dev > "../../admin-panel.log" 2>&1 &
ADMIN_PANEL_PID=$!
echo $ADMIN_PANEL_PID > "../../$ADMIN_PANEL_PID_FILE"
echo "Admin panel started with PID: $ADMIN_PANEL_PID"
cd ../..

# Give the admin panel a moment to start
sleep 3
if ! kill -0 "$ADMIN_PANEL_PID" 2>/dev/null; then
    echo "ERROR: Admin panel process died immediately after startup"
    echo "Admin panel logs:"
    cat "admin-panel.log" || true
    exit 1
fi

# Step 6: Wait for admin panel to be ready
echo "Waiting for admin panel to be ready..."

# First check if the admin panel is running locally
echo "Checking if admin panel is running on port 3001..."
wait4x http http://localhost:3001/api/health --timeout 30s

# Then check if it's accessible through the proxy
echo "Checking admin panel through Oathkeeper proxy..."
wait4x http http://localhost:4455/admin/api/health --timeout 30s

echo "All services are ready!"
exit 0