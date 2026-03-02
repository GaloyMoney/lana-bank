---
name: lana-multi-commit
description: Multi-Commit Workflow Instructions
---

# Multi-Commit Workflow Instructions

## CRITICAL: Staged Commit Workflow

When implementing plans that require multiple commits, you MUST follow this exact workflow. DO NOT make all changes first and then try to commit by selectively staging already executed changes.

### The Correct Workflow

For each commit in a multi-commit plan:

1. **Make ONLY the changes for that specific commit**
   - Do not touch files that belong to later commits
   - Keep the scope narrow to just what's needed for this commit

2. **Run verification checks BEFORE committing**
   - Run `SQLX_OFFLINE=true cargo check -p <package>` for Rust changes
   - Run `make check-code-rust` if touching multiple packages
   - Run `make sdl-rust` if GraphQL schema changes are involved
   - Run `make sdl-js` and `make check-code-apps` for frontend changes

3. **Commit the changes**
   - Stage only the files for this commit
   - Write a clear commit message
   - Verify the commit succeeded with `git log -1`

4. **Verify clean state before proceeding**
   - Run `git status` to ensure working directory is as expected
   - Only then proceed to the next commit

### Example: Three-Commit Refactor

```
# Commit 1: Core changes
1. Edit core/module/file1.rs
2. Edit core/module/file2.rs
3. Run: SQLX_OFFLINE=true cargo check -p core-module
4. Run: git add core/module/file1.rs core/module/file2.rs
5. Run: git commit -m "feat: add new method to module"
6. Verify: git log -1 && git status

# Commit 2: Remove old code
1. Edit core/other/lib.rs (remove old function)
2. Run: SQLX_OFFLINE=true cargo check -p core-other
3. Run: git add core/other/lib.rs
4. Run: git commit -m "refactor: remove deprecated function"
5. Verify: git log -1 && git status

# Commit 3: Update GraphQL layer
1. Edit lana/admin-server/src/graphql/...
2. Edit apps/admin-panel/...
3. Run: SQLX_OFFLINE=true cargo check -p admin-server
4. Run: make sdl-rust && make sdl-js
5. Run: make check-code-apps (or at minimum: cd apps/admin-panel && pnpm lint && pnpm tsc-check)
6. Run: git add <all relevant files including generated>
7. Run: git commit -m "feat: update GraphQL layer"
8. Verify: git log -1 && git status
```

### NEVER Do This

- Make all changes across all files first, then try to commit in stages
- Use `git stash` or patches to retroactively split already-made changes
- Skip verification steps between commits
- Commit without checking that the code compiles

### Why This Matters

1. **Each commit must compile independently** - Many files are generated programmatically (e.g., GraphQL schema, SQLx cache, Apollo codegen). If checks aren't run before each commit, these generated files won't get updated. This breaks the build because these files serve as a boundary between backend and frontend, and CI ensures they stay in sync with the codebase.
2. **Easier to debug** - If a check fails, you know exactly which change caused it
3. **Cleaner git history** - Each commit is atomic and self-contained
4. **Easier code review** - Reviewers can understand changes incrementally

### Recovery If You Made All Changes First

If you've already made all changes and need to commit in stages:

1. Create patches for each logical group:
   ```bash
   git diff <files-for-commit-1> > /tmp/commit1.patch
   git diff <files-for-commit-2> > /tmp/commit2.patch
   ```

2. Reset to clean state:
   ```bash
   git checkout -- .
   ```

3. Apply and verify each patch in order:
   ```bash
   git apply /tmp/commit1.patch
   # Run checks
   git add <files> && git commit -m "..."

   git apply /tmp/commit2.patch
   # Run checks
   git add <files> && git commit -m "..."
   ```

This is a fallback - the correct approach is to not make all changes first.
