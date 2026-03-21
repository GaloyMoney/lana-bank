#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OPEN_CLAW_AGENT="${OPEN_CLAW_AGENT:-workspace-lana-qa}"

SOURCE_SKILL_DIR="${REPO_ROOT}/.claude/skills/lana-qa"
SOURCE_SKILL_FILE="${SOURCE_SKILL_DIR}/SKILL.md"
SOURCE_BIN="${REPO_ROOT}/target/release/lana-admin"

SOURCE_CHAOS_DIR="${REPO_ROOT}/.claude/skills/lana-chaos"
SOURCE_CHAOS_FILE="${SOURCE_CHAOS_DIR}/SKILL.md"

DEST_SKILL_DIR="${HOME}/.openclaw/${OPEN_CLAW_AGENT}/skills/lana-qa"
DEST_BIN="${DEST_SKILL_DIR}/lana-admin"
DEST_CHAOS_DIR="${HOME}/.openclaw/${OPEN_CLAW_AGENT}/skills/lana-chaos"

[[ -f "${SOURCE_SKILL_FILE}" ]] || {
  echo "Missing source skill file: ${SOURCE_SKILL_FILE}" >&2
  exit 1
}

echo "Building lana-admin (release)..."
(
  cd "${REPO_ROOT}"
  SQLX_OFFLINE="${SQLX_OFFLINE:-true}" cargo build --release -p lana-admin
)

[[ -f "${SOURCE_BIN}" ]] || {
  echo "Build finished but binary not found: ${SOURCE_BIN}" >&2
  exit 1
}

mkdir -p "${DEST_SKILL_DIR}"

cp "${SOURCE_SKILL_FILE}" "${DEST_SKILL_DIR}/SKILL.md"
cp "${SOURCE_BIN}" "${DEST_BIN}"

chmod +x "${DEST_BIN}"
rm -rf "${DEST_SKILL_DIR}/workflows"
rm -f "${DEST_SKILL_DIR}/workflow-step-deps.sh"
rm -f "${DEST_SKILL_DIR}/lana-cli"

echo "Copied skill file to ${DEST_SKILL_DIR}/SKILL.md"
echo "Copied binary to ${DEST_BIN}"

if [[ -f "${SOURCE_CHAOS_FILE}" ]]; then
  mkdir -p "${DEST_CHAOS_DIR}"
  cp "${SOURCE_CHAOS_FILE}" "${DEST_CHAOS_DIR}/SKILL.md"
  echo "Copied chaos skill to ${DEST_CHAOS_DIR}/SKILL.md"
else
  echo "Skipping lana-chaos skill (not found at ${SOURCE_CHAOS_FILE})"
fi
