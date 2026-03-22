---
id: authentication-architecture
title: Authentication and Authorization
sidebar_position: 4
---

# Authentication and Authorization Architecture

This document describes Lana's authentication and authorization system, including Keycloak integration, OAuth 2.0 flows, and RBAC implementation.

```mermaid
graph TD
    ROOT["localhost:4455"]
    ROOT --> R1["admin.localhost:4455"]
    ROOT --> R2["app.localhost:4455"]
    ROOT --> R3["keycloak"]
    ROOT --> R4["oathkeeper"]
    ROOT --> R5["admin-server"]
    ROOT --> R6["customer-server"]
    ROOT --> R7["graphql /admin"]
    ROOT --> R8["graphql /customer"]
    ROOT --> R9["static assets"]
    ROOT --> R10["health checks"]
```

## Overview

Lana's security architecture consists of:

- **Keycloak**: Identity provider and token issuer
- **Oathkeeper**: API gateway for JWT validation
- **Casbin**: Role-based access control engine
- **Audit System**: Authorization decision logging

## Architecture

```mermaid
graph TD
    CLIENT["Client<br/>(Browser/App)"] --> LOGIN["Login Form"]
    LOGIN --> KC["Keycloak<br/>(Auth Server)"]
    KC -->|"JWT Token"| OAT["Oathkeeper<br/>(Gateway)"]
    OAT -->|"Validated Request"| GQL["GraphQL Server"]
    GQL --> CASBIN["Casbin<br/>(RBAC Check)"]
```

## Keycloak Configuration

### Realms

| Realm | Purpose | Users |
|-------|---------|-------|
| admin | Administrative access | Bank employees |
| customer | Customer access | Bank customers |

### Clients

Each realm has configured clients:

- **admin-panel**: Web application for administrators
- **customer-portal**: Web application for customers
- **api-client**: For programmatic API access

### Token Configuration

JWT tokens include:
- User ID (`sub`)
- Realm roles
- Client roles
- Token expiration

## Oathkeeper Gateway

Oathkeeper validates incoming requests at port 4455:

### Rules Configuration

```yaml
# Admin API rule
- id: admin-api
  upstream:
    url: http://admin-server:5253
  match:
    url: http://admin.localhost:4455/graphql
    methods: [POST]
  authenticators:
    - handler: jwt
      config:
        jwks_urls:
          - http://keycloak:8081/realms/admin/protocol/openid-connect/certs
  authorizer:
    handler: allow
```

### Request Mutation

```mermaid
sequenceDiagram
    participant Client as Admin Panel<br/>(Browser)
    participant OAT as Oathkeeper<br/>(Port 4455)
    participant KC as Keycloak<br/>(OIDC)
    participant AS as admin-server<br/>(Port 5253)
    participant LA as lana-app<br/>(Business Logic)
    participant CASBIN as Casbin<br/>(RBAC)
    participant PG as PostgreSQL
    participant OUT as Outbox<br/>(Event Publishing)

    Client->>OAT: POST /admin/graphql<br/>Authorization: Bearer JWT
    OAT->>KC: Validate JWT via JWKS
    KC-->>OAT: Token valid
    OAT->>AS: Forward request<br/>X-User-Id, X-User-Email headers
    AS->>LA: Extract context, execute resolver
    LA->>CASBIN: Check permission
    CASBIN->>PG: Query roles & permissions
    PG-->>CASBIN: Role data
    CASBIN-->>LA: Permission granted
    LA->>PG: Execute business operation
    PG-->>LA: Result
    LA->>OUT: Publish domain events
    OUT-->>LA: Events queued
    LA-->>AS: GraphQL response
    AS-->>OAT: Response
    OAT-->>Client: JSON response
```

Validated requests include:
- `X-Auth-Subject`: User ID
- `X-Auth-Roles`: User roles

## Role-Based Access Control

### Casbin Model

```ini
[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
```

### Permission Structure

| Role | Permissions |
|------|-------------|
| SUPERUSER | All permissions |
| BANK_MANAGER | Customer, credit, reports management |
| CREDIT_OFFICER | Credit facility operations |
| TELLER | Basic deposit/withdrawal operations |

### GraphQL Authorization

```rust
#[derive(SimpleObject)]
pub struct Query;

#[Object]
impl Query {
    #[graphql(guard = "RoleGuard::new(Permission::CustomerRead)")]
    async fn customer(&self, ctx: &Context<'_>, id: ID) -> Result<Customer> {
        // Implementation
    }
}
```

## Permission Hierarchy

```mermaid
graph TD
    SU["SUPERUSER"] --> BM["BANK_MANAGER"]
    SU --> ADM["ADMIN"]
    BM --> CO["CREDIT OFFICER"]
    BM --> CS["CUSTOMER SERVICE"]
    ADM --> RV["REPORT VIEWER"]
    ADM --> CA["CONFIG ADMIN"]
```

## Audit Logging

All authorization decisions are logged:

```rust
pub struct AuthorizationAudit {
    timestamp: DateTime<Utc>,
    subject: SubjectId,
    object: String,
    action: String,
    decision: Decision,
    reason: Option<String>,
}
```

### Audit Query

```graphql
query GetAuthorizationAudits($filter: AuditFilter!) {
  authorizationAudits(filter: $filter) {
    edges {
      node {
        timestamp
        subject
        action
        decision
      }
    }
  }
}
```

## Session Management

### Token Refresh

Tokens have configurable lifetimes:
- Access token: 5 minutes
- Refresh token: 30 minutes
- Session: 8 hours

### Logout

```typescript
// Client-side logout
keycloak.logout({
  redirectUri: window.location.origin,
});
```

## Security Best Practices

### Token Storage

- Store tokens in memory (not localStorage)
- Use httpOnly cookies for refresh tokens
- Clear tokens on logout

### CORS Configuration

- Restrict allowed origins
- Validate Referer headers
- Use strict Content-Type checking

### Rate Limiting

- Implement per-user rate limits
- Monitor for unusual patterns
- Block after failed authentication attempts

