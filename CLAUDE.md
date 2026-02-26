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

For detailed patterns on `Idempotent<T>`, `idempotency_guard!`, and infallible queries, use the `lana-es-entity-patterns` skill.

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
- **NEVER use `.map_err()` when a `From` impl exists** — just use `?`. The `?` operator automatically converts errors via `From`. Writing `.map_err(MyError::Variant)` when `#[from]` is already derived is redundant and obscures the conversion. Only use `.map_err()` when there is no `From` impl and you don't want to add one.

## Git and Github
- When checking github action (gh pr checks), use a timeout of 30m
- Use conventional commits
- Open draft PR by default

## Observability
- For anything related to traces, spans, performance, latency, errors, jobs, or runtime behavior — whether asked by the user or needed during your own investigation — delegate to the `lana-trace-analyzer` subagent. Do not query tracing backends directly.

## Frontend
- Do not edit es.json, let lingo.dev github action do it.