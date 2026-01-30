---
id: authentication
title: Authentication
sidebar_position: 3
---

# Authentication

All Lana API requests require authentication.

## Overview

Lana uses industry-standard authentication:

- **OAuth 2.0 / OpenID Connect** for user authentication
- **API tokens** for service-to-service communication

## Authentication Methods

### User Authentication

For applications where users log in:

1. Redirect to identity provider
2. Receive authorization code
3. Exchange for access token
4. Include token in API requests

### Service Authentication

For backend integrations:

1. Obtain service credentials
2. Request access token
3. Include token in API requests

## Making Authenticated Requests

Include the access token in the Authorization header:

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id } }"}' \
  https://your-lana-instance/graphql
```

## Token Refresh

Access tokens expire. Implement token refresh to maintain sessions.

### Token Lifetimes

| Token Type | Default Lifetime |
|------------|------------------|
| Access token | 5 minutes |
| Refresh token | 30 minutes |
| Session | 8 hours |

### Refreshing Tokens

```bash
curl -X POST \
  -d "client_id=api-client" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  https://your-keycloak-server/realms/admin/protocol/openid-connect/token
```

## Security Best Practices

- Store tokens in memory when possible (not localStorage)
- Use httpOnly cookies for refresh tokens in web applications
- Clear tokens on logout
- Always use HTTPS for API requests
