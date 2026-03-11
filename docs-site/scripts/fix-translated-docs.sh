#!/usr/bin/env bash
set -euo pipefail

# fix-translated-docs.sh
#
# Post-processing for Lingo.dev markdown translations.
#
# Strategy: Strip whatever Lingo.dev produced for walkthrough sections,
# then restore them from the ES file on the main branch (preferred),
# the pre-run ES snapshot (second choice), or EN source with simple
# substitutions (last resort).
#
# Steps:
#   1) Rewrite leaked /current/en/ screenshot paths to /current/es/
#   2) Strip mangled walkthrough sections and restore from snapshot, main, or EN
#   3) Validate: no /en/ paths, no missing screenshots
#
# Required env vars:
#   ES_WALKTHROUGH_SNAPSHOT  Path to snapshot dir (from translate-docs.sh)
#                            May be empty for standalone use (git-main / EN fallback)

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
DOCS_DIR="$SCRIPT_DIR/../docs"
ES_DIR="$SCRIPT_DIR/../i18n/es/docusaurus-plugin-content-docs/current"
ES_WALKTHROUGH_SNAPSHOT="${ES_WALKTHROUGH_SNAPSHOT:-}"

EN_WT_PATTERN='^## Admin Panel Walkthrough'
ES_WT_PATTERN='^## (Recorrido|Tutorial|Guía|Admin Panel Walkthrough)'

[ -d "$DOCS_DIR" ] || { echo "ERROR: EN docs not found: $DOCS_DIR" >&2; exit 1; }
[ -d "$ES_DIR" ] || { echo "ERROR: ES docs not found: $ES_DIR" >&2; exit 1; }

extract_screenshot_names() {
  grep -o '/img/screenshots/[^)]*\.png' "$1" 2>/dev/null | sed 's|.*/||' | sort -u || true
}

extract_screenshot_names_stdin() {
  grep -o '/img/screenshots/[^)]*\.png' 2>/dev/null | sed 's|.*/||' | sort -u || true
}

# extract_git_walkthrough REL_PATH
# Extracts walkthrough zone from the ES file on the main branch.
# Prints the walkthrough text (with /en/ paths rewritten to /es/).
# Returns 1 if the file or walkthrough doesn't exist on main.
extract_git_walkthrough() {
  local rel_path="$1"
  local git_path="docs-site/i18n/es/docusaurus-plugin-content-docs/current/$rel_path"
  local content
  content=$(git -C "$REPO_ROOT" show "main:$git_path" 2>/dev/null) || return 1
  [ -z "$content" ] && return 1

  # Find walkthrough zone in the git content
  local first_line
  first_line=$(printf '%s\n' "$content" | grep -n -m1 -E "$ES_WT_PATTERN" | cut -d: -f1)
  [ -z "$first_line" ] && return 1

  local total end_line
  total=$(printf '%s\n' "$content" | wc -l | tr -d ' ')
  end_line=$total

  while IFS= read -r match; do
    local lnum="${match%%:*}"
    [ "$lnum" -le "$first_line" ] && continue
    local text="${match#*:}"
    if ! printf '%s\n' "$text" | grep -qE "$ES_WT_PATTERN"; then
      end_line=$((lnum - 1))
      break
    fi
  done < <(printf '%s\n' "$content" | grep -n '^## ')

  printf '%s\n' "$content" | sed -n "${first_line},${end_line}p" \
    | sed 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g'
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

replace_trailing_section() {
  local file="$1" heading="$2"
  local start_line tmp

  [ -f "$file" ] || return 0
  start_line=$(grep -n -m1 -iF "$heading" "$file" | cut -d: -f1)
  [ -n "$start_line" ] || return 0

  tmp=$(mktemp)
  head -n "$((start_line - 1))" "$file" > "$tmp"
  cat >> "$tmp"
  mv "$tmp" "$file"
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

  # ── Choose walkthrough source: git main > snapshot > EN fallback ──
  # git main is preferred because the working-tree snapshot may already
  # contain corrupted EN-fallback text from a previous run.
  en_wt_screenshots=$(printf '%s\n' "$en_walkthrough" \
    | grep -o '/img/screenshots/[^)]*\.png' 2>/dev/null \
    | sed 's|.*/||' | sort -u || true)

  restored=""
  source_label=""

  # 1) Try git main (known-good ES walkthrough from the main branch)
  git_walkthrough=$(extract_git_walkthrough "$rel_path" || true)
  if [ -n "$git_walkthrough" ]; then
    git_screenshots=$(printf '%s\n' "$git_walkthrough" | extract_screenshot_names_stdin)
    missing=""
    if [ -n "$en_wt_screenshots" ]; then
      missing=$(comm -23 \
        <(printf '%s\n' "$en_wt_screenshots") \
        <(printf '%s\n' "$git_screenshots") 2>/dev/null || true)
    fi

    if [ -z "$missing" ]; then
      # Guard against pre-existing duplicates in git-main content:
      # if git-main has more walkthrough headings than EN, trim the extras.
      en_wt_count=$(printf '%s\n' "$en_walkthrough" | grep -cE "$EN_WT_PATTERN" || true)
      git_wt_count=$(printf '%s\n' "$git_walkthrough" | grep -cE "$ES_WT_PATTERN" || true)
      if [ "$git_wt_count" -gt "$en_wt_count" ] && [ "$en_wt_count" -gt 0 ]; then
        nth_next=$((en_wt_count + 1))
        cut_line=$(printf '%s\n' "$git_walkthrough" \
          | grep -n -E "$ES_WT_PATTERN" \
          | sed -n "${nth_next}p" | cut -d: -f1)
        if [ -n "$cut_line" ]; then
          git_walkthrough=$(printf '%s\n' "$git_walkthrough" | head -n "$((cut_line - 1))")
        fi
      fi
      restored="$git_walkthrough"
      source_label="git main"
    fi
  fi

  # 2) Try working-tree snapshot (captured before Lingo.dev ran)
  if [ -z "$restored" ]; then
    snapshot_file=""
    [ -n "$ES_WALKTHROUGH_SNAPSHOT" ] && snapshot_file="$ES_WALKTHROUGH_SNAPSHOT/$rel_path"

    if [ -n "$snapshot_file" ] && [ -f "$snapshot_file" ]; then
      snap_screenshots=$(extract_screenshot_names "$snapshot_file")
      missing=""
      if [ -n "$en_wt_screenshots" ]; then
        missing=$(comm -23 \
          <(printf '%s\n' "$en_wt_screenshots") \
          <(printf '%s\n' "$snap_screenshots") 2>/dev/null || true)
      fi

      if [ -z "$missing" ]; then
        restored=$(sed 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' "$snapshot_file")
        source_label="ES snapshot"
      fi
    fi
  fi

  # 3) EN fallback (last resort — minimal substitutions)
  if [ -z "$restored" ]; then
    restored=$(printf '%s\n' "$en_walkthrough" | sed -E \
      -e 's|/img/screenshots/current/en/|/img/screenshots/current/es/|g' \
      -e 's|^## Admin Panel Walkthrough|## Recorrido en Panel de Administración|' \
      -e 's|^\*\*Step ([0-9]+)\.|**Paso \1.|')
    source_label="EN fallback"
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

# ── Step 3: Normalize repeated Lingo tail sections ────────────────────
echo "Step 3: Normalizing repeated tail sections..."

replace_trailing_section \
  "$ES_DIR/for-platform-engineers/system-architecture.md" \
  "### Patrón CQRS" <<'EOF'
### Patrón CQRS

Segregación de responsabilidad de comandos y consultas:
- Rutas de lectura optimizadas
- Operaciones de escritura separadas
- Consistencia eventual cuando sea apropiado
EOF

replace_trailing_section \
  "$ES_DIR/technical-documentation/accounting/fiscal-year.md" \
  "### Resumen del flujo de trabajo" <<'EOF'
### Resumen del Flujo de Trabajo

```mermaid
flowchart LR
    A[Inicializar primer ejercicio fiscal] --> B[Operar durante el año]
    B --> C[Cerrar meses 1-12 secuencialmente]
    C --> D[Cerrar ejercicio fiscal]
    D --> E[Abrir siguiente ejercicio fiscal]
    E --> B
```

Este ciclo se repite anualmente. Cada ejercicio fiscal proporciona un límite claro para la presentación de informes financieros y garantiza que los libros del banco se cierren y trasladen adecuadamente a intervalos regulares.
EOF

replace_trailing_section \
  "$ES_DIR/technical-documentation/credit/disbursal.md" \
  "## Qué verificar después del paso 29" <<'EOF'
## Qué Verificar Después del Paso 29

- El estado del desembolso es `Confirmed`.
- El desembolso es visible bajo la facilidad y cliente esperados.
- El historial de la facilidad refleja la actividad de ejecución/liquidación.
- Las vistas de repago muestran el impacto de la obligación para el nuevo principal.
EOF

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
