#!/usr/bin/env bash
set -euo pipefail

echo "--- Starting Lana server for BATS tests ---"

# Get repo root and source helpers
export REPO_ROOT=$(git rev-parse --show-toplevel)
if [[ -f "${REPO_ROOT}/bats/helpers.bash" ]]; then
    source "${REPO_ROOT}/bats/helpers.bash"
else
    echo "Error: helpers.bash not found at ${REPO_ROOT}/bats/helpers.bash"
    exit 1
fi

# Start server with cleanup trap
start_server
trap 'echo "--- Stopping Lana server ---"; stop_server' EXIT

echo "--- Running BATS tests ---"
bats -t bats 