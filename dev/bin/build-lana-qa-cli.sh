#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/../.." && pwd)"
SOURCE_SKILL_DIR="${REPO_ROOT}/.claude/skills/lana-qa"
SOURCE_BIN="${REPO_ROOT}/target/release/lana-admin"
SOURCE_DEPS_SCRIPT="${REPO_ROOT}/dev/bin/workflow-step-deps.sh"
SOURCE_WORKFLOWS_DIR="${SOURCE_SKILL_DIR}/workflows"
OPEN_CLAW_AGENT="${OPEN_CLAW_AGENT:-workspace-lana-qa}"
OPENCLAW_SKILLS_DIR="${HOME}/.openclaw/${OPEN_CLAW_AGENT}/skills"
DEST_SKILL_DIR="${OPENCLAW_SKILLS_DIR}/lana-qa"
DEST_BIN="${DEST_SKILL_DIR}/lana-admin"
DEST_DEPS_SCRIPT="${DEST_SKILL_DIR}/workflow-step-deps.sh"
DEST_WORKFLOWS_DIR="${DEST_SKILL_DIR}/workflows"

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

echo "Building lana-admin (release)..."
(
  cd "${REPO_ROOT}"
  SQLX_OFFLINE="${SQLX_OFFLINE:-true}" cargo build --release -p lana-admin
)

if [[ ! -f "${SOURCE_BIN}" ]]; then
  echo "Build finished but binary not found: ${SOURCE_BIN}" >&2
  exit 1
fi

if [[ ! -f "${SOURCE_DEPS_SCRIPT}" ]]; then
  echo "Missing dependency script: ${SOURCE_DEPS_SCRIPT}" >&2
  exit 1
fi

cp "${SOURCE_SKILL_FILE}" "${DEST_SKILL_DIR}/SKILL.md"
if [[ -d "${SOURCE_WORKFLOWS_DIR}" ]]; then
  rm -rf "${DEST_WORKFLOWS_DIR}"
  cp -R "${SOURCE_WORKFLOWS_DIR}" "${DEST_WORKFLOWS_DIR}"
fi
cp "${SOURCE_BIN}" "${DEST_BIN}"
cp "${SOURCE_DEPS_SCRIPT}" "${DEST_DEPS_SCRIPT}"
chmod +x "${DEST_BIN}"
chmod +x "${DEST_DEPS_SCRIPT}"
rm -f "${DEST_SKILL_DIR}/lana-cli"

echo "Copied skill file to ${DEST_SKILL_DIR}/SKILL.md"
if [[ -d "${DEST_WORKFLOWS_DIR}" ]]; then
  echo "Copied workflows to ${DEST_WORKFLOWS_DIR}"
fi
echo "Copied binary to ${DEST_BIN}"
echo "Copied deps script to ${DEST_DEPS_SCRIPT}"
