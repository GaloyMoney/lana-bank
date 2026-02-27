# Merge Exploration: `dev/lanacli` into `lana/cli` (server crate)

## Goal
Evaluate what it would take to merge the new admin CLI (`dev/lanacli`) with the existing `lana/cli` crate (current server entrypoint) without regressing server behavior or developer workflows.

## Current State

### `lana/cli` (`package = "lana-cli"`)
- Primary role: run services (`admin-server`, `customer-server`) and utility commands (`build-info`, `dump-default-config`, etc.).
- Runtime expectations: long-running process, signal handling, app startup/shutdown orchestration.
- Entry path: `lana/cli/src/lib.rs::run()` + `lana/cli/src/main.rs`.

### `dev/lanacli` (`package = "lanacli"`, bin = `lanacli`)
- Primary role: admin GraphQL operations (prospect/customer/accounting/etc.), login/logout token caching.
- Runtime expectations: short-lived command execution.
- Entry path: `dev/lanacli/src/main.rs`.
- Bats now depends on this CLI behavior for admin operations.

## Merge Options

### Option A: Single package (`lana-cli`) with top-level business commands (recommended)
Make `lana-cli` a multi-role binary:
- `lana-cli serve` (server behavior)
- `lana-cli <resource> <action> ...` for admin operations (for example `lana-cli customer create`)
- `lana-cli login/logout` (admin auth session)

Pros:
- One canonical binary and packaging surface.
- No duplicated auth/client stack.
- Matches “crate/server + admin ops in one place” intent.
- Avoids an extra namespace layer (`lana-cli admin customer create`).

Cons:
- Larger server binary + broader dependency surface.
- Need to preserve existing automation assumptions.

### Option B: Shared library + two binaries
Extract `dev/lanacli` logic into a library crate and call it from:
- `lanacli` binary (compat)
- `lana-cli <resource> <action> ...` wrapper

Pros:
- Low-risk migration with compatibility period.

Cons:
- Two binaries remain in tree for longer.

### Option C: Keep separate crates, standardize interfaces only
Minimal churn, but does not really merge crate/server ownership.

## Recommended Path (phased)

### Phase 0: Compatibility wrapper
1. Keep `dev/lanacli` as-is.
2. Add top-level admin command paths in `lana/cli` that delegate to shared admin command runner.
3. Keep `lanacli` binary as a thin compatibility wrapper calling the same runner.
4. Introduce explicit `lana-cli serve` as the server entrypoint (while preserving current behavior during migration).

### Phase 1: Internal convergence
1. Move admin CLI modules from `dev/lanacli` into `lana/cli` (or a shared internal crate).
2. Keep existing command names/flags stable.
3. Keep session storage path stable (`~/.config/lanacli/session.json`) during transition.

### Phase 2: Build and release convergence
1. Update build/release pipeline to publish a single canonical binary strategy.
2. If deprecating `lanacli`, provide alias/wrapper period and migration note.

### Phase 3: Cleanup
1. Remove duplicate crate wiring.
2. Remove compatibility wrappers once no CI/dev tooling depends on legacy entrypoint.

## Concrete File-Level Work Needed

### `lana/cli`
- `src/lib.rs`
  - Extend `Commands` with top-level admin resources (`customer`, `prospect`, `accounting`, etc.).
  - Keep server/utility commands (`serve`, `build-info`, etc.) reserved and non-conflicting.
  - Route admin commands to shared command execution path.
- `src/main.rs`
  - Keep current rustls provider install + runtime setup.
- `Cargo.toml`
  - Add admin GraphQL/auth dependencies currently in `dev/lanacli` (or shared crate dep).

### `dev/lanacli`
- Convert into either:
  - compatibility wrapper binary; or
  - shared library + wrapper binary.
- Preserve CLI interface used in Bats until transition is complete.

### CI/Tooling
- Verify Bats `LANACLI` path assumptions and decide when to point to `lana-cli`.
- Verify nix/build artifacts for expected binary naming.

## Key Risks
- Accidental server startup behavior changes (default command semantics).
- CLI name collisions between server utility commands and top-level admin resources.
- Regression in Bats/admin workflows if command flags/output format shift.
- Dependency bloat in server package and slower build times.
- Auth/session path changes causing hidden login regressions.

## Acceptance Criteria for merge project
- `lana-cli serve` behaves identically to current `lana-cli` server mode.
- `lana-cli <resource> <action> ...` reaches command parity with `lanacli ...`.
- Existing Bats suite passes with either compatibility binary or updated invocation.
- CI/release produce expected artifacts with documented migration path.

## Suggested PR breakdown
1. PR1: shared admin command runner crate/module + compatibility wrapper.
2. PR2: add top-level `lana-cli <resource> <action>` command surface, keep `lanacli` wrapper.
3. PR3: migrate CI/docs/tooling to canonical invocation.
4. PR4: deprecate/remove legacy wrapper (optional, after soak period).

## Scope of this exploration PR
- No functional merge performed yet.
- This document captures implementation plan, risk map, and sequencing for a safe merge.
