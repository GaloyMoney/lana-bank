#!/bin/bash

# Script to normalize line endings in the repository
# This should be run once after setting up .editorconfig and .gitattributes

set -e

echo "🔧 Normalizing line endings in the repository..."

# First, let's see what files have CRLF line endings
echo "📋 Checking for files with CRLF line endings..."
CRLF_FILES=$(find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.json" -o -name "*.md" -o -name "*.yml" -o -name "*.yaml" -o -name "*.py" -o -name "*.sh" -o -name "*.bash" -o -name "*.sql" -o -name "*.gql" -o -name "*.css" -o -name "*.html" \) \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    -exec grep -l $'\r' {} \; 2>/dev/null || true)

if [ -n "$CRLF_FILES" ]; then
    echo "🔄 Found files with CRLF line endings:"
    echo "$CRLF_FILES"

    echo "🛠️ Converting CRLF to LF..."
    echo "$CRLF_FILES" | while read -r file; do
        if [ -n "$file" ]; then
            echo "  Converting: $file"
            # Convert CRLF to LF
            sed -i 's/\r$//' "$file"
        fi
    done
else
    echo "✅ No files with CRLF line endings found!"
fi

# Remove trailing whitespace from all text files (except lock files)
echo "🧹 Removing trailing whitespace..."
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.json" -o -name "*.yml" -o -name "*.yaml" -o -name "*.py" -o -name "*.sh" -o -name "*.bash" -o -name "*.sql" -o -name "*.gql" -o -name "*.css" -o -name "*.html" \) \
    ! -name "*.lock" \
    ! -name "pnpm-lock.yaml" \
    ! -name "Cargo.lock" \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    -exec sed -i 's/[[:space:]]*$//' {} \;

# Ensure files end with a newline (except lock files)
echo "📝 Ensuring files end with newline..."
find . -type f \( -name "*.rs" -o -name "*.toml" -o -name "*.ts" -o -name "*.tsx" -o -name "*.js" -o -name "*.jsx" -o -name "*.json" -o -name "*.yml" -o -name "*.yaml" -o -name "*.py" -o -name "*.sh" -o -name "*.bash" -o -name "*.sql" -o -name "*.gql" -o -name "*.css" -o -name "*.html" \) \
    ! -name "*.lock" \
    ! -name "pnpm-lock.yaml" \
    ! -name "Cargo.lock" \
    ! -path "./target/*" \
    ! -path "./.git/*" \
    ! -path "./node_modules/*" \
    -exec sh -c 'if [ -s "$1" ] && [ "$(tail -c 1 "$1" | wc -l)" -eq 0 ]; then echo "" >> "$1"; fi' _ {} \;

echo "✨ Line ending normalization complete!"

echo ""
echo "🎯 Next steps:"
echo "1. Run 'git add .' to stage the changes"
echo "2. Run 'git commit -m \"Normalize line endings\"' to commit"
echo "3. Consider running formatters:"
echo "   - Rust: 'cargo fmt'"
echo "   - TypeScript/JavaScript: 'pnpm run format' (in frontend apps)"
echo "4. Add this script to your CI/CD to prevent future inconsistencies"
