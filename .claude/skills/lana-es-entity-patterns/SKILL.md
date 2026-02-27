---
name: lana-es-entity-patterns
description: Reference guide for es-entity patterns including Idempotent mutations, idempotency_guard! macro, infallible queries, and caller patterns. Use when writing or modifying entity code.
---

# Entity Patterns (es-entity)

## Idempotent Entity Mutations (lint: `entity-mutate-idempotent`)

All public `&mut self` methods on `#[derive(EsEntity)]` structs must return `Idempotent<T>` or `Result<Idempotent<T>, E>`. This is enforced by the `entity-mutate-idempotent` custom lint. Private methods (`fn`, no `pub`) are exempt.

**`Idempotent<T>` enum** (from `es_entity`):
- `Idempotent::Executed(T)` — mutation was applied, caller should persist
- `Idempotent::AlreadyApplied` — exact same operation was already performed, caller can skip DB writes
- Methods: `did_execute() -> bool`, `was_already_applied() -> bool`, `unwrap() -> T`, `expect(msg) -> T`
- Note: there is no `map()` method — use `match` to transform the inner value

**Use `idempotency_guard!` macro** to detect replays by checking event history:
```rust
pub fn confirm(&mut self) -> Result<Idempotent<TxId>, MyError> {
    idempotency_guard!(
        self.events.iter_all().rev(),
        MyEvent::Confirmed { .. }
    );
    // precondition checks come AFTER the guard
    if !self.is_approved() { return Err(MyError::NotApproved); }
    // ... push event and return Idempotent::Executed(...)
}
```
- Place the guard at the **top** of the method, before any precondition error checks
- Use `.rev()` when the matching event can only appear once (efficiency)
- Prefer `idempotency_guard!` over manual `if self.is_*() { return Ok(Idempotent::AlreadyApplied) }` — the macro checks events directly and is less error-prone

**Don't mask errors as `AlreadyApplied`:**
- `AlreadyApplied` means "this exact operation was already performed successfully"
- If a condition like "no next period" or "cancelled" represents a **bug or invalid state**, return an error, not `AlreadyApplied`
- Example: `NoNextAccrualPeriod` is an error (something went wrong), not idempotency

**Separate find (query) from create (mutation):**
- Query methods: `&self`, return `Option<T>` (not `Result` for "not found")
- Mutation methods: `&mut self`, return `Result<Idempotent<T>, E>`
- Callers use find-then-create pattern:
```rust
if let Some(id) = entity.find_account(&key) {
    return Ok(id);
}
let data = entity.create_account(&key)?.expect("create executes when find returned None");
// ... persist ...
```

**Caller patterns** — use `did_execute()` to skip DB round-trips:
```rust
let result = entity.some_mutation(data)?;
if result.did_execute() {
    repo.update(&mut entity).await?;
}
```

## Transaction Types: DbOp vs DbOpWithTime

- `DbOp<'c>` — base transaction wrapper. Returned by `repo.begin_op()` and `current_job.begin_op()`.
- `DbOpWithTime<'c>` — wraps `DbOp` with guaranteed cached time. Created via `db_op.with_db_time()` which **consumes** the `DbOp` (no way to extract it back).
- `OpWithTime<'a, Op>` — **borrows** an `&mut Op` instead of consuming. Use `OpWithTime::cached_or_db_time(&mut op)`.
- `JobCompletion::CompleteWithOp` only accepts `DbOp<'static>`. If `_in_op` methods need `DbOpWithTime`, the runner must commit manually and return `Complete` instead.
- Many ledger `_in_op` methods (e.g., `settle_disbursal_in_op`) take `&mut DbOpWithTime<'_>` specifically — check the signature before assuming `DbOp` will work.

## retry_on_concurrent_modification

- `#[es_entity::retry_on_concurrent_modification]` wraps the function in a retry loop, re-calling on version conflicts.
- **Incompatible with `&mut` parameters**: the macro re-invokes the inner function on each retry, but a mutable reference can't be passed multiple times. Only use with owned/`Copy` parameters.
- `RetryableInto<T>` is used instead of `Into<T>` so the macro can re-pass the value on retry.
- When a method is only called from a job runner, the retry attribute is unnecessary — the job framework retries the entire job on failure (default: up to 30 attempts with exponential backoff).

## Infallible Entity Queries (lint: `entity-query-infallible`)

All public `&self` methods on `#[derive(EsEntity)]` structs must NOT return `Result`. This is enforced by the `entity-query-infallible` custom lint. Private methods (`fn`, no `pub`) are exempt.

- Queries return direct values or `Option<T>`, never `Result`
- If validation is needed, move it to the caller (use-case layer) or a constructor/wrapper type
- Example: instead of `fn check(&self) -> Result<Data, Error>`, use `fn find(&self) -> Option<Data>` and let the caller construct the appropriate error
