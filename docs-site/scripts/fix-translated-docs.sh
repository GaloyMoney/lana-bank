#!/usr/bin/env bash
set -euo pipefail

# fix-translated-docs.sh
#
# Post-processing for Lingo.dev markdown translations.
#
# Lingo.dev translates all content including walkthrough sections.
# Post-processing only needs to:
#   1) Rewrite leaked /current/en/ screenshot paths to /current/es/
#   1b) Fix file-path links to generated API docs for i18n
#   2) Validate: no /en/ paths, no missing screenshots

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="$SCRIPT_DIR/../docs"
ES_DIR="$SCRIPT_DIR/../i18n/es/docusaurus-plugin-content-docs/current"

[ -d "$DOCS_DIR" ] || { echo "ERROR: EN docs not found: $DOCS_DIR" >&2; exit 1; }
[ -d "$ES_DIR" ] || { echo "ERROR: ES docs not found: $ES_DIR" >&2; exit 1; }

extract_screenshot_names() {
  grep -o '/img/screenshots/[^)]*\.png' "$1" 2>/dev/null | sed 's|.*/||' | sort -u || true
}

# ── Step 1: Fix /en/ → /es/ screenshot paths ─────────────────────────
echo "Step 1: Fixing /en/ → /es/ screenshot paths..."
fix1_count=0
while IFS= read -r es_file; do
  if grep -q '/img/screenshots/current/en/' "$es_file"; then
    tmp=$(mktemp)
    sed 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' "$es_file" > "$tmp"
    mv "$tmp" "$es_file"
    fix1_count=$((fix1_count + 1))
  fi
done < <(find "$ES_DIR" -name '*.md' -type f)
echo "  Rewrote paths in $fix1_count file(s)"

# ── Step 1b: Fix file-path links to generated API docs ────────────────
# Generated API docs (admin-api, customer-api) don't exist in the i18n
# tree, so file-path links (.mdx extension) break the ES build. Replace
# with URL-relative slug-based links that Docusaurus resolves correctly.
echo "Step 1b: Fixing API reference links for i18n..."
fix1b_count=0
while IFS= read -r es_file; do
  if grep -q 'api-reference\.mdx' "$es_file"; then
    tmp=$(mktemp)
    sed -E 's|(\([^)]*)/api-reference\.mdx\)|\1)|g' "$es_file" > "$tmp"
    mv "$tmp" "$es_file"
    fix1b_count=$((fix1b_count + 1))
  fi
done < <(find "$ES_DIR" -name '*.md' -type f)
echo "  Fixed API links in $fix1b_count file(s)"

# ── Validation ────────────────────────────────────────────────────────
echo "Validation..."
errors=0

while IFS= read -r es_file; do
  rel_path="${es_file#$ES_DIR/}"
  en_file="$DOCS_DIR/$rel_path"
  [ -f "$en_file" ] || continue

  if grep -q '/img/screenshots/current/en/' "$es_file"; then
    echo "ERROR: EN screenshot path in ES: $rel_path" >&2
    errors=$((errors + 1))
  fi

  en_names=$(extract_screenshot_names "$en_file")
  [ -z "$en_names" ] && continue
  es_names=$(extract_screenshot_names "$es_file")

  missing=""
  if [ -z "$es_names" ]; then
    missing="$en_names"
  else
    missing=$(comm -23 \
      <(printf '%s\n' "$en_names") \
      <(printf '%s\n' "$es_names") 2>/dev/null || true)
  fi

  if [ -n "$missing" ]; then
    echo "ERROR: ES missing screenshots in $rel_path:" >&2
    printf '%s\n' "$missing" | sed 's/^/  - /' >&2
    errors=$((errors + 1))
  fi
done < <(find "$ES_DIR" -name '*.md' -type f)

if [ "$errors" -ne 0 ]; then
  echo "Post-processing failed with $errors error(s)." >&2
  exit 1
fi

echo "Done. Post-processing complete."
