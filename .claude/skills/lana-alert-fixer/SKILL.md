---
name: lana-alert-fixer
description: Use proactively when the user asks to investigate, debug, or fix an alert. Triggers on keywords like "alert", "investigate alert", "fix alert", "silence alert", "false positive", or when the user shares error messages from an alerting system.
---

# Investigate and Fix Alert

Structured workflow for investigating an alert, determining if it's a false positive or a real bug, and producing a local commit that either silences the false positive or fixes the issue.

## Alert Context

$ARGUMENTS

## Step 1: Find the Code Path

First, understand the user-provided alert context — it may come as free-text descriptions or as structured span/trace attributes (exception messages, file paths, line numbers, module names, operation names). If the context is too vague to investigate, ask the user for more details before searching.

Using the error messages, error types, and operation names from the alert context:

1. **Find the error type**: Search for the error message text in `error.rs` files or in the code that constructs the error. Identify the error enum and specific variant producing this error.
2. **Find the `ErrorSeverity` impl**: Locate the `impl ErrorSeverity for XxxError` block in the relevant `error.rs`. Check what severity level the current variant maps to — it should be `Level::ERROR` if it's triggering alerts.
3. **Find the call site**: Search for `#[instrument]` annotations, function names matching the operation, and use cases that return this error. Read the handler or use case that triggers this error path.
4. **Understand error handling**: Determine whether the error is caught and handled gracefully (returns a user-friendly response, no data loss) or propagates unhandled.

Key files to check:
- `core/<module>/src/<submodule>/error.rs` — error enum + `ErrorSeverity` impl
- `core/<module>/src/<submodule>/mod.rs` — use cases that call the failing operation
- `lib/tracing-utils/src/error_severity.rs` — the `ErrorSeverity` trait definition

## Step 2: Classify the Error

Based on the investigation, classify as:

### False Positive
The error is expected and handled gracefully. Indicators:
- Business precondition violation (e.g., "facility not active", "month not closed", "insufficient collateral")
- Expected external condition (e.g., third-party API returns 404 for a not-yet-created resource)
- State transition guard (e.g., "approval already in progress", "already denied")
- Concurrent modification that gets retried automatically
- The caller catches the error, returns an appropriate response, and no data is lost

### Real Problem
The error indicates something is actually broken. Indicators:
- Database/infrastructure failure that shouldn't happen
- Unhandled edge case or logic bug
- Data inconsistency or corruption
- The error propagates without graceful handling

Present your classification and rationale to the user before applying any fix.

## Step 3: Apply the Fix

### For False Positives: Lower the Severity

Change the error variant's severity from `Level::ERROR` to `Level::WARN` or `Level::INFO` in the `impl ErrorSeverity` match arm.

**Simple case** — the entire variant is a false positive:
```rust
// Before
Self::FacilityNotActive => Level::ERROR,
// After
Self::FacilityNotActive => Level::WARN,
```

**Conditional case** — only a specific sub-error is a false positive:
```rust
// Before
Self::ExternalApi(e) => Level::ERROR,
// After
Self::ExternalApi(ApiError { code: 404, .. }) => Level::WARN,
Self::ExternalApi(_) => Level::ERROR,
```

**Nested delegation** — if the error wraps another error that has its own `ErrorSeverity`:
```rust
Self::WrappedError(e) => e.severity(),
```

### For Real Problems: Fix the Bug

Fix the underlying issue in the application code. The fix depends on the specific problem — it could be a logic fix, a missing state check, an error handling improvement, etc.

## Step 4: Commit

Create a local commit with a conventional commit message that explains:
- **What alert** was firing
- **Why** the change was made (false positive rationale or bug description)

Examples:
```
fix: downgrade FacilityNotActive severity for business precondition errors

The alert fires when a user attempts an operation on an inactive facility.
This is a normal business precondition — the API returns an appropriate error
response and no data is affected. Lowering from ERROR to WARN to stop
false-positive alerts.
```

```
fix: handle race condition in deposit sync that caused duplicate entries

The alert fires due to an unhandled concurrent modification error when two
sync operations run simultaneously. Added idempotency check to prevent
duplicate deposit records.
```

Do NOT push or create a PR — only commit locally.

## Constraints

- **Never modify the alerting system or alerting trigger.** The fix is always in the application code — either lowering a severity level or fixing a bug. The alerting trigger (`error.level=ERROR`) must not be changed.
- **Only lower severity for genuinely false-positive errors.** The investigation in Steps 1-2 must confirm the error is handled gracefully before downgrading. Never blindly silence alerts.
- **Commit locally only.** Do not push, create PRs, or trigger CI.
