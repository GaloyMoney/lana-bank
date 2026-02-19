#!/bin/bash
set -euo pipefail

# Creates minimal placeholder PNGs for any screenshot references in markdown
# files that don't have actual files on disk. This allows the build to pass
# in CI even when screenshots haven't been fetched from Netlify.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCS_DIR="$(dirname "$SCRIPT_DIR")"
STATIC_DIR="$DOCS_DIR/static"

# Minimal valid 1x1 transparent PNG (67 bytes)
PLACEHOLDER_PNG_B64="iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg=="

# Find all screenshot references in markdown files
REFS=$(grep -roh '/img/screenshots/[^)"'"'"' ]*\.png' "$DOCS_DIR/docs" "$DOCS_DIR/versioned_docs" "$DOCS_DIR/i18n" 2>/dev/null | sort -u || true)

if [ -z "$REFS" ]; then
  echo "No screenshot references found in markdown files"
  exit 0
fi

CREATED=0
EXISTED=0

while IFS= read -r ref; do
  [ -z "$ref" ] && continue
  DEST="$STATIC_DIR$ref"

  if [ -f "$DEST" ]; then
    EXISTED=$((EXISTED + 1))
    continue
  fi

  mkdir -p "$(dirname "$DEST")"
  echo "$PLACEHOLDER_PNG_B64" | base64 -d > "$DEST"
  CREATED=$((CREATED + 1))
done <<< "$REFS"

echo "Screenshot placeholders: $CREATED created, $EXISTED already existed"
