#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SOURCE_SKILL_DIR="${REPO_ROOT}/.claude/skills/lana-qa"
SOURCE_BIN="${REPO_ROOT}/target/release/lana-admin-cli"
OPEN_CLAW_AGENT="${OPEN_CLAW_AGENT:-workspace-lana-qa}"
OPENCLAW_SKILLS_DIR="${HOME}/.openclaw/${OPEN_CLAW_AGENT}/skills"
DEST_SKILL_DIR="${OPENCLAW_SKILLS_DIR}/lana-qa"
DEST_BIN="${DEST_SKILL_DIR}/lana-admin-cli"

if [[ ! -d "${SOURCE_SKILL_DIR}" ]]; then
  mkdir -p "${SOURCE_SKILL_DIR}"
  echo "Created source skill directory: ${SOURCE_SKILL_DIR}"
fi

SOURCE_SKILL_FILE="${SOURCE_SKILL_DIR}/SKILL.md"

if [[ ! -f "${SOURCE_SKILL_FILE}" ]]; then
  echo "Missing source skill file: ${SOURCE_SKILL_FILE}" >&2
  exit 1
fi

mkdir -p "${DEST_SKILL_DIR}"

echo "Building lana-admin-cli (release)..."
(
  cd "${REPO_ROOT}"
  SQLX_OFFLINE="${SQLX_OFFLINE:-true}" cargo build --release -p lana-admin-cli
)

if [[ ! -f "${SOURCE_BIN}" ]]; then
  echo "Build finished but binary not found: ${SOURCE_BIN}" >&2
  exit 1
fi

cp "${SOURCE_SKILL_FILE}" "${DEST_SKILL_DIR}/SKILL.md"
cp "${SOURCE_BIN}" "${DEST_BIN}"
chmod +x "${DEST_BIN}"
rm -f "${DEST_SKILL_DIR}/lana-cli"

echo "Copied skill file to ${DEST_SKILL_DIR}/SKILL.md"
echo "Copied binary to ${DEST_BIN}"
