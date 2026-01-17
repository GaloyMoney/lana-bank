---
sidebar_position: 1
title: API Reference
description: GraphQL API documentation for Lana Bank
---

# API Reference

Lana Bank exposes two GraphQL APIs for different use cases.

## Admin API

The **Admin API** is designed for bank operators and administrators. It provides full access to all system capabilities including:

- Customer management and onboarding
- Credit facility creation, approval, and management
- Deposit and withdrawal operations
- Accounting and financial reporting
- Governance and approval workflows
- Custody and collateral management
- User and permission management
- Audit trail access

**Endpoint**: `http://admin.localhost:4455/graphql`

[View Admin API Documentation →](/api/admin)

## Customer API

The **Customer API** is designed for customer-facing applications like the Customer Portal. It provides customers with access to their own data:

- Account information and balances
- Credit facility status and history
- Deposit account operations
- KYC verification status
- Transaction history

**Endpoint**: `http://app.localhost:4455/graphql`

[View Customer API Documentation →](/api/customer)

## Domain Events

The system publishes **domain events** via the transactional outbox pattern for integration with external systems. These events cover:

- Access and user management
- Credit facility lifecycle (proposals, activation, payments, liquidations)
- Custody and wallet operations
- Customer onboarding and KYC
- Deposit and withdrawal operations
- Price updates
- Governance and approvals

[View Domain Events Documentation →](/api/events)

---

## Authentication

Both APIs require authentication via JWT tokens obtained from Keycloak. The token must be included in the `Authorization` header:

```
Authorization: Bearer <token>
```

### Using Apollo Sandbox

To use the interactive Apollo Sandbox explorer with our APIs:

1. **Obtain a JWT token** from Keycloak:
   - For **Admin API**: Log in to the Admin Panel and extract the token from browser DevTools (Network tab → any GraphQL request → Request Headers → `Authorization`)
   - For **Customer API**: Log in to the Customer Portal and extract the token similarly

2. **Open Apollo Sandbox** using one of the links above

3. **Add the Authorization header** in Apollo Sandbox:
   - Click the **Headers** tab at the bottom of the Operation panel
   - Add a new header:
     - **Key**: `Authorization`
     - **Value**: `Bearer <your-jwt-token>`

4. **Start exploring**: You can now run queries and mutations against the API

:::tip Token Expiration
JWT tokens expire after a period of time. If you receive authentication errors, obtain a fresh token by logging into the application again.
:::
