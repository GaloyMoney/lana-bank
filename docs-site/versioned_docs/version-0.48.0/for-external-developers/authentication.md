---
id: authentication
title: Authentication
sidebar_position: 3
---

# Authentication

All Lana API requests require authentication via OAuth 2.0 / OpenID Connect tokens.

## Authentication Methods

### User Authentication (Authorization Code Flow)

For applications where end users log in:

1. Redirect the user to the identity provider's authorization endpoint
2. Receive an authorization code via callback
3. Exchange the code for access and refresh tokens
4. Include the access token in API requests

### Service Authentication (Client Credentials)

For backend service-to-service integrations:

```bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

## Making Authenticated Requests

Include the access token in the `Authorization` header:

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id } }"}' \
  https://admin.your-instance.com/graphql
```

## Token Refresh

Access tokens expire and must be refreshed to maintain sessions.

### Token Lifetimes

| Token Type | Default Lifetime |
|------------|------------------|
| Access token | 5 minutes |
| Refresh token | 30 minutes |
| Session | 8 hours |

### Refreshing Tokens

```bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

## Security Best Practices

- **Token storage**: Store tokens in memory when possible, not in localStorage
- **Refresh tokens**: Use httpOnly cookies for refresh tokens in web applications
- **Logout**: Clear all tokens on logout
- **Transport**: Always use HTTPS for API requests
- **Rotation**: Implement automatic token refresh before expiry

