---
name: lana-test-writer
description: Guides test writing in the LANA Bank codebase. Use when writing new tests, deciding where tests should live, or understanding the testing philosophy.
---

# LANA Test Writing Guide

## Testing Philosophy

LANA follows hexagonal architecture and DDD principles. This architectural choice directly impacts where and how tests are written:

- **Logic lives in entities and value objects** - Business logic is pushed into the domain layer
- **Use cases are thin orchestrators** - They coordinate database operations and call entity methods
- **This makes mocking unnecessary** - Domain objects can be tested in isolation without external dependencies

# Unit Tests

- `cargo nextest run` - Run all Rust tests
- `cargo nextest run -p <crate>` - Run tests for a specific crate

## Where to Write Them

Write unit tests **directly in entity and value object files**, not in use cases or service layers.

- Entity tests: In `entity.rs` within a `#[cfg(test)] mod tests` block
- Value object tests: In the same file where the value object is defined, within a `#[cfg(test)] mod tests` block (often `primitives.rs`, but value objects can live in other files too). Use subagents to search for `#[cfg(test)]` modules in files containing value objects to see patterns.

Do NOT write unit tests for:
- Use cases (these are thin wrappers)
- Repository implementations
- Adapter layers

## No Mocking Policy

Unit tests should not use mocking frameworks. The architecture makes this unnecessary:

1. Entities contain business logic that operates on their own state
2. Value objects are self-contained with validation logic
3. Neither requires external dependencies to test

If you find yourself needing to mock something, consider whether the logic should be moved into an entity or value object.

## Entity Testing with Event Rehydration

LANA uses event sourcing for entities. Testing entities requires understanding the rehydration pattern:

1. Entities are reconstituted from a sequence of events
2. Tests build event vectors representing the entity's history
3. The entity is rehydrated using `TryFromEvents::try_from_events`
4. Commands can then be executed on the rehydrated entity

Use subagents to explore existing entity tests via `Glob("**/core/*/src/**/entity.rs")` to understand the specific patterns used in this codebase. These patterns can be mimicked when writing new tests.

## Comprehensive Test Planning

Before writing tests, create a plan to ensure thorough coverage:

1. **Examine existing tests** - Use subagents to find tests already covering the object/method being tested
2. **Identify all possible outcomes** - List every success case, error case, and edge case the method can produce
3. **Determine what's missing** - Compare existing coverage against the full outcome list
4. **Update or extend tests** - Modify existing tests or add new ones to achieve holistic coverage

Existing tests may need updates when:
- New functionality is added to a method
- Edge cases were previously untested
- Test setup can be shared across multiple scenarios

The goal is a complete, maintainable test suite - not just adding new tests in isolation.

# Integration Tests

- `cargo nextest run` - Run all Rust tests
- `cargo nextest run -p <crate>` - Run tests for a specific crate

## When Database Interaction Is Acceptable

Some functionality requires actual database interaction for testing:
- Event publication and outbox patterns
- Repository behavior verification
- Multi-step workflows involving persistence

These tests live in `/{core,lib}/<module>/tests/` directories. Use subagents with `Glob("**/{core,lib}/**/tests/")` to explore existing patterns.

**This is the exception, not the rule.** Only use database-backed tests when:
1. Testing outbox/event publication workflows
2. The behavior fundamentally depends on persistence semantics
3. There is specific behavior in the service/application layer that requires test-based verification

# BATS E2E Tests

`make e2e` - Run BATS integration tests

## Purpose

BATS tests verify end-to-end behavior through the GraphQL API. They exist in `/bats/`.

## What to Test

- **Happy paths**: Successful workflows and operations
- **User journeys**: Complete business processes (e.g., loan origination to disbursement)
- **API contracts**: GraphQL mutations and queries work as documented

## What NOT to Test

- **Edge cases**: These belong in unit tests where validation logic lives
- **Error conditions**: Test these at the entity/value object level
- **Internal implementation details**: BATS tests should treat the system as a black box

## Structure and Conventions

BATS tests use helper functions for common operations:
- GraphQL execution helpers
- State caching utilities
- Balance verification functions

## Shared Database State

All BATS tests run against a **single shared database instance**. This has important implications:

- **Tests are not isolated**: Each test inherits state from all previous tests
- **Sequential dependencies**: Tests are sometimes intentionally written as sequences where each builds on prior state
- **Pre-existing data**: When writing new tests, assume the database already contains data from other tests - it will not be a clean slate
- **Execution order matters**: Be aware that test files execute in a specific order, and tests within a file run sequentially

When writing new BATS tests:
1. Consider what state the database will be in when your test runs
2. Use unique identifiers or timestamps to avoid conflicts with existing test data
3. If your test relies on specific state, ensure earlier tests create that state or create it yourself
4. Do not assume any table is empty

# Test Writing Checklist

Before writing a test, answer these questions:

- [ ] **Where does the logic live?** If in an entity or VO, write a unit test
- [ ] **Am I testing a happy path through the API?** Consider a BATS test
- [ ] **Am I testing an edge case or validation?** This is a unit test
- [ ] **Do I need database interaction?** Only use `/core/<module>/tests/` if absolutely necessary
- [ ] **Am I tempted to mock?** Reconsider - the logic might be in the wrong place
- [ ] **Have I checked existing patterns?** Use subagents to explore before inventing
