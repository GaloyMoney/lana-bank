---
name: ci-monitor
description: Monitor PR CI checks, retry flaky failures, and fix real failures to get all checks passing.
---

# CI Monitor

Get all CI checks passing for the current PR. Monitor check status, retry flaky failures, and apply minimal fixes for real failures.

## Workflow

Follow this loop until all checks pass or a stop condition is reached.

### 1. Gather Context

```bash
git branch --show-current
gh pr view --json number,title,url,headRefName,baseRefName
```

Confirm you are on the correct branch and a PR exists before proceeding.

### 2. Check CI Status

```bash
gh pr checks
```

Evaluate the output:

- **All checks pass** → Report success and stop.
- **Checks still running** → Poll until complete:
  ```bash
  gh pr checks --watch --fail-fast
  ```
  Use a 30-minute timeout. If polling times out, report the status and stop.
- **Any checks failed** → Proceed to failure analysis.

### 3. Analyze and Act on Failures

For each failed check:

1. Identify the workflow run ID from `gh pr checks` output.
2. Retrieve failed job logs:
   ```bash
   gh run view <run-id> --log-failed
   ```
3. Classify the failure as **flaky** or **real** (see sections below).
4. Take action:
   - **Flaky**: Rerun failed jobs, then return to step 2.
     ```bash
     gh run rerun <run-id> --failed
     ```
   - **Real failure**: Fix it (see Fixing Real Failures), then return to step 2.

Repeat until all checks pass or a stop condition is hit.

## Common Flaky Patterns

Retry these without attempting a code fix:

- **External service timeouts** — Sumsub integration calls timing out
- **Container startup race conditions** — Docker/Podman failures in BATS or Cypress tests (e.g., services not ready, port conflicts)
- **Intermittent network issues** — Dependency resolution failures, registry timeouts

When in doubt, check if the same test passes on `main` or in recent PRs. If it does, treat it as a flake.

## Fixing Real Failures

Apply the minimal fix for each failure type. Each fix must be a separate commit with a conventional commit message.

### Cocogitto (`cocogitto.yml`)

Commit message does not follow conventional commit format.

- Read the Cocogitto output to find the offending commit.
- Amend the commit message to follow conventional commits. **This is the one exception** to the no-amend rule.
  ```bash
  git commit --amend -m "fix: corrected commit message"
  ```
- Push the amended branch (non-force push if possible; force push only for this specific case).

### Spelling (`spelling.yml`)

Typo detected by `typos` (configured in `typos.toml`).

- Read the logs to find the flagged word and file.
- Either correct the typo in the source file, or add the word to `typos.toml` if it is a valid domain term.
- Commit and push.

### Check Code Apps (`check-code-apps.yml`)

TypeScript, lint, or build errors in `apps/`.

- Reproduce locally:
  ```bash
  make check-code-apps
  ```
- Fix the reported errors in the relevant files under `apps/`.
- Commit and push.

### Nextest (`nextest.yml`)

Rust compilation or test assertion failure.

- Identify the failing crate and test from the logs.
- Reproduce locally:
  ```bash
  SQLX_OFFLINE=true cargo nextest run -p <crate>
  ```
- Fix the code issue. If it is a test assertion, verify the expected behavior is correct before changing the assertion.
- Commit and push.

### SQLx Offline Cache Mismatch

Compilation or test failures mentioning SQLx prepared statements being out of date.

- Regenerate the cache:
  ```bash
  make sqlx-prepare
  ```
- Commit the `.sqlx/` changes and push.

### GraphQL Schema Drift

Schema mismatch between Rust resolvers and `schema.graphql`.

- Regenerate the schema:
  ```bash
  make sdl
  ```
- Commit the schema changes and push.

### Other Checks

For BATS, Cypress, CodeQL, Data Pipeline, Flake Check, or pnpm Audit failures:

1. Read the full failed job logs with `gh run view <run-id> --log-failed`.
2. Identify the root cause from the log output.
3. If the fix is small and obvious, apply it, commit, and push.
4. If the fix is non-trivial or unclear, **stop and report** — do not guess.

## Safety Rules

These rules are non-negotiable:

- **Never force push or rewrite history** — except for Cocogitto commit message amends.
- **Never switch branches** or create new branches or PRs.
- **Never modify other branches** — only work on the current PR branch.
- **Never refactor unrelated code** — fixes must be scoped to the failing check.
- **Each fix must be a separate commit** with a conventional commit message (e.g., `fix: ...`, `chore: ...`).
- **If `git push` fails due to conflicts** → Stop and report. Do not attempt to resolve merge conflicts.
- **If 3 fix attempts fail for the same check** → Stop and report what is blocking progress.
- **If a failure requires significant rework** → Stop and report instead of attempting a large change.
- **If the failure is in code you do not understand** → Stop and report rather than guessing at a fix.

## Local Verification Commands

Run these before pushing a fix to catch issues early:

| Command | Purpose |
|---------|---------|
| `make check-code-rust` | Rust fmt, check, clippy, audit, deny |
| `make check-code-apps` | Frontend lint, type check, build |
| `SQLX_OFFLINE=true cargo nextest run -p <crate>` | Run specific crate tests |
| `make sqlx-prepare` | Update SQLx offline cache |
| `make sdl` | Regenerate GraphQL schema |

## CI Checks Reference

| Check Name | Workflow File | What It Validates |
|------------|---------------|-------------------|
| Nextest | `nextest.yml` | Rust unit tests |
| BATS | `bats.yml` | E2E integration tests |
| Check Code Apps | `check-code-apps.yml` | Frontend lint, type check, build |
| Cypress | `cypress.yml` | E2E UI tests |
| Data Pipeline | `data-pipeline.yml` | Dagster/dbt pipelines |
| CodeQL | `codeql.yml` | Security scanning |
| Cocogitto | `cocogitto.yml` | Conventional commit validation |
| Flake Check | `flake-check.yml` | Nix flake evaluation |
| pnpm Audit | `pnpm-audit.yml` | JS dependency security |
| Spelling | `spelling.yml` | Typo detection via `typos` |
