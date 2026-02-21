#!/bin/bash
# Convert BigQuery-specific SQL to cross-platform using dbt macros
#
# This script converts:
# - Backtick identifiers: `column_name` -> {{ ident('column_name') }}
#
# Usage:
#   ./scripts/convert_to_cross_platform.sh [--dry-run]

set -euo pipefail

DBT_DIR="$(dirname "$0")/../src/dbt_lana_dw"
DRY_RUN=false

if [[ "${1:-}" == "--dry-run" ]]; then
    DRY_RUN=true
    echo "=== DRY RUN - No files will be modified ==="
fi

# Count files with backticks
count=$(grep -rl '`[a-z_]*`' "$DBT_DIR/models" --include="*.sql" 2>/dev/null | wc -l || echo 0)
echo "Found $count SQL files with backtick identifiers"

if [[ "$count" -eq 0 ]]; then
    echo "No files to convert."
    exit 0
fi

# List files
echo ""
echo "Files to convert:"
grep -rl '`[a-z_]*`' "$DBT_DIR/models" --include="*.sql" 2>/dev/null || true

if [[ "$DRY_RUN" == "true" ]]; then
    echo ""
    echo "Example conversions:"
    grep -rh '`[a-z_]*`' "$DBT_DIR/models" --include="*.sql" 2>/dev/null | head -5 | while read -r line; do
        converted=$(echo "$line" | sed "s/\`\([a-z_]*\)\`/{{ ident('\1') }}/g")
        echo "  Before: $line"
        echo "  After:  $converted"
        echo ""
    done
    exit 0
fi

# Perform conversion
echo ""
echo "Converting..."

find "$DBT_DIR/models" -name "*.sql" -exec grep -l '`[a-z_]*`' {} \; 2>/dev/null | while read -r file; do
    echo "  Converting: $file"
    # Create backup
    cp "$file" "$file.bak"
    # Convert backticks to ident() macro
    sed -i "s/\`\([a-z_]*\)\`/{{ ident('\1') }}/g" "$file"
done

echo ""
echo "Conversion complete. Backup files created with .bak extension."
echo "Review the changes and delete .bak files when satisfied."
