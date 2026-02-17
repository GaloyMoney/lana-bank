---
id: index
title: Developer Guide
sidebar_position: 1
---

# Developer Guide

Welcome to the Lana developer documentation. This section covers everything you need to integrate with Lana's APIs and develop frontend applications.

## APIs Overview

Lana provides two GraphQL APIs:

| API | Purpose | Audience |
|-----|---------|----------|
| **[Admin API](../apis/admin-api/)** | Full system management - customers, credit, accounting, configuration | Internal systems, back-office applications |
| **[Customer API](../apis/customer-api/)** | Customer-facing operations - account info, facility status | Customer portals, mobile apps |

## Key Concepts

### GraphQL

Both APIs use GraphQL, providing:
- Strongly typed schemas
- Flexible queries - request exactly the data you need
- Real-time subscriptions for live updates

### Authentication

All API requests require authentication. See [Authentication](authentication) for setup details.

### Events

Lana uses event sourcing. You can subscribe to [Domain Events](../apis/events/) for real-time notifications of system changes.

## Client Integration

- [GraphQL Integration](graphql-integration) - Apollo Client setup, authentication, and usage patterns

## Frontend Development

Lana's frontend applications are built with Next.js and React:

- [Frontend Applications](frontend/) - Frontend stack overview
- [Admin Panel](frontend/admin-panel) - Admin panel architecture
- [Customer Portal](frontend/customer-portal) - Customer portal architecture
- [Shared Components](frontend/shared-components) - UI library and utilities
- [Credit UI](frontend/credit-ui) - Credit facility management

## API References

- [Admin API Reference](../apis/admin-api/) - Full admin operations and types
- [Customer API Reference](../apis/customer-api/) - Customer-facing operations
- [Domain Events](../apis/events/) - Event catalog and webhook integration

## Local Development

### Development URLs

| Service | URL |
|---------|-----|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |
| Keycloak | http://localhost:8081 |

### Useful Commands

```bash
# Start dependencies
make start-deps

# Frontend development
cd apps/admin-panel && pnpm dev
cd apps/customer-portal && pnpm dev

# Generate GraphQL types
pnpm codegen
```

