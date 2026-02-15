#!/bin/bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
DOCS_DIR="$(dirname "$SCRIPT_DIR")"
OUTPUT_DIR="$DOCS_DIR/static/img/screenshots"
VERSIONS_FILE="$DOCS_DIR/screenshot-versions.json"
NETLIFY_SITE="lana-manuals.netlify.app"

if [ ! -f "$VERSIONS_FILE" ]; then
  echo "Warning: $VERSIONS_FILE not found, skipping screenshot fetch"
  exit 0
fi

for version in $(jq -r 'keys[]' "$VERSIONS_FILE"); do
  commit=$(jq -r --arg v "$version" '.[$v]' "$VERSIONS_FILE")

  if [ -n "${SCREENSHOTS_BASE_URL:-}" ]; then
    BASE_URL="$SCREENSHOTS_BASE_URL"
  elif [ "$commit" = "latest" ]; then
    BASE_URL="https://$NETLIFY_SITE"
  else
    BASE_URL="https://commit-${commit}--${NETLIFY_SITE}"
  fi

  MANIFEST_URL="$BASE_URL/screenshots/manifest.txt"
  echo "Fetching manifest for version '$version' from $MANIFEST_URL"

  MANIFEST=$(curl -sf "$MANIFEST_URL" || true)
  if [ -z "$MANIFEST" ]; then
    echo "Warning: Could not fetch manifest for version '$version', skipping"
    continue
  fi

  while IFS= read -r file; do
    [ -z "$file" ] && continue

    # manifest entries are like: en/credit-facilities.cy.ts/01_screenshot.png
    DEST="$OUTPUT_DIR/$version/$file"
    mkdir -p "$(dirname "$DEST")"

    if [ -f "$DEST" ]; then
      continue
    fi

    URL="$BASE_URL/screenshots/$file"
    echo "  Downloading $file"
    curl -sf -o "$DEST" "$URL" || echo "  Warning: Failed to download $URL"
  done <<< "$MANIFEST"

  echo "Done fetching screenshots for version '$version'"
done

echo "All screenshots fetched to $OUTPUT_DIR"
