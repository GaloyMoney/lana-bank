#!/bin/bash
# analyze_sql_errors.sh - Analyze SQL files for transpilation errors

# Set bash to exit on error
set -e

COMPILED_DIR="meltano/.meltano/transformers/dbt/target/compiled"
ERROR_FILES_LOG="/tmp/sqlglot-errors.txt"
EXCLUSIONS_FILE="meltano/scripts/sqlglot_exclusions.txt"

echo "Scanning for SQL files with transpilation errors (excluding patterns from $EXCLUSIONS_FILE)..."
find "$COMPILED_DIR" -type f -name "*.sql" -print0 | xargs -0 -n1 bash -c "python meltano/scripts/transpile_sql.py \"\$0\" --exclusions-file \"$EXCLUSIONS_FILE\" >/dev/null 2>&1 || echo \"\$0\"" > "$ERROR_FILES_LOG"

# Count the number of error files
ERROR_COUNT=$(wc -l < "$ERROR_FILES_LOG")
if [[ "$ERROR_COUNT" -eq 0 ]]; then
  echo "✅ No SQL files with transpilation errors found!"
  exit 0
fi

echo "Found $ERROR_COUNT files with transpilation errors:"
cat "$ERROR_FILES_LOG" | nl

echo ""
echo "To analyze a specific file with detailed error information, run:"
echo "  python meltano/scripts/transpile_sql.py FILE_PATH --debug"
echo ""
echo "Example analysis of the first error file:"
FIRST_ERROR_FILE=$(head -n 1 "$ERROR_FILES_LOG")
if [[ -n "$FIRST_ERROR_FILE" ]]; then
  echo "=== Analysis of $FIRST_ERROR_FILE ==="
  python meltano/scripts/transpile_sql.py "$FIRST_ERROR_FILE" --debug 2>&1 | grep -v "^[^❌]" || true
  echo ""
  echo "For more details on this file, run:"
  echo "  python meltano/scripts/transpile_sql.py \"$FIRST_ERROR_FILE\" --debug"
fi

echo ""
echo "Common sqlglot parsing errors include:"
echo "1. JavaScript UDFs embedded in SQL"
echo "2. BigQuery-specific syntax (like ARRAY_AGG with HAVING)"
echo "3. Procedural SQL code (like loops and conditionals)"
echo "4. Language-specific SQL extensions"
echo ""
echo "Possible solutions:"
echo "1. Add patterns to exclude files in $EXCLUSIONS_FILE"
echo "2. Create dialect-specific versions of these files"
echo "3. Simplify complex SQL constructs to be more standard SQL"
echo ""
echo "To add a file pattern to exclusions, edit $EXCLUSIONS_FILE" 