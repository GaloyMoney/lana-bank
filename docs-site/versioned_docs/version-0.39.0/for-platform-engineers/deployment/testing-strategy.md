---
id: testing-strategy
title: Testing Strategy
sidebar_position: 4
---

# Testing Strategy

This document describes the testing approach used in Lana.

## Test Pyramid

```
                    ┌─────────────────┐
                    │    E2E Tests    │
                    │    (BATS)       │
                    └─────────────────┘
               ┌─────────────────────────────┐
               │    Integration Tests        │
               │    (Database, APIs)         │
               └─────────────────────────────┘
          ┌─────────────────────────────────────────┐
          │           Unit Tests                     │
          │    (Domain Logic, Pure Functions)        │
          └─────────────────────────────────────────┘
```

## Test Types

### Unit Tests

Test isolated business logic:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_interest_calculation() {
        let rate = InterestRate::new(Decimal::new(5, 2)); // 5%
        let principal = UsdCents::new(100_000);
        let days = 30;

        let interest = calculate_interest(principal, rate, days);

        assert_eq!(interest, UsdCents::new(411)); // ~$4.11
    }
}
```

### Integration Tests

Test with real database:

```rust
#[tokio::test]
async fn test_create_facility() {
    let app = TestApp::new().await;

    let customer = app.create_customer().await;
    let facility = app.create_facility(customer.id).await;

    assert_eq!(facility.status, FacilityStatus::PendingCollateral);
}
```

### E2E Tests (BATS)

Test complete workflows:

```bash
@test "create customer and facility" {
    # Create customer
    run create_customer
    [ "$status" -eq 0 ]

    # Create facility
    run create_facility "$customer_id"
    [ "$status" -eq 0 ]
}
```

### Frontend Tests (Cypress)

```typescript
describe('Credit Facility', () => {
  it('should create a new facility', () => {
    cy.login('admin', 'admin');
    cy.visit('/credit/new');
    cy.get('[data-testid="customer-select"]').click();
    cy.get('[data-testid="customer-option"]').first().click();
    cy.get('[data-testid="amount-input"]').type('10000');
    cy.get('[data-testid="submit-button"]').click();
    cy.contains('Facility created').should('be.visible');
  });
});
```

## Running Tests

### Rust Tests

```bash
# All tests
cargo nextest run

# Single crate
cargo nextest run -p core-credit

# Single test
cargo nextest run credit::tests::test_create_facility
```

### E2E Tests

```bash
make e2e
```

### Frontend Tests

```bash
# Headless
pnpm cypress:run-headless

# Interactive
pnpm cypress:open
```

## Test Data

### Fixtures

```rust
pub struct TestFixtures {
    pub customer: Customer,
    pub facility: CreditFacility,
}

impl TestFixtures {
    pub async fn create(app: &TestApp) -> Self {
        let customer = app.create_customer().await;
        let facility = app.create_facility(customer.id).await;
        Self { customer, facility }
    }
}
```

### Database Seeding

```bash
# Seed development data
cargo run -- seed
```

## Coverage

```bash
# Generate coverage report
cargo llvm-cov --html

# Open report
open target/llvm-cov/html/index.html
```

## CI Integration

Tests run on every PR:

- Unit tests (cargo nextest)
- Integration tests
- E2E tests (BATS)
- Frontend tests (Cypress)
- Code coverage check

