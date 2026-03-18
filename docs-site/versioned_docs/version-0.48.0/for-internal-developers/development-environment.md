---
id: development-environment
title: Development Environment
sidebar_position: 3
---

# Development Environment

This document describes how to set up and use the local development environment.

## Prerequisites

- Nix with flakes enabled
- Docker and Docker Compose
- Git

## Setup

### 1. Enter Nix Shell

```bash
cd lana-bank
nix develop
```

### 2. Start Dependencies

```bash
make start-deps
```

This starts:
- PostgreSQL (port 5433)
- Keycloak (port 8081)
- Oathkeeper (port 4455)

### 3. Run Migrations

```bash
cargo sqlx migrate run
```

### 4. Start Application

```bash
# Run all servers
cargo run

# Or use Tilt for interactive development
make dev-up
```

## Service URLs

| Service | URL |
|---------|-----|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |
| Keycloak | http://localhost:8081 |

## Tilt Development

Interactive development with automatic rebuilds:

```bash
# Start Tilt
make dev-up

# Open Tilt UI
# http://localhost:10350

# Stop Tilt
make dev-down
```

## Database Access

```bash
# Connect to PostgreSQL
psql -h localhost -p 5433 -U lana -d lana

# Reset database
make reset-deps
```

## Frontend Development

```bash
# Admin Panel
cd apps/admin-panel
pnpm install
pnpm dev

# Customer Portal
cd apps/customer-portal
pnpm install
pnpm dev
```

## Environment Variables

```bash
# .env.local
DATABASE_URL=postgres://lana:lana@localhost:5433/lana
KEYCLOAK_URL=http://localhost:8081
OATHKEEPER_URL=http://localhost:4455
```

## Keycloak Credentials

| Realm | User | Password |
|-------|------|----------|
| admin | admin | admin |
| customer | test@test.com | test |

## Common Issues

### Port Conflicts

```bash
# Check what's using a port
lsof -i :5433

# Kill process
kill -9 <PID>
```

### Database Reset

```bash
make reset-deps
cargo sqlx migrate run
```

### Cache Issues

```bash
# Clear Rust cache
cargo clean

# Clear pnpm cache
pnpm store prune
```

