# Instrument Name Mismatches Report

This report identifies places where the `#[instrument(name="...")]` attribute doesn't match the function name that follows it.

## Found Mismatches

### 1. lib/authz/src/lib.rs

**Line 268:** 
```rust
#[instrument(name = "authz.inspect_permission", skip(self))]
async fn evaluate_permission(
```
**Issue:** Instrument name is `inspect_permission` but function name is `evaluate_permission`

---

### 2. lib/job/src/executor.rs

**Line 261:**
```rust
#[instrument(name = "job.execute", skip_all, ...)]
async fn execute_job(
```
**Issue:** Instrument name is `execute` but function name is `execute_job`

---

### 3. lib/job/src/lib.rs

**Line 87:**
```rust
#[instrument(name = "lana.jobs.create_and_spawn", skip(self, db, config))]
pub async fn create_and_spawn_in_op<C: JobConfig>(
```
**Issue:** Instrument name is `create_and_spawn` but function name is `create_and_spawn_in_op`

**Line 107:**
```rust
#[instrument(name = "lana.jobs.create_and_spawn_at", skip(self, db, config))]
pub async fn create_and_spawn_at_in_op<C: JobConfig>(
```
**Issue:** Instrument name is `create_and_spawn_at` but function name is `create_and_spawn_at_in_op`

---

### 4. core/custody/src/lib.rs

**Line 237:**
```rust
#[instrument(name = "core_custody.find_all_custodians", skip(self), err)]
pub async fn find_all_wallets<T: From<Wallet>>(
```
**Issue:** Instrument name is `find_all_custodians` but function name is `find_all_wallets`

---

### 5. core/accounting/src/lib.rs

**Line 163:**
```rust
#[instrument(name = "core_accounting.find_ledger_account_by_code", skip(self), err)]
pub async fn find_ledger_account_by_id(
```
**Issue:** Instrument name is `find_ledger_account_by_code` but function name is `find_ledger_account_by_id`

---

### 6. core/governance/src/lib.rs

**Line 544:**
```rust
#[instrument(name = "governance.find_all_committees", skip(self), err)]
pub async fn find_all_approval_processes<T: From<ApprovalProcess>>(
```
**Issue:** Instrument name is `find_all_committees` but function name is `find_all_approval_processes`

---

## Summary

Total mismatches found: **6**

These mismatches can cause confusion in tracing and observability as the instrument names don't accurately reflect the actual function being called. It's recommended to update the instrument names to match their corresponding function names for better consistency and debugging experience.

## Recommendation

Update each instrument name to match its corresponding function name:

1. `authz.inspect_permission` → `authz.evaluate_permission`
2. `job.execute` → `job.execute_job`
3. `lana.jobs.create_and_spawn` → `lana.jobs.create_and_spawn_in_op`
4. `lana.jobs.create_and_spawn_at` → `lana.jobs.create_and_spawn_at_in_op`
5. `core_custody.find_all_custodians` → `core_custody.find_all_wallets`
6. `core_accounting.find_ledger_account_by_code` → `core_accounting.find_ledger_account_by_id`
7. `governance.find_all_committees` → `governance.find_all_approval_processes`