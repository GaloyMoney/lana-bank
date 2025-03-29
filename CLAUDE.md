# LANA Bank - Code Guidelines

## Build/Lint/Test Commands
- `make check-code` - Verify backend code (fmt, check, clippy, audit)
- `make check-code-apps` - Verify frontend code (lint, type check, build)
- `cargo nextest run` - Run all tests
- `cargo nextest run package::module::test_name` - Run single test
- `make e2e` - Run all E2E tests

## Code Style
- **Rust Architecture**: Hexagonal architecture with adapter/use-case layers
- **File Naming**: `mod.rs` (interface), `repo.rs` (storage), `entity.rs` (events), `error.rs` (errors)
- **Dependencies**: Add to root Cargo.toml with `{ workspace = true }`
- **GraphQL**: Don't edit schema.graphql manually, use `make sdl`
- **Formatting**: Use Rust fmt, Follow DDD (Domain-Driven Design) pattern
- **Error Handling**: Module-specific errors in `error.rs`
- **Frontend**: NextJS with TypeScript, lint with `pnpm lint`

## Module Structure
- Core can import lib, lana can import core
- Entity mutations must be idempotent, command functions use `&mut self`
- Events flow between module boundaries