---
id: testing-strategy
title: Estrategia de Pruebas
sidebar_position: 4
---

# Estrategia de Pruebas

Este documento describe las capas de pruebas, herramientas y metodologías utilizadas en Lana Bank para garantizar la calidad del código.

## Capas de Pruebas

### Pirámide de Pruebas

```
                    ┌─────────────┐
                    │    E2E      │  Cypress
                    │   (UI)      │
                    ├─────────────┤
                    │Integration  │  BATS
                    │   (API)     │
                    ├─────────────┤
                    │             │
                    │    Unit     │  cargo nextest
                    │   (Rust)    │
                    │             │
                    └─────────────┘
```

| Capa | Herramienta | Propósito |
|------|-------------|-----------|
| Unit | cargo nextest | Lógica de dominio y servicios |
| Integration | BATS | API GraphQL y flujos de negocio |
| E2E | Cypress | Interfaz de usuario |
| Data Pipeline | dbt test | Transformaciones de datos |

## Pruebas Unitarias con cargo nextest

### Ejecución de Pruebas

```bash
# Ejecutar todas las pruebas
cargo nextest run

# Ejecutar pruebas de un crate específico
cargo nextest run -p core-credit

# Ejecutar una prueba específica
cargo nextest run core_credit::credit_facility::tests::test_create

# Con logs detallados
RUST_LOG=debug cargo nextest run
```

### Configuración del Ejecutador

```toml
# .config/nextest.toml
[profile.default]
retries = 0
slow-timeout = { period = "60s", terminate-after = 2 }

[profile.ci]
retries = 2
fail-fast = false
```

### Patrón de Pruebas Unitarias

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_credit_facility() {
        // Arrange
        let app = test_app().await;
        let customer = create_test_customer(&app).await;

        let input = CreateFacilityInput {
            customer_id: customer.id,
            amount: Money::from_usd(100_000),
            terms_id: default_terms_id(),
        };

        // Act
        let facility = app.credit_facilities
            .create(test_subject(), input)
            .await
            .unwrap();

        // Assert
        assert_eq!(facility.status, FacilityStatus::Pending);
        assert_eq!(facility.customer_id, customer.id);
    }

    #[tokio::test]
    async fn test_activate_requires_approval() {
        let app = test_app().await;
        let facility = create_pending_facility(&app).await;

        // Sin aprobación, debería fallar
        let result = app.credit_facilities
            .activate(facility.id)
            .await;

        assert!(matches!(
            result,
            Err(CreditError::RequiresApproval)
        ));
    }
}
```

## Pruebas End-to-End con BATS

### Infraestructura de Pruebas BATS

```bash
# Estructura de archivos
bats/
├── helpers.bash          # Funciones auxiliares
├── credit-facility.bats  # Pruebas de crédito
├── deposit.bats          # Pruebas de depósitos
├── customer.bats         # Pruebas de clientes
└── accounting.bats       # Pruebas contables
```

### Funciones Auxiliares Clave

```bash
# bats/helpers.bash

# Obtener token de administrador
get_admin_token() {
    curl -s -X POST \
        "http://localhost:8081/realms/internal/protocol/openid-connect/token" \
        -d "client_id=admin-panel" \
        -d "username=admin" \
        -d "password=admin" \
        -d "grant_type=password" \
        | jq -r '.access_token'
}

# Ejecutar query GraphQL
graphql_admin() {
    local query="$1"
    local variables="${2:-{}}"
    local token=$(get_admin_token)

    curl -s -X POST \
        "http://admin.localhost:4455/graphql" \
        -H "Authorization: Bearer $token" \
        -H "Content-Type: application/json" \
        -d "{\"query\": \"$query\", \"variables\": $variables}"
}
```

### Patrón de Ejecución de Consultas GraphQL

```bash
# bats/credit-facility.bats

@test "create credit facility" {
    # Crear cliente primero
    customer_id=$(create_customer "Test Corp")

    # Crear facilidad
    result=$(graphql_admin "
        mutation CreateFacility(\$input: CreateCreditFacilityInput!) {
            creditFacilityCreate(input: \$input) {
                creditFacility {
                    id
                    status
                }
            }
        }
    " "{\"input\": {\"customerId\": \"$customer_id\", \"amount\": 100000}}")

    # Verificar
    facility_id=$(echo "$result" | jq -r '.data.creditFacilityCreate.creditFacility.id')
    status=$(echo "$result" | jq -r '.data.creditFacilityCreate.creditFacility.status')

    [ "$status" = "PENDING" ]
    [ -n "$facility_id" ]
}
```

### Aserciones Contables

```bash
# Verificar balance de cuenta
assert_account_balance() {
    local account_code="$1"
    local expected_balance="$2"
    local currency="${3:-USD}"

    local result=$(graphql_admin "
        query GetBalance(\$code: String!) {
            ledgerAccountByCode(code: \$code) {
                balance(currency: $currency) {
                    settled
                }
            }
        }
    " "{\"code\": \"$account_code\"}")

    local actual=$(echo "$result" | jq -r '.data.ledgerAccountByCode.balance.settled')

    [ "$actual" = "$expected_balance" ]
}
```

## Simulaciones de Escenarios

### Arquitectura de Pruebas de Simulación

```rust
// core/credit/tests/scenarios/mod.rs
pub struct Scenario {
    pub name: String,
    pub steps: Vec<ScenarioStep>,
}

pub enum ScenarioStep {
    CreateCustomer(CustomerInput),
    CreateFacility(FacilityInput),
    RecordDeposit(DepositInput),
    AdvanceTime(Duration),
    AssertBalance(AccountCode, Money),
}
```

### Implementación del Ejecutador

```rust
pub async fn run_scenario(scenario: Scenario, app: &LanaApp) -> Result<(), ScenarioError> {
    for step in scenario.steps {
        match step {
            ScenarioStep::CreateCustomer(input) => {
                app.customers.create(input).await?;
            }
            ScenarioStep::AdvanceTime(duration) => {
                // Simular paso del tiempo para devengo de intereses
                app.time_service.advance(duration);
                app.interest_accrual.process().await?;
            }
            ScenarioStep::AssertBalance(code, expected) => {
                let actual = app.accounting.get_balance(&code).await?;
                assert_eq!(actual, expected);
            }
            // ... más steps
        }
    }
    Ok(())
}
```

## Pruebas de UI con Cypress

### Stack de Pruebas de Cypress

```bash
# Estructura
apps/admin-panel/cypress/
├── e2e/
│   ├── login.cy.ts
│   ├── customers.cy.ts
│   └── credit-facilities.cy.ts
├── fixtures/
│   └── test-data.json
└── support/
    ├── commands.ts
    └── e2e.ts
```

### Inicio del Stack de Cypress

```bash
# Iniciar stack completo para Cypress
make cypress-stack

# Ejecutar tests headless
pnpm cypress:run-headless

# Ejecutar tests interactivamente
pnpm cypress:open
```

### Ejemplo de Test Cypress

```typescript
// cypress/e2e/credit-facilities.cy.ts
describe('Credit Facilities', () => {
    beforeEach(() => {
        cy.login('admin', 'admin');
    });

    it('creates a new credit facility', () => {
        // Navegar a facilidades
        cy.visit('/credit-facilities');
        cy.get('[data-testid="create-facility-btn"]').click();

        // Llenar formulario
        cy.get('[data-testid="customer-select"]').click();
        cy.contains('Test Customer').click();

        cy.get('[data-testid="amount-input"]').type('100000');
        cy.get('[data-testid="terms-select"]').click();
        cy.contains('Standard Terms').click();

        // Enviar
        cy.get('[data-testid="submit-btn"]').click();

        // Verificar creación
        cy.url().should('include', '/credit-facilities/');
        cy.contains('Pending Approval');
    });
});
```

## Pruebas de la Canalización de Datos

### Arquitectura de Pruebas

```bash
# Ejecutar pruebas de dbt
cd meltano
meltano invoke dbt test

# Ejecutar pruebas de frescura
meltano invoke dbt source freshness
```

### Tests de dbt

```yaml
# models/schema.yml
models:
  - name: credit_facilities_summary
    tests:
      - not_null:
          column_name: facility_id
      - unique:
          column_name: facility_id
    columns:
      - name: total_disbursed
        tests:
          - not_negative
```

## Calidad de Código y Análisis Estático

### Herramientas de Análisis

| Herramienta | Propósito | Comando |
|-------------|-----------|---------|
| clippy | Linting Rust | `cargo clippy` |
| rustfmt | Formateo Rust | `cargo fmt` |
| eslint | Linting TypeScript | `pnpm lint` |
| prettier | Formateo TypeScript | `pnpm format` |

### Verificaciones de Calidad Rust

```bash
# Clippy con warnings como errores
cargo clippy -- -D warnings

# Verificar formato
cargo fmt --check

# Verificar dependencias
cargo deny check
cargo audit
```

### Verificaciones de Calidad Frontend

```bash
# Lint
pnpm lint

# Type check
pnpm typecheck

# Format check
pnpm format:check
```

## Ejecución de Pruebas en CI/CD

### Matriz de Pruebas en GitHub Actions

```yaml
# .github/workflows/test.yml
jobs:
  test:
    strategy:
      matrix:
        test-type: [unit, e2e, cypress]

    steps:
      - uses: actions/checkout@v4

      - name: Run tests
        run: |
          case "${{ matrix.test-type }}" in
            unit) make test ;;
            e2e) make e2e ;;
            cypress) make cypress-headless ;;
          esac
```

### Comandos de Ejecución de Pruebas

```bash
# CI completo
make ci

# Solo tests unitarios
make test

# Solo E2E
make e2e

# Solo Cypress
make cypress-headless
```

## Flujos de Pruebas Locales

### Opciones de Ejecución

```bash
# Tests rápidos (sin base de datos)
cargo nextest run --no-default-features

# Tests con base de datos
make start-deps
cargo nextest run

# Tests específicos
cargo nextest run -p core-credit --test credit_facility

# Watch mode
cargo watch -x "nextest run"
```

### Generación de Datos de Prueba

```bash
# Seed de datos de prueba
cargo run --package lana-cli -- seed

# Generar datos aleatorios
cargo run --package lana-cli -- seed --random --count 100
```

## Variables de Entorno de Pruebas

### Variables Críticas

| Variable | Propósito | Valor Test |
|----------|-----------|------------|
| DATABASE_URL | Conexión PostgreSQL | postgres://lana:lana@localhost:5433/lana_test |
| SQLX_OFFLINE | Modo offline de SQLx | true |
| RUST_LOG | Nivel de logging | info |
| TEST_KEYCLOAK_URL | URL de Keycloak | http://localhost:8081 |
