# Testing Structure Analysis for Lana Bank Codebase

## Overview

This analysis examines the testing structure across the Lana Bank monorepo to understand how tests are organized and how you should think about testing at different levels.

## Testing Structure Overview

The codebase follows a multi-layered testing approach with different types of tests serving different purposes:

### 1. Unit Tests (Rust `#[test]` annotations)

**Location**: Embedded within source files using `#[cfg(test)]` modules

**Purpose**: Test individual functions, structs, and small units of logic in isolation

**Example patterns found**:
- `core/accounting/src/chart_of_accounts/csv.rs` - Tests CSV parsing logic
- `core/accounting/src/primitives.rs` - Tests primitive data structures
- `core/credit/src/terms/value.rs` - Tests credit term calculations

**How to think about them**: These are the fastest tests and test the smallest units of functionality. They should cover edge cases, validation logic, and basic business rules at the individual function level.

### 2. Integration Tests (Rust `tests/` directories)

**Location**: `core/*/tests/` directories

**Purpose**: Test how multiple components work together within a single module

**Structure**:
- Each `core/` module has its own `tests/` directory
- Common pattern: `helpers.rs` file provides shared test utilities
- Test files are named after the functionality they test (e.g., `ledger_account.rs`, `trial_balance.rs`)

**Key characteristics**:
- Use real database connections (`init_pool()` helper)
- Set up realistic test data and scenarios
- Test cross-module interactions within the core domain
- Use dummy implementations for authorization and external dependencies

**Example**: `core/accounting/tests/ledger_account.rs` tests how chart of accounts integrates with the Cala ledger system.

### 3. End-to-End Tests (BATS)

**Location**: `bats/` directory

**Purpose**: Test the entire system from the user's perspective via GraphQL APIs

**Structure**:
- Written in BATS (Bash Automated Testing System)
- Each `.bats` file tests a specific business domain (accounting, credit, customer, etc.)
- `helpers.bash` provides shared utilities for server management, authentication, and GraphQL execution
- Test data lives in subdirectories like `admin-gql/`, `customer-gql/`, `accounting-init/`

**Key characteristics**:
- Start/stop the actual server
- Use real HTTP/GraphQL requests
- Test complete user workflows
- Verify business logic end-to-end
- Test authentication and authorization

**Example**: `bats/accounting.bats` tests importing chart of accounts, executing transactions, and verifying ledger balances.

## Consistency Analysis

### Consistent Patterns:

1. **Helper modules**: All test directories have a `helpers.rs` file with common test utilities
2. **Database setup**: Integration tests consistently use `init_pool()` and `init_journal()` helpers
3. **Dummy implementations**: Tests use consistent dummy implementations for authorization (`DummySubject`, `DummyAction`, `DummyObject`)
4. **Error handling**: Tests consistently use `anyhow::Result<()>` for error handling
5. **Async testing**: Integration tests use `#[tokio::test]` for async test execution

### Test Data Management:

- **Unit tests**: Use hardcoded test data and random values
- **Integration tests**: Create test data programmatically with random identifiers to avoid conflicts
- **E2E tests**: Use both programmatic test data creation and seed data files

## How to Think About Each Test Type

### Unit Tests (`#[test]`)
- **When to use**: Testing individual functions, validation logic, calculations, parsing
- **What to test**: Edge cases, error conditions, business rule validation
- **Dependencies**: Mock/stub external dependencies
- **Speed**: Very fast (milliseconds)

### Integration Tests (`tests/` directory)
- **When to use**: Testing how components within a module work together
- **What to test**: Database interactions, cross-component workflows, complex business logic
- **Dependencies**: Use real database, dummy auth, mock external services
- **Speed**: Medium (seconds)

### End-to-End Tests (BATS)
- **When to use**: Testing complete user workflows and system behavior
- **What to test**: API contracts, authentication, full business scenarios
- **Dependencies**: Full system with real services
- **Speed**: Slow (minutes)

## Recommendations

1. **Follow the pyramid**: Write many unit tests, fewer integration tests, and select E2E tests for critical paths
2. **Use the existing helper patterns**: Leverage the established `helpers.rs` patterns for consistency
3. **Test at the right level**: Don't test business logic in E2E tests if it can be tested in unit tests
4. **Maintain isolation**: Integration tests should be able to run in parallel without interfering with each other
5. **Keep E2E tests focused**: BATS tests should verify complete workflows rather than individual functions

## Special Considerations

- **Ledger testing**: The Cala ledger integration requires careful setup and is tested primarily at the integration level
- **Authorization**: The RBAC system uses dummy implementations in tests, with real authorization tested in E2E scenarios
- **Event sourcing**: The system uses event sourcing patterns, so tests often verify both state changes and events
- **Banking regulations**: Tests include validation of accounting rules (double-entry bookkeeping, trial balance, etc.)