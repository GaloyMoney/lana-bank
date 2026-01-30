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

Lana uses OAuth 2.0 / OpenID Connect for authentication. Request a token from your Keycloak server:

```bash
curl -X POST \
  -d "client_id=api-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://your-keycloak-server/realms/admin/protocol/openid-connect/token
```

Extract the `access_token` from the response for use in API requests.

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
