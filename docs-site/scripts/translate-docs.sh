#!/usr/bin/env bash
set -euo pipefail

# translate-docs.sh
#
# Translates docs-site English docs to Spanish using Lingo.dev.
#
# Strategy: snapshot ES walkthrough sections, run Lingo.dev on full docs,
# then post-process to strip mangled walkthroughs and restore from snapshot.
#
# Usage:
#   bash docs-site/scripts/translate-docs.sh
#   bash docs-site/scripts/translate-docs.sh --full-run

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LINGO_DIR="$SCRIPT_DIR/../.lingo-docs"
DOCS_DIR="$SCRIPT_DIR/../docs"
ES_DIR="$SCRIPT_DIR/../i18n/es/docusaurus-plugin-content-docs/current"

ES_WT_PATTERN='^## (Recorrido en Panel de Administración|Admin Panel Walkthrough)'

while [ "$#" -gt 0 ]; do
  case "$1" in
    --full-run) shift ;;
    -h|--help)
      echo "Usage: bash docs-site/scripts/translate-docs.sh [--full-run]"
      exit 0
      ;;
    *) echo "ERROR: Unknown argument: $1" >&2; exit 1 ;;
  esac
done

[ -d "$LINGO_DIR" ] || { echo "ERROR: Lingo config dir not found: $LINGO_DIR" >&2; exit 1; }
[ -d "$DOCS_DIR" ] || { echo "ERROR: EN docs dir not found: $DOCS_DIR" >&2; exit 1; }

ES_WALKTHROUGH_SNAPSHOT=""

# Lingo.dev currently fails this large file with a reference-payload limit, so
# keep the checked-in Spanish copy and allow that specific failure to pass.
is_allowed_lingo_failure() {
  local log_file="$1"
  local file="$2"

  case "$file" in
    es/for-platform-engineers/functional-architecture.md)
      awk -v file="$file" '
        $0 ~ ("❌ " file " ") { seen=1; next }
        seen && /Reference payload too large/ { found=1; exit }
        seen && /❌ / { exit }
        END { exit found ? 0 : 1 }
      ' "$log_file"
      ;;
    *)
      return 1
      ;;
  esac
}

cleanup() {
  rm -f "$LINGO_DIR/en" "$LINGO_DIR/es" "$LINGO_DIR/scripts"
  [ -n "$ES_WALKTHROUGH_SNAPSHOT" ] && rm -rf "$ES_WALKTHROUGH_SNAPSHOT"
  true
}
trap cleanup EXIT

mkdir -p "$ES_DIR"

# Create symlinks for Lingo.dev locale layout
# -n prevents following existing symlinks (avoids creating nested artifacts like docs/docs)
ln -sfn ../docs "$LINGO_DIR/en"
ln -sfn ../i18n/es/docusaurus-plugin-content-docs/current "$LINGO_DIR/es"
ln -sfn ../scripts "$LINGO_DIR/scripts"

# ── Snapshot ES walkthrough sections before Lingo.dev can mangle them ──
ES_WALKTHROUGH_SNAPSHOT="$(mktemp -d)"
echo "Snapshotting ES walkthrough sections..."
snapshot_count=0

if [ -d "$ES_DIR" ]; then
  while IFS= read -r es_file; do
    rel_path="${es_file#$ES_DIR/}"

    # Find first walkthrough heading
    first_wt=$(grep -n -m1 -E "$ES_WT_PATTERN" "$es_file" | cut -d: -f1 || true)
    [ -z "$first_wt" ] && continue

    # Find end of walkthrough zone (next non-walkthrough ## heading, or EOF)
    total=$(wc -l < "$es_file" | tr -d ' ')
    end_line=$total

    while IFS= read -r match; do
      lnum="${match%%:*}"
      [ "$lnum" -le "$first_wt" ] && continue
      text="${match#*:}"
      if ! printf '%s\n' "$text" | grep -qE "$ES_WT_PATTERN"; then
        end_line=$((lnum - 1))
        break
      fi
    done < <(grep -n '^## ' "$es_file")

    # Save snapshot of walkthrough zone only
    snapshot_file="$ES_WALKTHROUGH_SNAPSHOT/$rel_path"
    mkdir -p "$(dirname "$snapshot_file")"
    sed -n "${first_wt},${end_line}p" "$es_file" > "$snapshot_file"
    snapshot_count=$((snapshot_count + 1))
  done < <(find "$ES_DIR" -name '*.md' -type f)
fi
echo "  Snapshotted $snapshot_count file(s)"

# ── Run Lingo.dev (always full — incremental --file is broken) ──
echo "Running Lingo.dev..."
LINGO_LOG_FILE="$(mktemp)"

pushd "$LINGO_DIR" >/dev/null
set +e
npx lingo.dev@latest run 2>&1 | tee "$LINGO_LOG_FILE"
lingo_status=${PIPESTATUS[0]}
set -e
popd >/dev/null

# Detect Lingo.dev partial failures (it exits 0 even when files fail)
failed_count=$(grep -c '❌' "$LINGO_LOG_FILE" || true)
allow_known_failures=0

if [ "$failed_count" -gt 0 ]; then
  mapfile -t failed_files < <(
    sed -nE 's/^.*❌ (es\/[^ ]+\.md) \(en → es\)$/\1/p' "$LINGO_LOG_FILE"
  )

  if [ "${#failed_files[@]}" -gt 0 ]; then
    allow_known_failures=1
    for failed_file in "${failed_files[@]}"; do
      if ! is_allowed_lingo_failure "$LINGO_LOG_FILE" "$failed_file"; then
        allow_known_failures=0
        break
      fi
    done
  fi
fi

if [ "$lingo_status" -ne 0 ] && [ "$allow_known_failures" -ne 1 ]; then
  echo "ERROR: Lingo.dev failed with exit code $lingo_status" >&2
  rm -f "$LINGO_LOG_FILE"
  exit "$lingo_status"
fi

if [ "$failed_count" -gt 0 ]; then
  if [ "$allow_known_failures" -eq 1 ]; then
    echo ""
    echo "WARNING: Ignoring known Lingo.dev failure for oversized reference payload:" >&2
    grep '❌' "$LINGO_LOG_FILE" >&2
  else
    echo ""
    echo "ERROR: Lingo.dev failed to translate $failed_count file(s):" >&2
    grep '❌' "$LINGO_LOG_FILE" >&2
    if grep -q 'Maximum number of translated words' "$LINGO_LOG_FILE"; then
      echo "" >&2
      echo "Root cause: Lingo.dev free plan word limit exhausted." >&2
      echo "Aborting to prevent committing deletion-only changes." >&2
    fi
    rm -f "$LINGO_LOG_FILE"
    exit 1
  fi
fi

rm -f "$LINGO_LOG_FILE"

# ── Post-process translations ──
echo "Running translation post-processing..."
export ES_WALKTHROUGH_SNAPSHOT
bash "$SCRIPT_DIR/fix-translated-docs.sh"

echo "Done. Translation flow complete."
