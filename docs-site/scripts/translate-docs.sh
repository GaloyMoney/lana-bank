#!/usr/bin/env bash
set -euo pipefail

# Translate docs-site English docs to Spanish using Lingo.dev.
#
# Lingo.dev expects a symmetric [locale]/ layout, but Docusaurus uses:
#   EN: docs-site/docs/
#   ES: docs-site/i18n/es/docusaurus-plugin-content-docs/current/
#
# This script creates temporary symlinks so Lingo.dev sees:
#   .lingo-docs/en  →  ../docs
#   .lingo-docs/es  →  ../i18n/es/docusaurus-plugin-content-docs/current
#
# Usage:
#   LINGO_API_KEY=<key> bash docs-site/scripts/translate-docs.sh
#   # or: npx lingo.dev auth   (interactive login, then run without key)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINGO_DIR="$SCRIPT_DIR/../.lingo-docs"
ES_DIR="$SCRIPT_DIR/../i18n/es/docusaurus-plugin-content-docs/current"

cleanup() {
  rm -f "$LINGO_DIR/en" "$LINGO_DIR/es"
}
trap cleanup EXIT

# Ensure the ES target directory exists
mkdir -p "$ES_DIR"

# Create symlinks
ln -sf ../docs "$LINGO_DIR/en"
ln -sf ../i18n/es/docusaurus-plugin-content-docs/current "$LINGO_DIR/es"

# Run Lingo.dev from the .lingo-docs directory
cd "$LINGO_DIR"
npx lingo.dev@latest i18n
