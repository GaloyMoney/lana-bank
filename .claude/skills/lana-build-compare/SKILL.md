---
name: lana-build-compare
description: Compare cargo build compilation times between current branch and main using cargo --timings.
---

# Compare Build Times

Compare full clean-build compilation times between the current branch and a base branch (default: main), identifying changes in build duration, parallelism, and per-crate timings.

## Arguments

$ARGUMENTS

If arguments are provided, use the first argument as the base branch name instead of `main`.

## Workflow

Builds are done **serially** (one after the other, never in parallel) to get accurate, comparable timings on the same machine under similar load.

### Step 1: Identify branches

1. Get current branch: `git branch --show-current`
2. Determine base branch: use `$ARGUMENTS` if provided, otherwise `main`
3. Display both branch names to the user

### Step 2: Build the base branch (in a worktree)

1. Fetch the base branch: `git fetch origin <base-branch>`
2. Create a temporary git worktree:
   ```
   git worktree add /tmp/lana-build-compare-base origin/<base-branch>
   ```
3. In the worktree directory, run a clean build:
   ```
   cd /tmp/lana-build-compare-base && cargo clean && rm -rf target/ && SQLX_OFFLINE=true cargo build --timings
   ```
4. Copy the timing report:
   ```
   cp /tmp/lana-build-compare-base/target/cargo-timings/cargo-timing.html /tmp/cargo-timing-base.html
   ```
5. Clean up the worktree:
   ```
   git worktree remove /tmp/lana-build-compare-base --force
   ```

If the build fails, report the error and stop.

### Step 3: Build the current branch

1. In the current working directory, run a clean build:
   ```
   cargo clean && rm -rf target/ && SQLX_OFFLINE=true cargo build --timings
   ```
2. Copy the timing report:
   ```
   cp target/cargo-timings/cargo-timing.html /tmp/cargo-timing-current.html
   ```

If the build fails, report the error and stop.

### Step 4: Extract metrics from HTML reports

Read both HTML files and extract:
- **Total wall clock time** (look for "Finished" line or the max timestamp)
- **Total CPU time**
- **Effective parallelism** (CPU time / wall time)
- **Total units compiled**
- **Fresh units** (should be 0 for clean builds — verify this)
- **Dirty units**
- **Top 15 slowest crates** with their build times

The timing data is embedded in the HTML. Look for the `<table>` with unit timings and the summary statistics.

### Step 5: Present comparison

#### Overall Summary

```
| Metric              | main       | <current-branch> | Delta     |
|---------------------|------------|-------------------|-----------|
| Wall clock time     | Xs         | Ys                | +/-Zs     |
| CPU time            | Xs         | Ys                | +/-Zs     |
| Parallelism         | X.Xx       | Y.Yx              | +/-Z.Zx   |
| Total crates        | N          | M                 | +/-K      |
| Fresh (should be 0) | 0          | 0                 |           |
```

#### Top 15 Slowest Crates

```
| Crate               | main (s) | current (s) | Delta (s) |
|----------------------|----------|-------------|-----------|
| crate-name           | X.X      | Y.Y         | +/-Z.Z    |
| ...                  |          |             |           |
```

#### New/Removed Crates

List any crates that appear in one build but not the other.

#### HTML Reports

```
Base branch report:    /tmp/cargo-timing-base.html
Current branch report: /tmp/cargo-timing-current.html
```

Tell the user they can open these in a browser for the full interactive view.

## Guidelines

- Always do clean builds (`cargo clean` + `rm -rf target/`) to ensure 0 fresh units
- Use git worktree to avoid disturbing the current working tree
- Clean up the worktree after the base branch build completes (success or failure)
- Use `SQLX_OFFLINE=true` for all cargo commands
- If a build fails, report the error and stop — do not continue to comparison
- Builds MUST be serial to avoid CPU/memory contention skewing results
