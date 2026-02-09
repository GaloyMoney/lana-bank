---
name: lana-monitor-pr-actions
description: Monitor PR CI checks, retry flaky failures, and fix real failures to get all checks passing.
---

# Monitor PR CI Actions

Your goal is to get all checks passing by retrying flake failures and fixing real failures.

## Workflow

1. Check the PR's CI status
2. If all checks pass, you're done
3. If checks are still running, poll until they complete
4. If any checks fail, retrieve the logs and determine whether it's a flake or a real failure
   - For flakes: retry the failed jobs, then continue monitoring
   - For real failures: implement a minimal fix, commit and push it, then continue monitoring

## Guidelines

- Make minimal, focused fixes — don't refactor unrelated code
- If a failure requires significant rework, stop and report this rather than attempting it
- Each fix should be a new commit with a conventional commit message
- Never rewrite git history or force push
- Never switch branches, create additional PRs, or modify other branches
- If pushing fails due to conflicts, stop and report — do not attempt to resolve them
- If you've attempted several fixes without success, stop and report what's blocking progress
