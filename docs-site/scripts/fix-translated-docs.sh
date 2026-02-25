#!/usr/bin/env bash
set -euo pipefail

# fix-translated-docs.sh
#
# Post-processing for Lingo.dev markdown translations.
#
# Strategy: Strip whatever Lingo.dev produced for walkthrough sections,
# then restore them from a pre-run ES snapshot (preferred) or EN source
# with simple substitutions (fallback).
#
# Steps:
#   1) Rewrite leaked /current/en/ screenshot paths to /current/es/
#   2) Strip mangled walkthrough sections and restore from snapshot or EN
#   3) Validate: no /en/ paths, no missing screenshots
#
# Required env vars:
#   ES_WALKTHROUGH_SNAPSHOT  Path to snapshot dir (from translate-docs.sh)
#                            May be empty for standalone use (EN fallback only)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
DOCS_DIR="$SCRIPT_DIR/../docs"
ES_DIR="$SCRIPT_DIR/../i18n/es/docusaurus-plugin-content-docs/current"
ES_WALKTHROUGH_SNAPSHOT="${ES_WALKTHROUGH_SNAPSHOT:-}"

EN_WT_PATTERN='^## Admin Panel Walkthrough'
ES_WT_PATTERN='^## (Recorrido en Panel de Administración|Admin Panel Walkthrough)'

[ -d "$DOCS_DIR" ] || { echo "ERROR: EN docs not found: $DOCS_DIR" >&2; exit 1; }
[ -d "$ES_DIR" ] || { echo "ERROR: ES docs not found: $ES_DIR" >&2; exit 1; }

extract_screenshot_names() {
  grep -o '/img/screenshots/[^)]*\.png' "$1" 2>/dev/null | sed 's|.*/||' | sort -u || true
}

# find_walkthrough_zone FILE HEADING_PATTERN
# Prints start_line:end_line. Returns 1 if no walkthrough heading found.
# The zone spans from the first matching heading to just before the next
# non-matching ## heading, or to EOF if all remaining ## headings match.
find_walkthrough_zone() {
  local file="$1" pattern="$2"
  local first_line
  first_line=$(grep -n -m1 -E "$pattern" "$file" | cut -d: -f1)
  [ -z "$first_line" ] && return 1

  local total end_line
  total=$(wc -l < "$file" | tr -d ' ')
  end_line=$total

  while IFS= read -r match; do
    local lnum="${match%%:*}"
    [ "$lnum" -le "$first_line" ] && continue
    local text="${match#*:}"
    if ! printf '%s\n' "$text" | grep -qE "$pattern"; then
      end_line=$((lnum - 1))
      break
    fi
  done < <(grep -n '^## ' "$file")

  echo "$first_line:$end_line"
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

# ── Step 2: Strip and restore walkthrough sections ────────────────────
echo "Step 2: Processing walkthrough sections..."
restore_count=0

while IFS= read -r en_file; do
  rel_path="${en_file#$DOCS_DIR/}"
  es_file="$ES_DIR/$rel_path"
  [ -f "$es_file" ] || continue

  # Only process EN files that have walkthrough sections
  en_zone=$(find_walkthrough_zone "$en_file" "$EN_WT_PATTERN") || continue
  en_wt_start=${en_zone%%:*}
  en_wt_end=${en_zone##*:}

  # Extract walkthrough content from EN
  en_walkthrough=$(sed -n "${en_wt_start},${en_wt_end}p" "$en_file")

  # ── Choose walkthrough source: ES snapshot (preferred) or EN fallback ──
  snapshot_file=""
  [ -n "$ES_WALKTHROUGH_SNAPSHOT" ] && snapshot_file="$ES_WALKTHROUGH_SNAPSHOT/$rel_path"

  if [ -n "$snapshot_file" ] && [ -f "$snapshot_file" ]; then
    en_wt_screenshots=$(printf '%s\n' "$en_walkthrough" \
      | grep -o '/img/screenshots/[^)]*\.png' 2>/dev/null \
      | sed 's|.*/||' | sort -u || true)
    snap_screenshots=$(extract_screenshot_names "$snapshot_file")

    missing=""
    if [ -n "$en_wt_screenshots" ]; then
      missing=$(comm -23 \
        <(printf '%s\n' "$en_wt_screenshots") \
        <(printf '%s\n' "$snap_screenshots") 2>/dev/null || true)
    fi

    if [ -z "$missing" ]; then
      # ES snapshot has all expected screenshots — use it (fix paths as safety)
      restored=$(sed 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' "$snapshot_file")
      source_label="ES snapshot"
    else
      # ES snapshot missing screenshots (EN added new steps) — EN fallback
      restored=$(printf '%s\n' "$en_walkthrough" | sed -E \
        -e 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' \
        -e 's|^## Admin Panel Walkthrough|## Recorrido en Panel de Administración|' \
        -e 's|^\*\*Step ([0-9]+)\.|**Paso \1.|')
      source_label="EN fallback (snapshot missing: $(echo "$missing" | tr '\n' ' '))"
    fi
  else
    # No snapshot available — EN fallback with substitutions
    restored=$(printf '%s\n' "$en_walkthrough" | sed -E \
      -e 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' \
      -e 's|^## Admin Panel Walkthrough|## Recorrido en Panel de Administración|' \
      -e 's|^\*\*Step ([0-9]+)\.|**Paso \1.|')
    source_label="EN fallback (no snapshot)"
  fi

  # ── Replace walkthrough zone in ES file ──
  if es_zone=$(find_walkthrough_zone "$es_file" "$ES_WT_PATTERN"); then
    es_wt_start=${es_zone%%:*}
    es_wt_end=${es_zone##*:}
    es_total=$(wc -l < "$es_file" | tr -d ' ')

    # Rebuild: [prose before walkthrough] + [restored] + [post-walkthrough]
    tmp=$(mktemp)
    if [ "$es_wt_start" -gt 1 ]; then
      head -n "$((es_wt_start - 1))" "$es_file" > "$tmp"
    fi
    printf '%s\n' "$restored" >> "$tmp"
    if [ "$es_wt_end" -lt "$es_total" ]; then
      # Ensure blank line before post-walkthrough content
      printf '\n' >> "$tmp"
      tail -n "+$((es_wt_end + 1))" "$es_file" >> "$tmp"
    fi
    mv "$tmp" "$es_file"
  else
    # No walkthrough in ES yet — append at end
    printf '\n%s\n' "$restored" >> "$es_file"
  fi

  echo "  $rel_path <- $source_label"
  restore_count=$((restore_count + 1))
done < <(find "$DOCS_DIR" -name '*.md' -type f)
echo "  Processed $restore_count file(s)"

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
