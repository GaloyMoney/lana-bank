# End-of-Line (EOL) Configuration Setup

This repository now enforces consistent LF (Unix-style) line endings across all file types using a multi-layered approach.

## 🔧 Configuration Files

### 1. `.editorconfig`
- Enforces consistent formatting across different editors and IDEs
- Sets `end_of_line = lf` for all files
- Configures appropriate indentation for each file type
- Ensures final newlines and trims trailing whitespace

### 2. `.gitattributes`
- Normalizes line endings at the Git level
- Forces LF endings for all text files
- Properly handles binary files
- Works regardless of contributor's OS or Git settings

### 3. `rustfmt.toml`
- Configures Rust formatter to use Unix line endings
- Sets `newline_style = "Unix"`
- Ensures consistent Rust code formatting

### 4. Git Configuration
- `core.autocrlf = false` - Disables automatic conversion
- `core.eol = lf` - Sets default line ending to LF

### 5. Prettier Configuration
- Already configured in frontend apps with `endOfLine: "lf"`
- Maintains consistency across TypeScript/JavaScript files

## 🚀 One-Time Setup

Run the normalization script to fix existing files:

```bash
./scripts/normalize-line-endings.sh
```

This script will:
- Detect files with CRLF line endings
- Convert them to LF
- Remove trailing whitespace
- Ensure files end with a newline

## 📋 How It Works

1. **EditorConfig**: Your editor/IDE automatically applies the rules when editing files
2. **Git Attributes**: Git normalizes line endings when files are committed/checked out
3. **Formatters**: 
   - Prettier (JS/TS): Already configured with `endOfLine: "lf"`
   - rustfmt (Rust): Configured with `newline_style = "Unix"`

## ✅ Verification

Check for CRLF files:
```bash
find . -type f -name "*.rs" -o -name "*.ts" -o -name "*.py" | xargs file | grep CRLF
```

If this returns nothing, all files have LF endings! 🎉

## 🔄 For New Contributors

New contributors don't need to do anything special:
- Their editor will automatically use the settings from `.editorconfig`
- Git will automatically normalize line endings via `.gitattributes`
- Formatters will enforce the correct line endings

## 🛠️ CI/CD Integration

Consider adding these checks to your CI pipeline:

```bash
# Check for CRLF line endings
find . -type f -name "*.rs" -o -name "*.ts" -o -name "*.py" | xargs file | grep CRLF && exit 1

# Run formatters
cargo fmt --check
cd apps/admin-panel && pnpm run format:check
cd apps/customer-portal && pnpm run format:check
```

## 📖 File Type Coverage

This setup handles all file types in your repository:
- **Rust**: `.rs` files via rustfmt + EditorConfig
- **TypeScript/JavaScript**: `.ts`, `.tsx`, `.js`, `.jsx` via Prettier + EditorConfig  
- **TOML**: `.toml` files via EditorConfig
- **Python**: `.py` files via EditorConfig
- **Shell**: `.sh`, `.bash` files via EditorConfig
- **YAML**: `.yml`, `.yaml` files via EditorConfig
- **JSON**: `.json` files via EditorConfig
- **SQL**: `.sql` files via EditorConfig
- **GraphQL**: `.gql` files via EditorConfig
- **Markdown**: `.md` files via EditorConfig
- **CSS/HTML**: `.css`, `.html` files via EditorConfig

## 🎯 Benefits

1. **Consistency**: All files use the same line ending style
2. **Cross-platform**: Works on Windows, macOS, and Linux
3. **Editor agnostic**: Works with VS Code, IntelliJ, Vim, etc.
4. **Automatic**: No manual intervention required
5. **Git-level**: Prevents inconsistent line endings in commits
6. **Comprehensive**: Covers all file types in your monorepo