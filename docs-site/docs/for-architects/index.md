---
id: index
title: Architecture Guide
sidebar_position: 1
---

# Architecture Guide

Welcome to the Lana architecture documentation. This section is for technical teams evaluating or deploying the platform.

## System Overview

Lana is a modern banking core built on:

- **Hexagonal Architecture** - Clean separation between domain logic, application services, and infrastructure
- **Event Sourcing** - Complete audit trail of all state changes
- **Domain-Driven Design** - Business logic organized around banking domain concepts
- **GraphQL APIs** - Flexible, strongly-typed API layer

## Documentation

### Functional Architecture

Comprehensive technical architecture covering:

- Application architecture and module design
- Communication patterns (event sourcing, outbox, webhooks)
- External integrations (KYC/KYB, payment gateways, regulatory systems)
- Security architecture (authentication, authorization, encryption)
- Infrastructure requirements

[View Functional Architecture](functional-architecture)

### Data Models

Entity relationship diagrams for the core databases:

- [ERD Overview](erds/) - Database schema documentation
- [Cala Ledger Schema](erds/cala) - Underlying ledger database
- [Lana Core Schema](erds/lana) - Main application database

### Deployment

Infrastructure and deployment requirements.

*[Deployment documentation coming soon - will be added from technical manual]*

### Integrations

External system integration patterns and requirements.

*[Integration documentation coming soon - will be added from technical manual]*

## Technology Stack

| Component | Technology |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Ledger | Cala (double-entry accounting) |
| Database | PostgreSQL |
| Events | Event sourcing with outbox pattern |
| Authentication | OAuth 2.0 / OpenID Connect |
