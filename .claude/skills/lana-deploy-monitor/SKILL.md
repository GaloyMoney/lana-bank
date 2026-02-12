---
name: lana-deploy-monitor
description: Monitor a commit's journey from main through the Concourse CI pipeline to staging deployment. Tracks lana-bank tests, testflight, and staging jobs.
---

# Monitor Deployment to Staging

Track a commit from `main` through the full Concourse CI/CD pipeline until it reaches staging.

## Pipeline Stages

The deployment flows through three Concourse pipelines in order:

1. **lana-bank pipeline** — runs tests and builds images
   - URL: https://ci.galoy.io/teams/dev/pipelines/lana-bank
2. **private-charts pipeline** — deploys to a testflight environment (`lana-bank-testflight` job)
   - URL: https://ci.galoy.io/teams/dev/pipelines/private-charts/jobs/lana-bank-testflight
3. **galoy-staging pipeline** — deploys to staging (`galoy-staging-lana-bank` job)
   - URL: https://ci.galoy.io/teams/dev/pipelines/galoy-staging/jobs/galoy-staging-lana-bank

## Resource Path Filters

Not every commit triggers every job. The lana-bank pipeline has multiple git resources with different path filters:

| Resource | Watches | Ignores |
|----------|---------|---------|
| `repo` | everything | `apps/**/*`, `ci/*` (except .md/.x/.k) |
| `admin-panel-src` | `apps/admin-panel/**`, `flake.nix`, `flake.lock`, `pnpm-lock.yaml` | — |
| `customer-portal-src` | `apps/customer-portal/**`, `flake.nix`, `flake.lock`, `pnpm-lock.yaml` | — |
| `dagster-src` | `dagster/*` | — |

## Job Dependency Chains

**Full pipeline (Rust / backend changes)** — triggered when `repo` gets a new version:
```
repo → test-integration + test-bats + flake-check (parallel)
    → build-rc (after all tests pass + frontend/dagster edge images built)
    → open-promote-rc-pr
    → private-charts/lana-bank-testflight
    → galoy-staging/galoy-staging-lana-bank
```

Note: The `release` and `bump-image-in-chart` jobs exist for tagging formal releases (RC → tagged release). They are **not** part of the deploy-to-staging path and should not be monitored by this skill.

**Frontend-only changes** (only `apps/admin-panel` or `apps/customer-portal`):
- Triggers `build-admin-panel-edge-image` and/or `build-customer-portal-edge-image`
- Does **NOT** trigger the `repo` resource (ignored paths), so tests and the full release chain do not run
- The new frontend images will be picked up on the **next** Rust-inclusive release that triggers `build-rc` → `bump-image-in-chart`

**Dagster-only changes** (only `dagster/`):
- Triggers `build-dagster-edge-image`
- `dagster-edge-image` is a `trigger: true` input in `build-rc`, so this **does** trigger the full pipeline

## Concourse Access

- Target: `galoy`
- Team: `dev`
- If any `fly` command returns an auth error, ask the operator to run `fly -t galoy login` to refresh credentials.

## Workflow

### Step 1: Identify the commit to track

Resolve the target commit using one of these approaches, in priority order:

1. **`$ARGUMENTS` contains a commit SHA, PR number, or PR URL** — resolve it to a commit on `main`.
   - For an already-merged PR: use `gh pr view <number> --json mergeCommit` to get the resulting commit SHA.
   - For an unmerged PR: see the "merge current PR" flow below.
2. **`$ARGUMENTS` is empty and the current branch has an open PR** — detect with `gh pr view --json number,state,reviewDecision,mergeCommit`. If an open PR exists, use the "merge current PR" flow below.
3. **`$ARGUMENTS` is empty and no open PR** — use the latest commit on `main`: `git log origin/main -1 --format='%H'` (fetch first with `git fetch origin main`).

**Merge current PR flow:**
When the target is an open (unmerged) PR:
1. Check approval status with `gh pr view --json reviewDecision`.
2. If `reviewDecision` is **not** `APPROVED`, stop and tell the user the PR is not yet approved — do not merge.
3. If approved, merge with `gh pr merge --squash`.
4. After merge completes, fetch main and resolve the resulting commit SHA with `gh pr view --json mergeCommit`.
5. Continue to Step 2 with the resulting commit SHA.

Display the commit SHA and its message to the user.

### Step 2: Classify the commit and determine pipeline path

Check what files changed in the commit using `git diff-tree --no-commit-id --name-only -r <sha>`.

Classify into one of:
- **Backend/Rust** — files outside `apps/` and `dagster/` changed → full pipeline via `repo`
- **Dagster** — only `dagster/*` changed → full pipeline via `dagster-src` → `build-dagster-edge-image` → `build-rc`
- **Frontend-only** — only `apps/admin-panel/**` or `apps/customer-portal/**` changed → limited pipeline
- **Mixed** — combination of the above → full pipeline (the `repo` resource will trigger)

If the commit is **frontend-only**:
1. Inform the user: "This commit only changes frontend code. It will build new edge images but will NOT trigger the full release pipeline. The images will be deployed to staging on the next backend-inclusive release."
2. Monitor `build-admin-panel-edge-image` and/or `build-customer-portal-edge-image` as applicable.
3. Check the relevant resource: `fly -t galoy resource-versions -r lana-bank/admin-panel-src -c 10` (or `customer-portal-src`).
4. Once the edge image build(s) complete, report success and stop — there is nothing more to track.

For all other cases, proceed with the full pipeline below.

### Step 3: Monitor lana-bank pipeline (full pipeline path)

Check whether the commit has been picked up by the `repo` resource:
`fly -t galoy resource-versions -r lana-bank/repo -c 10`
If not yet visible, poll every 30 seconds until it appears.

Once the commit is a known resource version, monitor these jobs in order:
1. **Tests (parallel):** `test-integration`, `test-bats`, `flake-check`
2. **Build:** `build-rc` (waits for tests + edge images)
3. **Promote:** `open-promote-rc-pr`

For each job, use `fly -t galoy builds -j lana-bank/<job> -c 5 --json` to get recent builds, then use the Concourse API to check which commit each build used:
```
fly -t galoy curl -- "/api/v1/builds/<build_id>/resources"
```
**Note:** `fly curl` outputs curl progress lines before JSON. Parse with `grep -o '{.*}' | tail -1` to extract the JSON body.

Look for the `repo` input whose `version.ref` matches the target commit.

Poll running builds every 30 seconds. Report the status of each job as it completes.

If any job fails, retrieve the logs with `fly -t galoy watch -j lana-bank/<job> -b <build_name>` (use `head` to limit output) and report the failure. Ask the user if they want to continue monitoring or stop.

### Step 4: Monitor testflight

Once `open-promote-rc-pr` succeeds, the private-charts pipeline should pick up the new chart version.

Monitor `fly -t galoy builds -j private-charts/lana-bank-testflight -c 5 --json` for a new build that started after the `open-promote-rc-pr` build completed.

Poll every 30 seconds until the testflight build completes. Report success or failure.

On failure, retrieve logs with `fly -t galoy watch -j private-charts/lana-bank-testflight -b <build_name>` and report.

### Step 5: Monitor staging deployment

Once testflight succeeds, monitor the staging deployment.

Use `fly -t galoy builds -j galoy-staging/galoy-staging-lana-bank -c 5 --json` for a new build that started after testflight completed.

To verify this staging build corresponds to our commit, check its resources:
```
fly -t galoy curl -- "/api/v1/builds/<build_id>/resources"
```
The `lana-repo` input's `version.ref` should match the target commit (or a descendant if more commits were merged).

Poll every 30 seconds until the staging build completes.

### Step 6: Report result

When the staging job succeeds, report:
```
Deployment complete! Commit <short-sha> (<message>) is now deployed to staging.

Timeline:
- lana-bank tests: <duration>
- build-rc: <duration>
- open-promote-rc-pr: <duration>
- testflight: <duration>
- staging: <duration>
- Total: <total duration>
```

If any stage fails, report which stage failed and include relevant log excerpts.

## Guidelines

- Always show the user which stage is currently active and what you're waiting for.
- Use concise status updates — don't flood with output.
- When polling, use `sleep 30` between checks.
- If a stage has been running for an unusually long time (>30 min for tests, >20 min for testflight, >25 min for staging), warn the user.
- If the commit hasn't appeared in the pipeline after 5 minutes, warn that it may not have triggered the pipeline (e.g., changes only in ignored paths).
