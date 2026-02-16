#!/usr/bin/env bash
set -euo pipefail

CORE_PID_FILE=".core.pid"
ADMIN_PANEL_PID_FILE=".admin-panel.pid"

echo "Stopping Cypress test stack..."

if [[ -f "$CORE_PID_FILE" ]]; then
    kill "$(cat "$CORE_PID_FILE")" 2>/dev/null || true
    rm -f "$CORE_PID_FILE"
fi

if [[ -f "$ADMIN_PANEL_PID_FILE" ]]; then
    kill "$(cat "$ADMIN_PANEL_PID_FILE")" 2>/dev/null || true
    rm -f "$ADMIN_PANEL_PID_FILE"
fi

pkill -f "lana-cli" || true
pkill -f "admin-panel.*pnpm.*start" || true

make stop-deps || true

echo "Cypress test stack stopped."
