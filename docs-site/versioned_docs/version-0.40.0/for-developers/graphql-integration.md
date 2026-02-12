---
id: graphql-integration
title: GraphQL Integration
sidebar_position: 3
---

# GraphQL Client Integration

This document describes how to integrate client applications with Lana's GraphQL APIs.

## Overview

Lana exposes two GraphQL APIs:

| API | Port | Purpose |
|-----|------|---------|
| Admin API | 5253 (via Oathkeeper 4455) | Administrative operations |
| Customer API | 5254 (via Oathkeeper 4455) | Customer operations |

## Client Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    GRAPHQL CLIENT                               │
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │ Apollo Client   │───▶│ HTTP Transport  │                    │
│  └─────────────────┘    └─────────────────┘                    │
│           │                     │                               │
│           ▼                     ▼                               │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │ Cache Layer     │    │ Auth Headers    │                    │
│  │ (InMemoryCache) │    │ (JWT Token)     │                    │
│  └─────────────────┘    └─────────────────┘                    │
│                                │                               │
│                                ▼                               │
│                    ┌─────────────────┐                         │
│                    │   Oathkeeper    │                         │
│                    │   (Gateway)     │                         │
│                    └─────────────────┘                         │
└─────────────────────────────────────────────────────────────────┘
```

## Apollo Client Setup

### Installation

```bash
npm install @apollo/client graphql
```

### Basic Configuration

```typescript
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

const httpLink = createHttpLink({
  uri: 'http://admin.localhost:4455/graphql',
});

const authLink = setContext((_, { headers }) => {
  const token = localStorage.getItem('token');
  return {
    headers: {
      ...headers,
      authorization: token ? `Bearer ${token}` : '',
    },
  };
});

const client = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});
```

## Authentication

### Authentication Flow

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   User       │───▶│   Keycloak   │───▶│  JWT Token   │
│   Login      │    │   Auth       │    │  Received    │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │  GraphQL     │
                                        │  Request     │
                                        │  with Token  │
                                        └──────────────┘
```

### Keycloak Configuration

```typescript
import Keycloak from 'keycloak-js';

const keycloak = new Keycloak({
  url: 'http://localhost:8081',
  realm: 'admin',  // or 'customer' for customer portal
  clientId: 'admin-panel',
});

await keycloak.init({ onLoad: 'login-required' });
```

## Code Generation

### Codegen Configuration

```yaml
# codegen.yml
schema:
  - http://admin.localhost:4455/graphql:
      headers:
        Authorization: Bearer ${TOKEN}

documents:
  - 'src/**/*.graphql'

generates:
  src/generated/graphql.ts:
    plugins:
      - typescript
      - typescript-operations
      - typescript-react-apollo
```

### Run Codegen

```bash
npx graphql-codegen
```

## Common Operations

### Queries

```graphql
# Get customer list
query GetCustomers($first: Int, $after: String) {
  customers(first: $first, after: $after) {
    edges {
      node {
        id
        email
        status
        createdAt
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

### Mutations

```graphql
# Create a customer
mutation CreateCustomer($input: CustomerCreateInput!) {
  customerCreate(input: $input) {
    customer {
      id
      email
      status
    }
  }
}
```

### Usage with React Hooks

```typescript
import { useGetCustomersQuery, useCreateCustomerMutation } from './generated/graphql';

function CustomerList() {
  const { data, loading, error } = useGetCustomersQuery({
    variables: { first: 10 },
  });

  const [createCustomer] = useCreateCustomerMutation();

  if (loading) return <Loading />;
  if (error) return <Error message={error.message} />;

  return (
    <ul>
      {data?.customers?.edges?.map((edge) => (
        <li key={edge.node.id}>{edge.node.email}</li>
      ))}
    </ul>
  );
}
```

## Pagination

Lana APIs use cursor-based pagination (Relay connections):

```graphql
query GetFacilities($first: Int!, $after: String) {
  creditFacilities(first: $first, after: $after) {
    edges {
      cursor
      node {
        id
        status
      }
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
    }
  }
}
```

## Error Handling

### Error Types

| Type | Description |
|------|-------------|
| Network Error | Connection error |
| GraphQL Error | Query/mutation error |
| Authentication Error | Invalid or expired token |

### Handling in Apollo Client

```typescript
import { onError } from '@apollo/client/link/error';

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (graphQLErrors) {
    graphQLErrors.forEach(({ message }) => {
      console.error(`GraphQL error: ${message}`);
    });
  }
  if (networkError) {
    console.error(`Network error: ${networkError}`);
  }
});
```

## Endpoints

### Local Development

| Service | URL |
|---------|-----|
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |

### Required Headers

```
Authorization: Bearer <jwt-token>
Content-Type: application/json
```

## API Documentation

- [Admin API](../apis/admin-api/api-reference.mdx) - Complete admin API reference
- [Customer API](../apis/customer-api/api-reference.mdx) - Complete customer API reference

