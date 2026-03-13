---
id: quickstart
title: Quickstart
sidebar_position: 2
---

# Developer Quickstart

Get started with the Lana APIs in minutes.

## Prerequisites

- API credentials provided by your Lana administrator
- A GraphQL client (curl, Postman, or a language-specific client library)
- Your Lana instance URL (e.g., `https://admin.your-instance.com`)

## Step 1: Obtain an Authentication Token

Lana uses OAuth 2.0 / OpenID Connect for authentication. Request a token from the identity provider:

```bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

Extract the `access_token` from the JSON response.

## Step 2: Make Your First Query

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id email } }"}' \
  https://admin.your-instance.com/graphql
```

## Step 3: Explore the Schema

Use GraphQL introspection or browse the API references to discover available operations:

- [Admin API Reference](../apis/admin-api/api-reference.mdx) — Full administrative operations
- [Customer API Reference](../apis/customer-api/api-reference.mdx) — Customer-facing operations

## Next Steps

- [Authentication](authentication) — Token management and refresh flows
- [GraphQL Integration](graphql-integration) — Client library setup and common patterns
- [Real-time Subscriptions](realtime-subscriptions) — Subscribe to real-time event notifications
- [Domain Events](../apis/events/events.md) — Event catalog
