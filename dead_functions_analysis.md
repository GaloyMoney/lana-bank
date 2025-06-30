# Dead Functions Analysis - Rust Codebase

This report identifies potential dead functions (unused code) in the Rust banking application codebase.

## Summary

I analyzed the Rust codebase and found several categories of potentially dead functions:

1. **Test utility functions not properly marked**
2. **Debug/development functions**
3. **Commented-out or abandoned functionality**
4. **Internal helper functions without callers**

## Confirmed Dead Functions

### 1. Test Helper Function
**File:** `lana/app/tests/sumsub.rs`
- `async fn _visit_permalink(url: &str) -> anyhow::Result<()>` (line 25)
  - **Issue:** Function is defined but never called
  - **Evidence:** There's commented-out code that might have used this function, but it's not currently in use
  - **Recommendation:** Remove or properly integrate into tests

## Test Utility Functions (Potentially Dead)

These functions are only used within test modules and might be redundant:

### 2. Dummy Audit Info Functions
Multiple `dummy_audit_info()` functions across the codebase that create placeholder audit information for tests:

**Files:**
- `core/governance/src/approval_process/entity.rs` (line 297)
- `core/governance/src/policy/repo.rs` (line 39)  
- `core/governance/src/policy/entity.rs` (line 164)
- `core/credit/src/interest_accrual_cycle/entity.rs` (line 405)
- `core/credit/src/credit_facility/entity.rs` (line 738)
- `core/credit/src/obligation/entity.rs` (line 688)
- `core/deposit/src/deposit/entity.rs` (line 129)
- `core/accounting/src/chart_of_accounts/tree.rs` (line 145)
- `core/accounting/src/chart_of_accounts/entity.rs` (line 358)
- `core/deposit/src/withdrawal/entity.rs` (line 262)
- `lib/authz/src/dummy.rs` (line 124)

**Analysis:** These functions create identical audit info structures and could be consolidated into a shared test utility.

## Functions with Unclear Usage

### 3. Private Helper Functions
These private functions might be dead but require deeper analysis:

**File:** `core/governance/src/policy/rules.rs`
- `fn make_set(ids: &[u32]) -> HashSet<u32>` (line 55)
  - **Status:** Used within the same file's tests, so NOT dead

## Development/Debug Functions

### 4. Test Infrastructure
**File:** `core/governance/src/policy/repo.rs`
- `pub async fn init_pool() -> anyhow::Result<sqlx::PgPool>` (line 46)
  - **Usage:** Appears to be test infrastructure that might be unused

## Functions Marked as Potentially Unused

Several functions are explicitly marked with `#[allow(dead_code)]` or similar attributes, indicating the developers are aware they might be unused but keeping them intentionally.

## Recommendations

### Immediate Actions:
1. **Remove confirmed dead function:** `_visit_permalink` in `lana/app/tests/sumsub.rs`
2. **Consolidate test utilities:** Create a shared test utility module for `dummy_audit_info()` functions
3. **Review test infrastructure:** Verify if functions like `init_pool()` are still needed

### Further Investigation Needed:
1. **Static analysis:** Run `cargo +nightly rustc -- -W dead_code` once database dependencies are resolved
2. **Usage analysis:** Search for dynamic/macro-generated calls to functions that might not show up in static analysis
3. **Integration testing:** Verify which functions are used in integration tests vs unit tests

## Notes on Analysis Limitations

- **Compilation Issues:** The codebase has database connectivity requirements that prevented full cargo analysis
- **Macro Usage:** Some functions might be called through macros or code generation that wouldn't show up in text searches
- **Test Functions:** Many functions marked as potentially dead are actually legitimate test functions
- **Public API:** Some unused public functions might be intentionally kept as part of the crate's API

## Estimated Impact

- **Confirmed dead code:** ~50 lines (1 function)
- **Potential consolidation opportunity:** ~200 lines (duplicate test utilities)
- **Low risk cleanup:** Removing confirmed dead code should have no impact on functionality