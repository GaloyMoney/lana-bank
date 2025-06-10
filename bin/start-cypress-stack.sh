#!/usr/bin/env bash
set -euo pipefail

# Start Cypress Test Stack
# This script replaces tilt-in-ci by starting all required services for cypress tests

LOG_FILE="cypress-stack.log"
CORE_PID_FILE=".core.pid"
ADMIN_PANEL_PID_FILE=".admin-panel.pid"
CI_MODE="${CI_MODE:-false}"

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
    
    # Stop docker services only if not in CI mode (CI will handle cleanup separately)
    if [[ "$CI_MODE" != "true" ]]; then
        make clean-deps || true
    fi
}

# Set up trap for cleanup only if not in CI mode
if [[ "$CI_MODE" != "true" ]]; then
    trap cleanup EXIT INT TERM
fi

# Check if required commands are available
command -v docker >/dev/null 2>&1 || { echo "Error: docker not found"; exit 1; }
command -v cargo >/dev/null 2>&1 || { echo "Error: cargo not found"; exit 1; }
command -v pnpm >/dev/null 2>&1 || { echo "Error: pnpm not found"; exit 1; }

echo "Starting Cypress test stack..."

# Step 1: Start dependencies (databases, auth services, etc.)
echo "Starting dependencies..."
make start-deps

# Step 2: Setup database
echo "Setting up database..."
make setup-db

# Step 3: Start core backend server in background
echo "Starting core server..."
export PG_CON="postgres://user:password@localhost:5433/pg"
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export BFX_LOCAL_PRICE="${BFX_LOCAL_PRICE:-1}"

# Start server in background and capture PID
nohup cargo run --bin lana-cli --features sim-time -- --config ./bats/lana-sim-time.yml > "$LOG_FILE" 2>&1 &
echo $! > "$CORE_PID_FILE"

# Step 4: Wait for core server to be ready
echo "Waiting for core server to be ready..."
for i in {1..60}; do
    # Try both the GraphQL endpoint and health endpoint
    if curl -s -f "http://localhost:5253/health" >/dev/null 2>&1 || \
       curl -s -f "http://localhost:5253/graphql" >/dev/null 2>&1; then
        echo "Core server is ready!"
        break
    fi
    if [[ $i -eq 60 ]]; then
        echo "Core server failed to start within 60 seconds"
        echo "Server logs:"
        cat "$LOG_FILE"
        exit 1
    fi
    echo "Waiting for core server... ($i/60)"
    sleep 1
done

# Step 5: Start admin panel in background
echo "Starting admin panel..."
export NEXT_PUBLIC_BASE_PATH="/admin"
export NEXT_PUBLIC_CORE_ADMIN_URL="/admin/graphql"

cd apps/admin-panel
nohup pnpm install --frozen-lockfile && pnpm dev > "../../admin-panel.log" 2>&1 &
echo $! > "../../$ADMIN_PANEL_PID_FILE"
cd ../..

# Step 6: Wait for admin panel to be ready
echo "Waiting for admin panel to be ready..."
for i in {1..120}; do
    # Try both the admin page and health endpoint through oathkeeper proxy
    if curl -s -f "http://localhost:4455/admin/api/health" >/dev/null 2>&1 || \
       curl -s -f "http://localhost:4455/admin" >/dev/null 2>&1; then
        echo "Admin panel is ready!"
        break
    fi
    if [[ $i -eq 120 ]]; then
        echo "Admin panel failed to start within 120 seconds"
        echo "Admin panel logs:"
        cat admin-panel.log
        exit 1
    fi
    echo "Waiting for admin panel... ($i/120)"
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

# In CI mode, just exit after starting services
# In dev mode, keep the script running until interrupted
if [[ "$CI_MODE" == "true" ]]; then
    echo "CI mode: Services started, script exiting"
    exit 0
else
    echo "Dev mode: Keeping services running. Press Ctrl+C to stop."
    wait
fi 