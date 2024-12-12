#!/bin/bash
set -eo pipefail  # Exit on error, pipe failure

REPO_ROOT=$(git rev-parse --show-toplevel)
LOGS_DIR="${REPO_ROOT}/logs"
mkdir -p "${LOGS_DIR}"

# Source CI environment if exists
[ -f tmp.env.ci ] && source tmp.env.ci || true

cd "${REPO_ROOT}"

# Run Tilt with full logging
echo "Starting Tilt CI at $(date)" | tee -a "${LOGS_DIR}/tilt-full.log"
tilt ci --file dev/Tiltfile 2>&1 | tee -a "${LOGS_DIR}/tilt-full.log" | tee >(grep cypress > "${LOGS_DIR}/cypress.log") | grep cypress

status=${PIPESTATUS[0]}

# Collect additional information on failure
if [[ $status -ne 0 ]]; then
    echo "Tilt CI failed with status $status at $(date)" | tee -a "${LOGS_DIR}/tilt-full.log"
    echo "=== Tilt Logs ===" | tee -a "${LOGS_DIR}/failure.log"
    tail -n 100 "${LOGS_DIR}/tilt-full.log" >> "${LOGS_DIR}/failure.log"
    
    echo "=== Container Status ===" | tee -a "${LOGS_DIR}/failure.log"
    docker ps -a >> "${LOGS_DIR}/failure.log"
    
    echo "=== Failed Container Logs ===" | tee -a "${LOGS_DIR}/failure.log"
    docker ps -a --filter "status=exited" --format '{{.Names}}' | while read container; do
        echo "=== $container ===" >> "${LOGS_DIR}/failure.log"
        docker logs $container &>> "${LOGS_DIR}/failure.log"
    done
    
    cat "${LOGS_DIR}/failure.log"
else
    echo "Tilt CI passed at $(date)" | tee -a "${LOGS_DIR}/tilt-full.log"
fi

exit "$status"