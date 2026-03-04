---
id: graphql-development
title: GraphQL Development
sidebar_position: 5
---

# GraphQL Development

This guide covers working with Lana's GraphQL APIs during local development.

## API Endpoints

| API | Local URL | Direct Port |
|-----|-----------|-------------|
| Admin API | http://admin.localhost:4455/graphql | 5253 |
| Customer API | http://app.localhost:4455/graphql | 5254 |

:::warning
Always use the Oathkeeper URLs (port 4455) for development. Direct ports lack authentication context.
:::

## Apollo Client Setup

Both frontend apps use Apollo Client for GraphQL communication.

### Installation

```bash
npm install @apollo/client graphql
```

### Configuration

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

## Code Generation

After modifying GraphQL schemas in Rust, regenerate the SDL and client types:

### 1. Regenerate GraphQL SDL

```bash
make sdl
```

This must be run after any changes to `async-graphql` resolvers in Rust.

### 2. Generate TypeScript Types

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

## Common Patterns

### Queries with Cursor Pagination

Lana APIs use Relay-style cursor-based pagination:

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

### Mutations

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

### React Hooks Usage

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

## Error Handling

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

## API References

- [Admin API Reference](../apis/admin-api) — Complete admin operations and types
- [Customer API Reference](../apis/customer-api) — Customer-facing operations
- [Domain Events](../apis/events) — Event catalog
