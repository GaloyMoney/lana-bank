#!/usr/bin/env bash
set -euo pipefail

# Start Cypress Test Stack
# This script replaces tilt-in-ci by starting all required services for cypress tests

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

# Set up trap for cleanup
trap cleanup EXIT INT TERM

# Check if required commands are available
command -v podman >/dev/null 2>&1 || { echo "Error: podman not found"; exit 1; }
command -v nix >/dev/null 2>&1 || { echo "Error: nix not found"; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo "Error: pnpm not found"; exit 1; }

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

# Add database connectivity check
echo "Checking database connectivity..."
for i in {1..30}; do
    if podman exec lana-bank-core-pg-1 pg_isready -U user >/dev/null 2>&1; then
        echo "Core database is ready!"
        break
    fi
    if [[ $i -eq 30 ]]; then
        echo "Core database failed to start within 30 seconds"
        echo "Container logs:"
        podman logs lana-bank-core-pg-1 --tail 50 || true
        exit 1
    fi
    echo "Waiting for core database... ($i/30)"
    sleep 1
done

# Step 3: Start core backend server in background
echo "Starting core server..."
export PG_CON="postgres://user:password@localhost:5433/pg"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export BFX_LOCAL_PRICE="${BFX_LOCAL_PRICE:-1}"

# Start server in background and capture PID using nix
nix build .
nohup nix run . -- --config ./bats/lana-sim-time.yml > "$LOG_FILE" 2>&1 &
echo $! > "$CORE_PID_FILE"

# Step 4: Wait for core server to be ready
echo "Waiting for core server to be ready..."
for i in {1..300}; do
    # Try both the GraphQL endpoint and health endpoint
    if curl -s -f "http://localhost:5253/health" >/dev/null 2>&1 || \
       curl -s -f "http://localhost:5253/graphql" >/dev/null 2>&1; then
        echo "Core server is ready!"
        break
    fi
    if [[ $i -eq 300 ]]; then
        echo "Core server failed to start within 300 seconds"
        echo "Server logs:"
        cat "$LOG_FILE"
        exit 1
    fi
    echo "Waiting for core server... ($i/300)"
    sleep 1
done

# Step 5: Start admin panel in background
echo "Starting admin panel..."
export NEXT_PUBLIC_BASE_PATH="/admin"
export NEXT_PUBLIC_CORE_ADMIN_URL="/admin/graphql"

cd apps/admin-panel
nohup nix develop -c bash -c "pnpm install --frozen-lockfile && pnpm dev" > "../../admin-panel.log" 2>&1 &
echo $! > "../../$ADMIN_PANEL_PID_FILE"
cd ../..

# Step 6: Wait for admin panel to be ready
echo "Waiting for admin panel to be ready..."
for i in {1..180}; do
    # Try both the admin page and health endpoint through oathkeeper proxy
    if curl -s -f "http://localhost:4455/admin/api/health" >/dev/null 2>&1 || \
       curl -s -f "http://localhost:4455/admin" >/dev/null 2>&1; then
        echo "Admin panel is ready!"
        break
    fi
    if [[ $i -eq 180 ]]; then
        echo "Admin panel failed to start within 180 seconds"
        echo "Admin panel logs:"
        cat admin-panel.log
        exit 1
    fi
    echo "Waiting for admin panel... ($i/180)"
    sleep 1
done

echo "All services are ready!"

# Final health validation
echo "Performing final health checks..."
if ! curl -s -f "http://localhost:5253/health" >/dev/null 2>&1; then
    echo "WARNING: Core server health check failed"
fi

if ! curl -s -f "http://localhost:4455/admin/api/health" >/dev/null 2>&1; then
    echo "WARNING: Admin panel health check failed"
fi

echo "✅ Services URLs:"
echo "  Core server: http://localhost:5253/graphql"
echo "  Admin panel: http://localhost:4455/admin"
echo "📋 Logs:"
echo "  Core server: $LOG_FILE"
echo "  Admin panel: admin-panel.log"

# Services started successfully, exiting
echo "Services started successfully!"
exit 0