---
id: graphql-integration
title: Integración GraphQL
sidebar_position: 3
---

# Integración del Cliente GraphQL

Este documento describe cómo integrar aplicaciones cliente con las APIs GraphQL de Lana.

## Visión General

Lana expone dos APIs GraphQL:

| API | Puerto | Propósito |
|-----|--------|-----------|
| Admin API | 5253 (via Oathkeeper 4455) | Operaciones administrativas |
| Customer API | 5254 (via Oathkeeper 4455) | Operaciones de clientes |

## Arquitectura del Cliente

```
┌─────────────────────────────────────────────────────────────────┐
│                    CLIENTE GRAPHQL                              │
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │ Apollo Client   │───▶│ HTTP Transport  │                    │
│  │                 │    │                 │                    │
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
│                                │                               │
│                                ▼                               │
│               ┌────────────────┴────────────────┐              │
│               │                                 │              │
│        ┌──────▼──────┐                   ┌──────▼──────┐       │
│        │ Admin Server│                   │Customer     │       │
│        │  (GraphQL)  │                   │Server       │       │
│        └─────────────┘                   └─────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

## Configuración de Apollo Client

### Instalación

```bash
npm install @apollo/client graphql
```

### Configuración Básica

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

## Autenticación

### Flujo de Autenticación

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Usuario    │───▶│   Keycloak   │───▶│  JWT Token   │
│   Login      │    │   Auth       │    │  Recibido    │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │  GraphQL     │
                                        │  Request     │
                                        │  con Token   │
                                        └──────────────┘
```

### Configuración de Keycloak

```typescript
import Keycloak from 'keycloak-js';

const keycloak = new Keycloak({
  url: 'http://localhost:8081',
  realm: 'admin',  // o 'customer' para portal de clientes
  clientId: 'admin-panel',
});

await keycloak.init({ onLoad: 'login-required' });
```

## Generación de Código

### Configuración de Codegen

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

### Ejecutar Codegen

```bash
npx graphql-codegen
```

## Operaciones Comunes

### Queries

```graphql
# Obtener lista de clientes
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
# Crear un cliente
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

### Uso con Hooks de React

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

## Paginación

### Cursor-based Pagination

Las APIs de Lana usan paginación basada en cursor (conexiones Relay):

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

### Implementación de Infinite Scroll

```typescript
const { data, fetchMore } = useCreditFacilitiesQuery({
  variables: { first: 20 },
});

const loadMore = () => {
  if (data?.creditFacilities?.pageInfo.hasNextPage) {
    fetchMore({
      variables: {
        after: data.creditFacilities.pageInfo.endCursor,
      },
    });
  }
};
```

## Manejo de Errores

### Tipos de Errores

| Tipo | Descripción |
|------|-------------|
| Network Error | Error de conexión |
| GraphQL Error | Error en la query/mutation |
| Authentication Error | Token inválido o expirado |

### Manejo en Apollo Client

```typescript
import { onError } from '@apollo/client/link/error';

const errorLink = onError(({ graphQLErrors, networkError }) => {
  if (graphQLErrors) {
    graphQLErrors.forEach(({ message, locations, path }) => {
      console.error(`GraphQL error: ${message}`);
    });
  }
  if (networkError) {
    console.error(`Network error: ${networkError}`);
  }
});

const client = new ApolloClient({
  link: errorLink.concat(authLink.concat(httpLink)),
  cache: new InMemoryCache(),
});
```

## Caché

### Configuración de InMemoryCache

```typescript
const cache = new InMemoryCache({
  typePolicies: {
    Query: {
      fields: {
        creditFacilities: {
          keyArgs: ['status'],
          merge(existing, incoming, { args }) {
            if (!args?.after) return incoming;
            return {
              ...incoming,
              edges: [...(existing?.edges || []), ...incoming.edges],
            };
          },
        },
      },
    },
  },
});
```

## Subscriptions (Futuro)

Las subscriptions GraphQL permitirán actualizaciones en tiempo real:

```graphql
subscription OnFacilityStatusChanged($facilityId: ID!) {
  facilityStatusChanged(facilityId: $facilityId) {
    id
    status
    updatedAt
  }
}
```

## Endpoints

### Desarrollo Local

| Servicio | URL |
|----------|-----|
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |
| GraphQL Playground | http://admin.localhost:4455/graphql |

### Headers Requeridos

```
Authorization: Bearer <jwt-token>
Content-Type: application/json
```

## Documentación de APIs

- [Admin API](../apis/admin-api) - Referencia completa de la API de administración
- [Customer API](../apis/customer-api) - Referencia completa de la API de clientes

