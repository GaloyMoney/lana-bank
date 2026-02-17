#!/bin/bash
set -euo pipefail

# Runs Cypress tests in both locales (ES and EN) and copies screenshots
# to the docs-site static directory for local builds.
#
# Prerequisites: the full stack must be running (make dev-up or make start-deps)
#
# Usage:
#   cd docs-site && bash scripts/generate-screenshots-local.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCS_DIR="$(dirname "$SCRIPT_DIR")"
REPO_ROOT="$(dirname "$DOCS_DIR")"
ADMIN_PANEL="$REPO_ROOT/apps/admin-panel"
CYPRESS_SCREENSHOTS="$ADMIN_PANEL/cypress/manuals/screenshots"
OUTPUT_DIR="$DOCS_DIR/static/img/screenshots/current"

echo "=== Generating screenshots for docs ==="
echo "Output: $OUTPUT_DIR"

# Run ES locale
echo ""
echo "--- Running Cypress (ES) ---"
cd "$ADMIN_PANEL"
rm -rf "$CYPRESS_SCREENSHOTS"
pnpm cypress:run-headless 2>&1 || { echo "ERROR: Cypress ES run failed"; exit 1; }

mkdir -p "$OUTPUT_DIR/es"
cp -r "$CYPRESS_SCREENSHOTS"/* "$OUTPUT_DIR/es/"
echo "ES screenshots copied to $OUTPUT_DIR/es/"

# Save ES screenshots before EN run overwrites them
ES_BACKUP=$(mktemp -d)
cp -r "$CYPRESS_SCREENSHOTS"/* "$ES_BACKUP/"

# Run EN locale
echo ""
echo "--- Running Cypress (EN) ---"
rm -rf "$CYPRESS_SCREENSHOTS"
pnpm cypress:run-headless --env TEST_LANGUAGE=en 2>&1 || { echo "ERROR: Cypress EN run failed"; exit 1; }

mkdir -p "$OUTPUT_DIR/en"
cp -r "$CYPRESS_SCREENSHOTS"/* "$OUTPUT_DIR/en/"
echo "EN screenshots copied to $OUTPUT_DIR/en/"

rm -rf "$ES_BACKUP"

echo ""
echo "=== Done. Screenshots available at $OUTPUT_DIR ==="
echo "EN: $(find "$OUTPUT_DIR/en" -name '*.png' | wc -l | tr -d ' ') files"
echo "ES: $(find "$OUTPUT_DIR/es" -name '*.png' | wc -l | tr -d ' ') files"
