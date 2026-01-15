---
sidebar_position: 1
title: API Reference
description: GraphQL API documentation for Lana Bank
---

# API Reference

Lana Bank exposes two GraphQL APIs for different use cases:

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

**Endpoint**: `/admin/graphql`

[View Admin API Documentation →](/api/admin)

## Customer API

The **Customer API** is designed for customer-facing applications like the Customer Portal. It provides customers with access to their own data:

- Account information and balances
- Credit facility status and history
- Deposit account operations
- KYC verification status
- Transaction history

**Endpoint**: `/customer/graphql`

[View Customer API Documentation →](/api/customer)

---

## Authentication

Both APIs require authentication via JWT tokens obtained from Keycloak. The token must be included in the `Authorization` header:

```
Authorization: Bearer <token>
```
