---
id: admin-panel
title: Admin Panel
sidebar_position: 2
---

# Admin Panel

This document describes the Admin Panel architecture and development.

## Purpose

The Admin Panel is the main interface for bank staff:

- Customer management
- Credit administration
- Deposit and withdrawal operations
- Approvals and governance
- Financial reports

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    ADMIN PANEL                                  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Next.js App Router                     │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Dashboard  │  │ Customers  │  │  Credit    │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  │  ┌────────────┐  ┌────────────┐  ┌────────────┐         │  │
│  │  │ Deposits   │  │ Approvals  │  │  Reports   │         │  │
│  │  └────────────┘  └────────────┘  └────────────┘         │  │
│  └──────────────────────────────────────────────────────────┘  │
│                              │                                  │
│                              ▼                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                    Apollo Client                          │  │
│  │                 (Admin GraphQL API)                       │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## Project Structure

```
apps/admin-panel/
├── app/
│   ├── layout.tsx           # Main layout
│   ├── page.tsx             # Dashboard
│   ├── customers/           # Customer module
│   ├── credit/              # Credit module
│   ├── deposits/            # Deposit module
│   ├── approvals/           # Approval module
│   └── reports/             # Reports module
├── components/
│   ├── layout/              # Layout components
│   ├── customers/           # Customer components
│   ├── credit/              # Credit components
│   └── shared/              # Shared components
└── lib/
    ├── apollo.ts            # Apollo configuration
    └── keycloak.ts          # Keycloak configuration
```

## Authentication

### Keycloak Configuration

```typescript
import Keycloak from 'keycloak-js';

export const keycloak = new Keycloak({
  url: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
  realm: 'admin',
  clientId: 'admin-panel',
});
```

### Route Protection

```typescript
export function ProtectedRoute({ children, requiredRole }) {
  const { isAuthenticated, hasRole } = useAuth();

  if (!isAuthenticated) {
    return <LoginRedirect />;
  }

  if (requiredRole && !hasRole(requiredRole)) {
    return <AccessDenied />;
  }

  return children;
}
```

## Development

### Commands

```bash
# Development
pnpm dev

# Production build
pnpm build

# Lint
pnpm lint

# Generate GraphQL types
pnpm codegen
```

### Environment Variables

```env
NEXT_PUBLIC_GRAPHQL_URL=http://admin.localhost:4455/graphql
NEXT_PUBLIC_KEYCLOAK_URL=http://localhost:8081
NEXT_PUBLIC_KEYCLOAK_REALM=admin
NEXT_PUBLIC_KEYCLOAK_CLIENT_ID=admin-panel
```

