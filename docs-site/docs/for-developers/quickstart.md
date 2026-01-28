---
id: quickstart
title: Quickstart
sidebar_position: 2
---

# Developer Quickstart

Get started with the Lana APIs in minutes.

## Prerequisites

- API credentials (contact your Lana administrator)
- A GraphQL client (curl, Postman, or language-specific client)

## Your First API Call

### 1. Obtain Authentication Token

*[Authentication setup details coming soon]*

### 2. Query the Admin API

```graphql
query {
  me {
    id
    email
  }
}
```

### 3. Explore the Schema

Use GraphQL introspection or browse the [Admin API Reference](admin-api/) to discover available operations.

## Next Steps

- [Admin API Reference](admin-api/) - Full API documentation
- [Customer API Reference](customer-api/) - Customer-facing operations
- [Domain Events](events/) - Subscribe to system events
- [Authentication](authentication) - Detailed auth setup

*Full quickstart guide coming soon.*
