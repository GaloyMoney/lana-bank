---
id: index
title: Platform Engineering Guide
sidebar_position: 1
---

# Platform Engineering Guide

Welcome to the Lana platform engineering documentation. This section is for technical teams evaluating or deploying the platform.

## System Overview

Lana is a modern banking core built on:

- **Hexagonal Architecture** - Clean separation between domain logic, application services, and infrastructure
- **Event Sourcing** - Complete audit trail of all state changes
- **Domain-Driven Design** - Business logic organized around banking domain concepts
- **GraphQL APIs** - Flexible, strongly-typed API layer

## Documentation

### System Architecture

- [System Architecture](system-architecture) - System layers and component overview
- [Functional Architecture](functional-architecture) - Comprehensive technical architecture
- [Authentication Architecture](authentication-architecture) - Keycloak, OAuth 2.0, gateway design

### Data Pipelines

- [Data Pipelines](data-pipelines) - Meltano, dbt, BigQuery

### Data Models

- [ERD Overview](erds/) - Database schema documentation
- [Cala Ledger Schema](erds/cala) - Underlying ledger database
- [Lana Core Schema](erds/lana) - Main application database

### Deployment and Operations

- [Deployment Guide](deployment/) - Deployment overview
- [Build System](deployment/build-system) - Nix, cross-compilation, Docker
- [Development Environment](deployment/development-environment) - Local setup
- [Testing Strategy](deployment/testing-strategy) - Testing layers and tools
- [CI/CD Pipeline](deployment/ci-cd) - GitHub Actions, Concourse, releases

:::tip
Looking for domain internals, event sourcing, background jobs, or observability? See the [Internal Developer Guide](../for-internal-developers/) — those topics have moved there.
:::

## Technology Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Ledger | Cala (double-entry accounting) |
| Database | PostgreSQL |
| Events | Event sourcing with outbox pattern |
| Authentication | OAuth 2.0 / OpenID Connect |

