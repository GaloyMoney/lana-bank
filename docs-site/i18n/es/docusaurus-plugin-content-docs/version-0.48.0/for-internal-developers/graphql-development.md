---
id: graphql-development
title: Desarrollo con GraphQL
sidebar_position: 5
---

# Desarrollo con GraphQL

Esta guía cubre el trabajo con las APIs GraphQL de Lana durante el desarrollo local.

## Endpoints de la API

| API | URL Local | Puerto Directo |
|-----|-----------|----------------|
| API de Administración | http://admin.localhost:4455/graphql | 5253 |
| API de Cliente | http://app.localhost:4455/graphql | 5254 |

:::warning
Usa siempre las URLs de Oathkeeper (puerto 4455) para desarrollo. Los puertos directos carecen de contexto de autenticación.
:::

## Configuración de Apollo Client

Ambas aplicaciones frontend utilizan Apollo Client para la comunicación GraphQL.

### Instalación

```bash
npm install @apollo/client graphql
```

### Configuración

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

## Generación de Código

Después de modificar los esquemas GraphQL en Rust, regenera el SDL y los tipos del cliente:

### 1. Regenerar el SDL de GraphQL

```bash
make sdl
```

Esto debe ejecutarse después de cualquier cambio en los resolvers de `async-graphql` en Rust.

### 2. Generar Tipos de TypeScript

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

```bash
pnpm codegen
```

## Patrones Comunes

### Consultas con Paginación por Cursor

Las APIs de Lana utilizan paginación basada en cursores al estilo Relay:

```graphql
query GetCustomers($first: Int!, $after: String) {
  customers(first: $first, after: $after) {
    edges {
      cursor
      node {
        id
        email
        status
      }
    }
    pageInfo {
      hasNextPage
      endCursor
    }
  }
}
```

### Mutaciones

```graphql
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

### Uso de React Hooks

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

## Manejo de Errores

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

## Referencias de la API

- [Referencia de la API de Administración](../apis/admin-api) — Operaciones y tipos administrativos completos
- [Referencia de la API de Cliente](../apis/customer-api) — Operaciones orientadas al cliente
- [Eventos de Dominio](../apis/events/events.md) — Catálogo de eventos
