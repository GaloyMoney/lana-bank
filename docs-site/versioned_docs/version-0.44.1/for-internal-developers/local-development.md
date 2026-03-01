---
id: local-development
title: Local Development Setup
sidebar_position: 2
---

# Local Development Setup

This guide walks you through setting up a local development environment for lana-bank.

## Prerequisites

- [Nix](https://nixos.org/download.html) with flakes enabled
- Docker and Docker Compose

## Quick Start

### 1. Enter the Nix Shell

```bash
nix develop
```

This provides a reproducible shell with all required tools: Rust stable toolchain, Node.js 20, pnpm 10, Python 3.13, PostgreSQL client tools, `sqlx-cli`, and Tilt.

### 2. Start Dependencies

```bash
make start-deps
```

This starts the following Docker services:

| Service | Port | Purpose |
|---------|------|---------|
| `core-pg` (PostgreSQL) | 5433 | Main application database |
| `keycloak` | 8081 | Identity provider (OIDC) |
| `keycloak-pg` | 5437 | Keycloak database |
| `oathkeeper` | 4455 | API gateway (JWT validation) |
| `otel-agent` | 4317, 4318 | OpenTelemetry collector |

To include Dagster (data pipelines):

```bash
DAGSTER=true make start-deps
```

### 3. Run the Backend

```bash
make setup-db run-server
```

This runs database migrations and starts the Rust application server.

### 4. Run Frontend Apps

In separate terminals:

```bash
# Admin Panel
cd apps/admin-panel && pnpm dev

# Customer Portal
cd apps/customer-portal && pnpm dev
```

## Development URLs

| Service | URL |
|---------|-----|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |
| Admin GraphQL API | http://admin.localhost:4455/graphql |
| Customer GraphQL API | http://app.localhost:4455/graphql |
| Keycloak Admin Console | http://localhost:8081 |

:::info
GraphQL APIs must be accessed through Oathkeeper (port 4455) which handles JWT validation. Direct ports (5253/5254) lack authentication context and will not work properly.
:::

:::tip
If `app.localhost` doesn't resolve, add `127.0.0.1 app.localhost` and `::1 app.localhost` to your `/etc/hosts` file.
:::

## Interactive Development with Tilt

For hot-reloading of all services:

```bash
make dev-up
```

Tilt orchestrates Docker services + local app processes with live reload. Stop with:

```bash
make dev-down
```

## Common Commands

| Command | Purpose |
|---------|---------|
| `make start-deps` | Start Docker dependencies |
| `make stop-deps` | Stop Docker dependencies |
| `make reset-deps` | Clean and restart databases |
| `make check-code-rust` | Verify Rust code compiles |
| `make check-code-apps` | Lint, type-check, and build frontends |
| `cargo nextest run` | Run all Rust tests |
| `cargo nextest run -p <crate>` | Run tests for a single crate |
| `make e2e` | Run BATS end-to-end tests |
| `make sdl` | Regenerate GraphQL schemas |
| `make sqlx-prepare` | Update SQLx offline query cache |

:::warning
Prefix direct `cargo` commands with `SQLX_OFFLINE=true` to use the offline query cache instead of requiring a running database.
:::

## Database Access

Connect to the main PostgreSQL database:

```bash
psql postgres://user:password@localhost:5433/pg
```

Run migrations manually:

```bash
cargo sqlx migrate run
```

Migrations are located in `lana/app/migrations/`.

## Environment Variables

The Nix shell automatically sets key environment variables:

| Variable | Value | Purpose |
|----------|-------|---------|
| `PG_CON` | `postgres://user:password@localhost:5433/pg` | Database connection |
| `ENCRYPTION_KEY` | (dev key) | Encryption key for secrets |
| `KC_URL` | `http://localhost:8081` | Keycloak URL |
| `REALM` | (configured per realm) | Keycloak realm |
