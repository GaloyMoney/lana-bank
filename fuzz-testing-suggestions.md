# Fuzz Testing Strategy for Lana Bank Core Banking Application

## Overview

Fuzz testing is critical for financial applications to ensure robustness against malformed inputs, edge cases, and potential security vulnerabilities. This document outlines specific areas in the codebase where fuzz testing would be most valuable.

## Priority Areas for Fuzz Testing

### 1. **Core Money Operations** (`core/money/`)

**Why**: Financial calculations must be bulletproof against overflow, underflow, and precision errors.

**Targets**:
- `Satoshis` and `SignedSatoshis` arithmetic operations
- `UsdCents` and `SignedUsdCents` calculations
- BTC/USD conversion functions
- Money serialization/deserialization

**Implementation Location**: `core/money/fuzz/`

**Example fuzz targets**:
```rust
// fuzz/targets/satoshis_arithmetic.rs
#[macro_use] extern crate libfuzzer_sys;
use core_money::{Satoshis, SignedSatoshis};

fuzz_target!(|data: &[u8]| {
    if data.len() >= 16 {
        let a = i64::from_le_bytes(data[0..8].try_into().unwrap());
        let b = i64::from_le_bytes(data[8..16].try_into().unwrap());
        
        if let (Ok(sats_a), Ok(sats_b)) = (
            SignedSatoshis::try_from_btc(rust_decimal::Decimal::from(a)),
            SignedSatoshis::try_from_btc(rust_decimal::Decimal::from(b))
        ) {
            let _ = sats_a + sats_b;
            let _ = sats_a - sats_b;
        }
    }
});
```

### 2. **Accounting Primitives** (`core/accounting/src/primitives.rs`)

**Why**: Account codes and names are parsed from external input and used throughout the system.

**Targets**:
- `AccountName::from_str()` - Line 64
- `AccountCode::from_str()` - Line 244  
- `AccountCodeSection::from_str()` - Line 111
- `AccountIdOrCode::from_str()` - Line 289

**Implementation Location**: `core/accounting/fuzz/`

**Example fuzz targets**:
```rust
// fuzz/targets/account_parsing.rs
#[macro_use] extern crate libfuzzer_sys;
use core_accounting::primitives::{AccountName, AccountCode, AccountCodeSection};

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        let _ = s.parse::<AccountName>();
        let _ = s.parse::<AccountCode>();
        let _ = s.parse::<AccountCodeSection>();
    }
});
```

### 3. **JSON Serialization/Deserialization**

**Why**: JSON parsing is a common attack vector and critical for API security.

**Targets**:
- All `serde_json::from_str()` calls found in the codebase
- GraphQL input validation
- CSV parsing in `core/accounting/src/csv/`

**Implementation Location**: `lib/fuzz/` or individual module fuzz directories

**Example fuzz targets**:
```rust
// fuzz/targets/json_parsing.rs
#[macro_use] extern crate libfuzzer_sys;
use serde_json;

fuzz_target!(|data: &[u8]| {
    if let Ok(s) = std::str::from_utf8(data) {
        // Test various JSON structures used in the app
        let _ = serde_json::from_str::<core_money::Satoshis>(s);
        let _ = serde_json::from_str::<core_accounting::primitives::AccountCode>(s);
        // Add other commonly deserialized types
    }
});
```

### 4. **GraphQL Input Validation** (`lana/admin-server/src/graphql/`, `lana/customer-server/src/graphql/`)

**Why**: GraphQL endpoints are external-facing and handle user input.

**Targets**:
- Query and mutation input validation
- GraphQL scalar parsing
- Input sanitization

**Implementation Location**: `lana/admin-server/fuzz/`, `lana/customer-server/fuzz/`

### 5. **CSV Processing** (`core/accounting/src/csv/`)

**Why**: CSV parsing is notoriously error-prone and handles financial data.

**Targets**:
- CSV parsing logic in `core/accounting/src/csv/mod.rs`
- Data validation and transformation

**Implementation Location**: `core/accounting/fuzz/`

### 6. **External API Response Parsing**

**Why**: External data sources can send malformed responses.

**Targets**:
- BitFinex API response parsing (`core/price/src/bfx_client/`)
- SumSub webhook parsing (`lana/app/src/applicant/sumsub_auth.rs`)
- Custodian webhook parsing

**Implementation Location**: `core/price/fuzz/`, `lana/app/fuzz/`

### 7. **Authorization and Access Control** (`lib/authz/`, `core/access/`)

**Why**: Authorization bugs can lead to privilege escalation.

**Targets**:
- Permission parsing and validation
- Subject/Object/Action parsing
- RBAC rule evaluation

**Implementation Location**: `lib/authz/fuzz/`, `core/access/fuzz/`

## Implementation Strategy

### Phase 1: Set up Infrastructure

1. **Add cargo-fuzz to workspace**:
```toml
# Add to workspace Cargo.toml
[workspace.dependencies]
cargo-fuzz = "0.11"
libfuzzer-sys = "0.4"
```

2. **Create fuzz testing structure**:
```
fuzz/
├── Cargo.toml
├── targets/
│   ├── money_arithmetic.rs
│   ├── account_parsing.rs
│   ├── json_parsing.rs
│   └── csv_parsing.rs
└── corpus/
    ├── money_arithmetic/
    ├── account_parsing/
    └── json_parsing/
```

### Phase 2: Core Financial Operations (High Priority)

1. **Money arithmetic operations**
2. **Account code parsing**
3. **Financial calculation edge cases**

### Phase 3: Input Validation (Medium Priority)

1. **JSON deserialization**
2. **GraphQL input validation**
3. **CSV parsing**

### Phase 4: External Interfaces (Medium Priority)

1. **API response parsing**
2. **Webhook handling**
3. **File format parsing**

### Phase 5: Security-Critical Components (High Priority)

1. **Authorization logic**
2. **Cryptographic operations**
3. **Session handling**

## Specific Implementation Locations

### Core Modules

- `core/money/fuzz/` - Money arithmetic and conversions
- `core/accounting/fuzz/` - Account parsing and ledger operations
- `core/credit/fuzz/` - Credit facility calculations
- `core/deposit/fuzz/` - Deposit processing
- `core/governance/fuzz/` - Policy and approval logic

### Library Modules

- `lib/authz/fuzz/` - Authorization and RBAC
- `lib/audit/fuzz/` - Audit log parsing
- `lib/cloud-storage/fuzz/` - File handling

### Application Modules

- `lana/admin-server/fuzz/` - Admin GraphQL endpoints
- `lana/customer-server/fuzz/` - Customer GraphQL endpoints
- `lana/app/fuzz/` - Core business logic

## Integration with CI/CD

### Makefile Integration

Add to the existing `Makefile`:

```makefile
.PHONY: fuzz-test fuzz-money fuzz-accounting fuzz-graphql

fuzz-test: fuzz-money fuzz-accounting fuzz-graphql

fuzz-money:
	cd core/money && cargo fuzz run money_arithmetic -- -max_total_time=300

fuzz-accounting:
	cd core/accounting && cargo fuzz run account_parsing -- -max_total_time=300

fuzz-graphql:
	cd lana/admin-server && cargo fuzz run graphql_input -- -max_total_time=300

fuzz-ci:
	# Short runs for CI
	cd core/money && cargo fuzz run money_arithmetic -- -max_total_time=60
	cd core/accounting && cargo fuzz run account_parsing -- -max_total_time=60
```

### Testing Strategy

1. **Local Development**: Developers run focused fuzz tests on modules they're modifying
2. **CI Pipeline**: Short fuzz test runs (1-2 minutes) to catch obvious issues
3. **Nightly Builds**: Long-running fuzz tests (hours) to find deeper issues
4. **Release Testing**: Extended fuzz testing before major releases

## Monitoring and Corpus Management

1. **Corpus Collection**: Save interesting inputs found during fuzzing
2. **Regression Testing**: Use discovered inputs as regression tests
3. **Coverage Tracking**: Monitor code coverage from fuzz testing
4. **Issue Tracking**: Log and track fuzz-discovered issues

## Expected Benefits

1. **Security**: Find input validation vulnerabilities before attackers do
2. **Robustness**: Improve handling of edge cases and malformed input
3. **Compliance**: Meet security requirements for financial applications
4. **Confidence**: Increase confidence in code reliability under stress

## Getting Started

1. Choose one high-priority target (recommend starting with money arithmetic)
2. Set up basic fuzz testing infrastructure
3. Create initial corpus with known valid inputs
4. Run initial fuzz tests and fix any immediate issues
5. Gradually expand to other areas

## ✅ Implementation Complete: Money Arithmetic Fuzz Test

**Location**: `core/money/fuzz/`

I have successfully implemented the first fuzz test for Satoshis arithmetic operations:

### What's Implemented

- **Fuzz Target**: `core/money/fuzz/targets/satoshis_arithmetic.rs`
- **Configuration**: `core/money/fuzz/Cargo.toml`
- **Initial Corpus**: Several seed files with edge cases
- **Demo Script**: `core/money/fuzz/reproduce_crash.sh`

### Test Coverage

The fuzz test covers:
- ✅ Basic arithmetic operations (add, subtract)
- ✅ BTC conversion (to_btc, try_from_btc)
- ✅ Type conversions (Satoshis ↔ SignedSatoshis)
- ✅ Overflow detection
- ✅ Precision handling

### 🐛 **BUG FOUND!**

**The fuzz test immediately found a critical bug:**

```
thread panicked at: Satoshis must be integer sized for i64: TryFromIntError
```

**Issue**: Converting large `Satoshis` (u64) to `SignedSatoshis` (i64) panics when the value exceeds `i64::MAX` (9.2 quintillion satoshis / 92 million BTC).

**Impact**: This could crash the application when processing very large amounts.

**Fix Required**: Replace `expect()` with proper error handling using `TryFrom` trait.

### Running the Fuzz Test

```bash
# Navigate to the fuzz directory
cd core/money/fuzz

# Run a quick test (10 seconds)
cargo +nightly fuzz run satoshis_arithmetic -- -max_total_time=10

# Run the demonstration
./reproduce_crash.sh

# Reproduce the specific crash
cargo +nightly fuzz run satoshis_arithmetic artifacts/satoshis_arithmetic/crash-*
```

### Next Priority Targets

Based on this success, implement fuzz tests for:

1. **Account Code Parsing** (`core/accounting/fuzz/`)
2. **JSON Deserialization** (`lib/fuzz/`)  
3. **GraphQL Input Validation** (`lana/admin-server/fuzz/`)
4. **CSV Processing** (`core/accounting/fuzz/`)

This systematic approach will significantly improve the robustness and security of the core banking application.