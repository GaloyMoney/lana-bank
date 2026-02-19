#!/usr/bin/env bash
set -euo pipefail

# devenv-cleanup.sh — Release the devenv slot for this worktree
#
# Removes this worktree's entry from the slot registry and cleans up
# all generated files (.env, docker-compose.override.yml, .devenv/).

REPO_ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
REGISTRY_DIR="${HOME}/.local/share/lana-bank"
REGISTRY_FILE="${REGISTRY_DIR}/devenv-slots"

if [[ ! -f "${REGISTRY_FILE}" ]]; then
  echo "No devenv slot registry found — nothing to clean up."
  exit 0
fi

# Remove this worktree's entry from the registry
CLEANED=""
RELEASED_SLOT=""
while IFS=: read -r s path; do
  if [[ "${path}" == "${REPO_ROOT}" ]]; then
    RELEASED_SLOT="${s}"
  else
    CLEANED="${CLEANED}${s}:${path}"$'\n'
  fi
done < "${REGISTRY_FILE}"
printf '%s' "${CLEANED}" > "${REGISTRY_FILE}"

# Remove generated files
rm -f "${REPO_ROOT}/.env"
rm -f "${REPO_ROOT}/docker-compose.override.yml"
rm -rf "${REPO_ROOT}/.devenv"

if [[ -n "${RELEASED_SLOT}" ]]; then
  echo "Released devenv slot ${RELEASED_SLOT} for ${REPO_ROOT}"
else
  echo "No devenv slot found for ${REPO_ROOT} — nothing to release."
fi
