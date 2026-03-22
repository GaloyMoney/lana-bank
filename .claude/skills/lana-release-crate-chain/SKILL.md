---
name: lana-release-crate-chain
description: Roll out a new version of a crate (typically es-entity) through the full dependency chain — es-entity, job, obix, cala — then update lana-bank. Handles PRs, CI, Concourse releases, and crates.io publishing for each step.
---

# Release Crate Chain

Roll out a crate update through the dependency chain: **es-entity -> job -> obix -> cala -> lana-bank**.

Each crate in the chain must be released sequentially because downstream crates depend on the upstream ones being published to crates.io first.

## Invocation

This skill is invoked explicitly via `/lana-release-crate-chain`. It is never triggered automatically.

The user may pass arguments after the slash command:
- `/lana-release-crate-chain` — start from es-entity, roll through the full chain
- `/lana-release-crate-chain start from obix` — skip es-entity and job, start at obix
- `/lana-release-crate-chain up to cala` — stop after cala, skip lana-bank
- `/lana-release-crate-chain es-entity PR #113` — start by merging a specific PR

## Repository Locations

| Crate | Path | GitHub | Concourse Pipeline |
|-------|------|--------|--------------------|
| es-entity | `/Users/n/Code/es-entity` | `GaloyMoney/es-entity` | `es-entity` |
| job | `/Users/n/Code/job` | `GaloyMoney/job` | `job` |
| obix | `/Users/n/Code/obix` | `GaloyMoney/obix` | `obix` |
| cala | `/Users/n/Code/cala` | `GaloyMoney/cala` | `cala` |
| lana-bank | `/Users/n/Code/lana-bank` | `GaloyMoney/lana-bank` | `lana-bank` |

## Dependency Chain

```
es-entity
  <- job (depends on es-entity)
    <- obix (depends on es-entity + job)
      <- cala (depends on es-entity + job + obix)
        <- lana-bank (depends on es-entity + job + obix + cala-ledger)
```

Each crate's `Cargo.toml` has workspace-level dependency declarations. Check the `[workspace.dependencies]` section for the current version pins.

## Per-Crate Release Flow

For each crate in the chain, follow these steps in order:

### Step 1: Merge the PR on GitHub

If the crate already has an open PR with the changes (e.g., a version bump PR), merge it. If not, create one:

1. `cd` to the repo directory
2. Create a branch, update `Cargo.toml` to bump the upstream dependency version
3. Run `cargo update -p <upstream-crate>` to update the lockfile
4. Commit, push, create a draft PR with `gh pr create --draft`
5. Wait for GitHub Actions CI to pass: `gh pr checks <PR> --watch --fail-fast` (use 30m timeout)
6. If checks fail, read the logs with `gh run view <run-id> --log-failed`, fix, and push again
7. Once checks pass, mark ready and merge: `gh pr ready <PR> && gh pr merge <PR> --merge`

### Step 2: Wait for Concourse release pipeline

After merging to main, the Concourse pipeline automatically runs tests and then releases:

1. **Authenticate if needed:** `fly -t galoy login` (browser-based login)
2. **Trigger the release job** (if it doesn't auto-trigger):
   ```
   fly -t galoy trigger-job -j <pipeline>/release
   ```
3. **Watch the release job:**
   ```
   fly -t galoy watch -j <pipeline>/release
   ```
   This will stream logs until the job completes. The release job:
   - Bumps the version (via semver resource)
   - Updates Cargo.toml with the release version
   - Publishes to crates.io
   - Creates a git tag and GitHub release
   - The subsequent `set-dev-version` job bumps to the next `-dev` version

4. **If the release job is not yet triggered** (waiting for `check-code` and `tests` to pass first):
   ```
   fly -t galoy watch -j <pipeline>/check-code
   fly -t galoy watch -j <pipeline>/tests
   ```
   Wait for both to pass, then watch the release job.

### Step 3: Verify crates.io publication

After the Concourse release job completes, verify the new version is on crates.io:

```bash
# Check the latest version on crates.io (may take a minute to index)
cargo search <crate-name> 2>/dev/null | head -1
```

The version should match what Concourse published. Note the exact version number for the next crate's dependency update.

### Step 4: Move to the next crate

Once the new version is confirmed on crates.io, proceed to the next crate in the chain and repeat from Step 1.

## Concourse Pipeline Details

All crate pipelines (es-entity, job, obix, cala) follow the same structure:

**Jobs:**
- `check-code` — `nix flake check` (triggered by git push to main)
- `tests` — `nix run .#nextest` (triggered by git push to main)
- `release` — Publishes to crates.io (requires check-code + tests to pass, manual trigger or auto)
- `set-dev-version` — Bumps to next `-dev` version (auto after release)

**Pipeline URLs:** `https://ci.galoy.io/teams/dev/pipelines/<pipeline-name>`

**Fly commands reference:**
```bash
# List pipelines
fly -t galoy pipelines

# Watch a job's output
fly -t galoy watch -j <pipeline>/<job>

# Trigger a job manually
fly -t galoy trigger-job -j <pipeline>/<job>

# Check job status
fly -t galoy builds -j <pipeline>/<job> -c 1

# Unpause a pipeline (if paused)
fly -t galoy unpause-pipeline -p <pipeline>
```

## Updating lana-bank (Final Step)

Lana-bank is different from the crate chain — it's not published to crates.io. Instead:

1. Create a branch in the lana-bank worktree
2. Update `Cargo.toml` workspace dependencies for all bumped crates
3. Run `cargo update` to refresh the lockfile
4. Run `SQLX_OFFLINE=true cargo check` to verify compilation
5. If GraphQL schema changes are involved, run `make sdl` to regenerate
6. Create a draft PR, wait for CI, then merge (or leave for review)

## Error Handling

- **Concourse auth expired:** Run `fly -t galoy login` and retry
- **Release job fails:** Check logs with `fly -t galoy watch -j <pipeline>/release`, diagnose, fix on main, and retrigger
- **crates.io publish fails:** Usually a version conflict or yanked version. Check the publish-to-crates task logs
- **GitHub Actions flaky:** Retry failed jobs with `gh run rerun <run-id> --failed`
- **Dependency resolution fails:** Make sure the upstream crate is actually indexed on crates.io (can take 1-2 minutes after publish)

## Progress Tracking

Use tasks to track progress through the chain. Example:

```
[ ] es-entity: merge PR -> release -> verify on crates.io
[ ] job: bump es-entity -> PR -> merge -> release -> verify
[ ] obix: bump es-entity + job -> PR -> merge -> release -> verify
[ ] cala: bump es-entity + job + obix -> PR -> merge -> release -> verify
[ ] lana-bank: bump all deps -> PR -> merge
```

Report progress to the user after each crate completes.
