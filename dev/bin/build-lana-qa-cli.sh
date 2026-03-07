#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
OPEN_CLAW_AGENT="${OPEN_CLAW_AGENT:-workspace-lana-qa}"

SOURCE_SKILL_DIR="${REPO_ROOT}/.claude/skills/lana-qa"
SOURCE_SKILL_FILE="${SOURCE_SKILL_DIR}/SKILL.md"
SOURCE_WORKFLOWS_DIR="${SOURCE_SKILL_DIR}/workflows"
SOURCE_BIN="${REPO_ROOT}/target/release/lana-admin"
SOURCE_DEPS_SCRIPT="${REPO_ROOT}/dev/bin/workflow-step-deps.sh"

DEST_SKILL_DIR="${HOME}/.openclaw/${OPEN_CLAW_AGENT}/skills/lana-qa"
DEST_BIN="${DEST_SKILL_DIR}/lana-admin"
DEST_DEPS_SCRIPT="${DEST_SKILL_DIR}/workflow-step-deps.sh"
DEST_WORKFLOWS_DIR="${DEST_SKILL_DIR}/workflows"

[[ -f "${SOURCE_SKILL_FILE}" ]] || {
  echo "Missing source skill file: ${SOURCE_SKILL_FILE}" >&2
  exit 1
}

[[ -d "${SOURCE_WORKFLOWS_DIR}" ]] || {
  echo "Missing workflows directory: ${SOURCE_WORKFLOWS_DIR}" >&2
  exit 1
}

[[ -f "${SOURCE_DEPS_SCRIPT}" ]] || {
  echo "Missing dependency helper: ${SOURCE_DEPS_SCRIPT}" >&2
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
rm -rf "${DEST_WORKFLOWS_DIR}"

cp "${SOURCE_SKILL_FILE}" "${DEST_SKILL_DIR}/SKILL.md"
cp -R "${SOURCE_WORKFLOWS_DIR}" "${DEST_WORKFLOWS_DIR}"
cp "${SOURCE_BIN}" "${DEST_BIN}"
cp "${SOURCE_DEPS_SCRIPT}" "${DEST_DEPS_SCRIPT}"

chmod +x "${DEST_BIN}"
chmod +x "${DEST_DEPS_SCRIPT}"
rm -f "${DEST_SKILL_DIR}/lana-cli"

echo "Copied skill file to ${DEST_SKILL_DIR}/SKILL.md"
echo "Copied workflows to ${DEST_WORKFLOWS_DIR}"
echo "Copied binary to ${DEST_BIN}"
echo "Copied deps script to ${DEST_DEPS_SCRIPT}"
