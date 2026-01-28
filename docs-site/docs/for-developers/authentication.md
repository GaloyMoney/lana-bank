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

*[Detailed authentication documentation coming soon - will be added from technical manual]*
