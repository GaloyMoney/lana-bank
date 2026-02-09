# LANA Bank - Code Guidelines

## Overview
Core banking application for bitcoin-backed lending. Uses CALA ledger for double-entry bookkeeping with strong consistency guarantees through custom templates.

## Development Environment
- Requires Nix flakes - run `nix develop` for shell with all dependencies
- Includes: Node 20, Rust stable, Python 3.13, pnpm 10
- `make dev-up` / `make dev-down` - Start/stop full stack with Tilt (interactive development)
- For AI-driven development, prefer:
  - `make start-deps` - Start dependencies (databases, Keycloak, etc.)
  - `make reset-deps` - Clean and restart databases
  - `make stop-deps` - Stop dependencies
  - `DAGSTER=true make start-deps` - Start dependencies including Dagster

## Local Development URLs
| Service | URL/Port |
|---------|----------|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |
| Admin GraphQL | http://admin.localhost:4455/graphql (via Oathkeeper) |
| Customer GraphQL | http://app.localhost:4455/graphql (via Oathkeeper) |
| Keycloak | http://localhost:8081 |
| PostgreSQL (lana) | localhost:5433 |

Note: GraphQL APIs must be accessed through Oathkeeper (port 4455) which handles JWT validation. Direct ports (5253/5254) lack authentication and won't work.

## Build/Lint/Test Commands
- `make check-code-rust` - Verify Rust code compiles
- `make check-code-apps` - Verify frontend code (lint, type check, build)
- `cargo nextest run` - Run all tests
- `cargo nextest run -p <crate>` - Run single crate tests
- `cargo nextest run <crate>::<module>::<test_name>` - Run single test
- `make e2e` - Run BATS end-to-end tests (configs in `/bats/`)
- `pnpm cypress:run-headless` - Frontend E2E tests
- `make sqlx-prepare` - Update .sqlx offline cache (don't edit manually)
- Prefix direct cargo commands with `SQLX_OFFLINE=true`

## Directory Structure
| Directory | Purpose |
|-----------|---------|
| `/core/` | Rust: Domain logic modules (accounting, credit, customer, deposit) |
| `/lana/` | Rust: Application layer (admin-server, customer-server, CLI) |
| `/lib/` | Rust: Shared libraries/adapters (audit, authz, cloud-storage, smtp) |
| `/apps/` | Next.js frontends (admin-panel, customer-portal) |
| `/dagster/` | Data pipelines (dbt + Dagster) |
| `/tf/` | Terraform/OpenTofu infrastructure |
| `/bats/` | E2E test configurations |

## Code Style
- **Rust Architecture**: Hexagonal architecture with adapter/use-case layers
- **File Naming**: `mod.rs` (interface), `repo.rs` (storage), `entity.rs` (private events), `error.rs` (errors), `primitives.rs` (value objects), `publisher.rs` (outbox), `job.rs` (async jobs)
- **Public Events**: `public/` subfolder contains public events shared across module boundaries
- **Dependencies**: Add to root Cargo.toml with `{ workspace = true }`
- **GraphQL**: Don't edit schema.graphql manually, use `make sdl`
- **Formatting**: Use Rust fmt, Follow DDD (Domain-Driven Design) pattern
- **Error Handling**: Module-specific errors in `error.rs`
- **Frontend**: NextJS with TypeScript, lint with `pnpm lint`

## Module Structure
- Core can import lib, lana can import core
- Events flow between module boundaries

## Event Sourcing
- Uses `es-entity` external crate for event-driven architecture (provides `EsEntity`, `EsRepo` derive macros)
- Entity files define events and state transitions
- Events are immutable, state rebuilt from event stream
- EsEntity structs follow DDD with two method types:
  - **Commands** (`&mut self`): mutate state, return `Idempotent<T>` or `Result<Idempotent<T>, E>`
  - **Queries** (`&self`): read state, return direct values or `Option<T>`, never `Result`

### Idempotent Entity Mutations (lint: `entity-mutate-idempotent`)
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

### Infallible Entity Queries (lint: `entity-query-infallible`)
All public `&self` methods on `#[derive(EsEntity)]` structs must NOT return `Result`. This is enforced by the `entity-query-infallible` custom lint. Private methods (`fn`, no `pub`) are exempt.

- Queries return direct values or `Option<T>`, never `Result`
- If validation is needed, move it to the caller (use-case layer) or a constructor/wrapper type
- Example: instead of `fn check(&self) -> Result<Data, Error>`, use `fn find(&self) -> Option<Data>` and let the caller construct the appropriate error

## Database
- Migrations location: `/lana/app/migrations/`
- Run migrations: `cargo sqlx migrate run`
- SQLx provides compile-time query verification
- Run `cargo sqlx prepare` to update offline query cache

## Authentication & Authorization
- Keycloak for OIDC identity (separate realms for admin/customer)
- Oathkeeper gateway validates JWTs
- Casbin for RBAC (policies stored in PostgreSQL)
- Audit logging for all authorization decisions

## CI/CD
- PRs trigger: nextest, cypress, bats, check-code-apps
- Security scanning: cargo audit, cargo deny, pnpm audit, CodeQL
- Schema changes require `make sdl` before commit

## Common Pitfalls
- Run `cargo fmt` before commit
- Always regenerate GraphQL schema after Rust changes: `make sdl`
- SQLx queries are compile-time checked - update offline cache if needed
- Frontend uses Apollo Client - run codegen after schema changes
- Don't mix admin/customer Keycloak realms

## Function Naming Conventions (lint: `constructor-naming`)
- `new` - Always succeeds, sync. Must NOT return `Result`. Must NOT be async.
- `try_new` - Might fail, sync. MUST return `Result`. Must NOT be async.
- `init` - Might fail, async. MUST be async. MUST return `Result`.
- `*_without_audit` suffix - Must return `Result<X, XError>`, not `Option`

## Coding Rules
- Use rustls, not openssl (use `rustls-tls` feature flag)
- Use Strum library for enum <-> string conversion
- Use OTEL for debugging, not println (except in tests)
- Use `#[serde(rename_all = "camelCase")]` instead of manual field renames
- Don't add `#[allow(dead_code)]`
- Prefer `?` operator over `.map_err()` for error conversion when `From` is implemented

## Git and Github
- When checking github action (gh pr checks), use a timeout of 30m
- Use conventional commits
- Open draft PR by default
