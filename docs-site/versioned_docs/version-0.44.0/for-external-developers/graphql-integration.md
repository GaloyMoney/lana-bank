---
id: graphql-integration
title: GraphQL Integration
sidebar_position: 4
---

# GraphQL Integration

This guide covers integrating client applications with Lana's GraphQL APIs.

## API Endpoints

Lana exposes two GraphQL APIs:

| API | Purpose | Typical URL |
|-----|---------|-------------|
| **Admin API** | Administrative operations — customers, credit, accounting | `https://admin.your-instance.com/graphql` |
| **Customer API** | Customer-facing operations — account info, facility status | `https://app.your-instance.com/graphql` |

## Making Requests

### With curl

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ customers(first: 10) { edges { node { id email } } } }"}' \
  https://admin.your-instance.com/graphql
```

### With JavaScript (Apollo Client)

```bash
npm install @apollo/client graphql
```

```typescript
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

const httpLink = createHttpLink({
  uri: 'https://admin.your-instance.com/graphql',
});

const authLink = setContext((_, { headers }) => ({
  headers: {
    ...headers,
    authorization: `Bearer ${getAccessToken()}`,
  },
}));

const client = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});
```

### With Python

```python
import requests

url = "https://admin.your-instance.com/graphql"
headers = {
    "Authorization": f"Bearer {access_token}",
    "Content-Type": "application/json",
}

query = """
query {
  customers(first: 10) {
    edges {
      node {
        id
        email
        status
      }
    }
  }
}
"""

response = requests.post(url, json={"query": query}, headers=headers)
data = response.json()
```

## Pagination

Lana APIs use cursor-based pagination following the Relay specification:

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
      hasPreviousPage
      startCursor
      endCursor
    }
  }
}
```

To fetch the next page, pass the `endCursor` value as the `after` parameter.

## Error Handling

GraphQL errors are returned in the `errors` array of the response:

```json
{
  "data": null,
  "errors": [
    {
      "message": "Not authorized",
      "path": ["customerCreate"],
      "extensions": {
        "code": "FORBIDDEN"
      }
    }
  ]
}
```

| Error Type | Description | Action |
|------------|-------------|--------|
| `FORBIDDEN` | Insufficient permissions | Check API credentials and role |
| `UNAUTHENTICATED` | Invalid or expired token | Refresh the access token |
| `BAD_USER_INPUT` | Invalid input data | Check the request parameters |
| `INTERNAL_SERVER_ERROR` | Server-side error | Retry with exponential backoff |

## Required Headers

```
Authorization: Bearer <access-token>
Content-Type: application/json
```

## API References

- [Admin API Reference](../apis/admin-api/api-reference.mdx) — Complete admin operations and types
- [Customer API Reference](../apis/customer-api/api-reference.mdx) — Complete customer operations
