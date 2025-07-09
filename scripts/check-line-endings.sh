#!/bin/bash

# Script to check for CRLF line endings in the repository
# Returns exit code 1 if any CRLF files are found

set -e

echo "🔍 Checking for CRLF line endings..."

# Check for files with CRLF line endings
CRLF_FILES=$(find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.json" -o -name "*.md" -o -name "*.yml" -o -name "*.yaml" -o -name "*.py" -o -name "*.sh" -o -name "*.bash" -o -name "*.sql" -o -name "*.gql" -o -name "*.css" -o -name "*.html" \) \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    ! -name "*.lock" \
    ! -name "pnpm-lock.yaml" \
    ! -name "Cargo.lock" \
    -exec grep -l $'\r' {} \; 2>/dev/null || true)

if [ -n "$CRLF_FILES" ]; then
    echo "❌ Found files with CRLF line endings:"
    echo "$CRLF_FILES"
    echo ""
    echo "💡 To fix this, run: ./scripts/normalize-line-endings.sh"
    exit 1
else
    echo "✅ All files use LF line endings"
fi