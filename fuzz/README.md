# Fuzz Testing for Lana Bank Core Types

This directory contains fuzz tests for critical core banking types, following the pattern established by [rust-lightning](https://github.com/lightningdevkit/rust-lightning/tree/main/fuzz).

## Architecture

### Self-Contained Workspace

Following rust-lightning's approach, this fuzz directory operates as a **self-contained workspace** (`[workspace] members = ["."]`) that:

- **Prevents workspace conflicts** - No interference with the main workspace
- **Isolates dependencies** - Fuzz-specific dependencies don't affect main project  
- **Enables independent configuration** - Can have different profiles, lints, etc.
- **Simplifies CI/CD** - Fuzzing can be run independently

### Dependency Isolation

Unlike the main workspace, fuzz tests use explicit dependency versions to ensure complete isolation:

```toml
# Explicit versions for isolation from main workspace
rust_decimal = "1.36"
serde = { version = "1.0", features = ["derive"] }
# ... etc
```

This prevents conflicts with GraphQL/async dependencies that would interfere with fuzzing.

## Setup

### Prerequisites

The Nix environment (flake.nix) provides all necessary tools:
- **Rust Nightly** - Required for libfuzzer integration
- **cargo-fuzz** - Fuzzing framework  
- **libfuzzer** - Underlying fuzzing engine

No manual installation required - everything is provided by the flake!

### Self-Contained Workspace

The fuzz directory operates independently from the main workspace:

- Has its own `Cargo.toml` with `[workspace] members = ["."]`
- Uses explicit dependency versions for complete isolation
- Can be built and run without main workspace dependencies

## Quick Start

Run the simplified fuzz testing with a single Makefile target:

```bash
# Run 60-second fuzz test (includes all setup)
make fuzz
```

Or explore with the demo:

```bash
cd fuzz
./demo.sh
```

### Manual Testing

If you want to run specific targets manually:

```bash
# Test money arithmetic for 10 seconds
cd fuzz
cargo +nightly fuzz run money_arithmetic -- -max_total_time=10

# Test JSON serialization  
cargo +nightly fuzz run json_serialization -- -max_total_time=10
```

### CI Integration

Add to your CI pipeline:

```bash
# Quick smoke tests (60 seconds)
make fuzz
```

## Fuzz Targets

### 1. Money Arithmetic (`money_arithmetic`)

**Purpose**: Find bugs in financial calculations, conversions, and overflow handling.

**Tests**:
- Satoshis and SignedSatoshis arithmetic operations
- BTC conversion round-trips
- USD operations  
- Type conversions (signed ↔ unsigned)
- Overflow and underflow edge cases

**Key Bug Found**: Conversion from large Satoshis to SignedSatoshis panics instead of returning an error.

### 2. Account Parsing (`account_parsing`)

**Purpose**: Find edge cases in account code and name validation.

**Tests**:
- AccountName, AccountCode, AccountCodeSection parsing
- String validation and normalization
- Round-trip serialization
- Special character handling

### 3. JSON Serialization (`json_serialization`)

**Purpose**: Find serialization/deserialization bugs across all core types.

**Tests**:
- Core money types (Satoshis, UsdCents, etc.)
- Accounting primitives
- Nested structures and collections
- Malformed JSON handling
- Round-trip serialization

## Corpus Management

Each target has a corpus directory with seed inputs:

- `corpus/money_arithmetic/` - Edge case numeric values
- `corpus/account_parsing/` - Valid account codes and names  
- `corpus/json_serialization/` - JSON structures with core types

### Adding New Corpus Files

```bash
# Add a new money test case
echo -n -e '\x00\x00\x00\x00\x00\x00\x00\x01\xff\xff\xff\xff\xff\xff\xff\xff' > corpus/money_arithmetic/edge_case

# Add account parsing case  
echo -n 'liabilities:loans:bitcoin_backed' > corpus/account_parsing/loan_account

# Add JSON case
echo -n '{"amount": {"satoshis": -1}, "account": "invalid::"}' > corpus/json_serialization/edge_case
```

## Integration with Makefile

Add these targets to your `Makefile`:

```makefile
.PHONY: fuzz-test fuzz-money fuzz-accounting fuzz-json fuzz-ci

fuzz-test: fuzz-money fuzz-accounting fuzz-json

fuzz-money:
	cd fuzz && cargo +nightly fuzz run money_arithmetic -- -max_total_time=300

fuzz-accounting:
	cd fuzz && cargo +nightly fuzz run account_parsing -- -max_total_time=300

fuzz-json:
	cd fuzz && cargo +nightly fuzz run json_serialization -- -max_total_time=300

fuzz-ci:
	# Short runs for CI
	cd fuzz && cargo +nightly fuzz run money_arithmetic -- -max_total_time=60
	cd fuzz && cargo +nightly fuzz run account_parsing -- -max_total_time=60
	cd fuzz && cargo +nightly fuzz run json_serialization -- -max_total_time=60
```

## Findings and Bug Reports

### Critical Bug: Money Conversion Panic

**Target**: `money_arithmetic`  
**Issue**: Converting large `Satoshis` to `SignedSatoshis` panics instead of returning error
**Location**: `core/money/src/lib.rs` - `From<Satoshis> for SignedSatoshis`
**Fix**: Replace `From` with `TryFrom` and proper error handling

### Running Specific Crashes

When fuzz testing finds a crash, reproduce it with:

```bash
cargo +nightly fuzz run money_arithmetic artifacts/money_arithmetic/crash-<hash>
```

## Best Practices

1. **Regular Testing**: Run fuzz tests locally before major commits
2. **CI Integration**: Include short fuzz runs in your CI pipeline  
3. **Corpus Growth**: Save interesting inputs to expand test coverage
4. **Bug Tracking**: Document and track all fuzz-discovered issues
5. **Coverage Monitoring**: Use coverage tools to ensure fuzz tests reach critical code paths

## Advanced Usage

### Long-running Fuzzing

For deeper testing, run extended sessions:

```bash
# Run for 1 hour
cargo +nightly fuzz run money_arithmetic -- -max_total_time=3600

# Run indefinitely (stop with Ctrl+C)
cargo +nightly fuzz run money_arithmetic
```

### Custom Dictionaries

Create custom dictionaries for better fuzzing:

```bash
# Create dictionary file
echo "assets" > fuzz/dict/accounting_terms
echo "liabilities" >> fuzz/dict/accounting_terms
echo "equity" >> fuzz/dict/accounting_terms

# Use dictionary
cargo +nightly fuzz run account_parsing -- -dict=dict/accounting_terms
```

### Coverage Analysis

```bash
# Generate coverage data
cargo +nightly fuzz coverage money_arithmetic
cargo +nightly cov -- show target/*/release/money_arithmetic -instr-profile=coverage/money_arithmetic/coverage.profdata
```

## Security Implications

This fuzz testing is critical for financial software because:

- **Arithmetic bugs** can lead to incorrect money calculations
- **Parsing vulnerabilities** can be exploited via malformed API inputs
- **Serialization issues** can cause data corruption or crashes
- **Overflow conditions** may allow value manipulation

Always investigate and fix any crashes or panics found during fuzzing, as they represent potential security vulnerabilities in a financial application. 