---
id: index
title: Customer Management
sidebar_position: 1
---

# Customer Management

The Customer Management system covers the complete customer lifecycle, from initial registration and KYC verification to active account status.

```mermaid
graph TD
    subgraph Frontend["Customer Portal Structure"]
        ROOT_PAGE["app/page.tsx<br/>Root Page"]
        LAYOUT["app/layout.tsx<br/>Main Layout"]
    end

    subgraph SharedComponents["Shared Components"]
        STORYBOOK["@storybook<br/>UI Library"]
        THEME["Theme Provider<br/>next/themes"]
        CSS["Tailwind CSS<br/>(CSS Definition)"]
    end

    subgraph GQL["GraphQL Integration"]
        GQL_COMP["components/*<br/>UI Components"]
        GQL_API["GraphQL API<br/>customer-server"]
    end

    subgraph Auth["Authentication Flow"]
        AUTH_CFG["Auth.ts<br/>NextAuth Config"]
        AUTH_MIDDLEWARE["appAuthProvider.tsx<br/>Auth API Route"]
        OIDC["OIDC Provider<br/>KeycloakProvider"]
        JWT["JWT Session<br/>JwtLibrary"]
    end

    STORYBOOK --> CSS
    THEME --> CSS
    ROOT_PAGE --> GQL_COMP
    GQL_COMP --> GQL_API
    AUTH_CFG --> OIDC
    AUTH_MIDDLEWARE --> AUTH_CFG
    OIDC --> JWT
```

## System Components

| Component | Module | Purpose |
|-----------|--------|---------|
| Customer Management | core-customer | Persistence, profiles, and documents |
| KYC Processing | core-applicant | Sumsub integration |
| User Onboarding | user-onboarding | Keycloak provisioning |

## Customer Lifecycle

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Initial   │───▶│     KYC     │───▶│   Deposit   │───▶│   Active    │
│Registration │    │ Verification│    │   Account   │    │  Customer   │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

The system establishes the fundamental identity layer required before customers can access financial products:

1. **Initial registration**: Basic customer data capture
2. **KYC verification**: Identity validation through Sumsub
3. **Deposit account**: Automatic creation after KYC approval
4. **Product access**: Credit facilities enabled

## Customer Types

| Type | Description | Accounting Treatment |
|------|-------------|---------------------|
| INDIVIDUAL | Natural person | Individual accounts |
| GOVERNMENT_ENTITY | Government organization | Government accounts |
| PRIVATE_COMPANY | Private corporation | Business accounts |
| BANK | Banking institution | Interbank accounts |
| FINANCIAL_INSTITUTION | Financial services company | Institutional accounts |
| FOREIGN_AGENCY_OR_SUBSIDIARY | Foreign agency/subsidiary | Foreign accounts |
| NON_DOMICILED_COMPANY | Non-domiciled corporation | Non-resident accounts |

## Customer Status

| Status | Description |
|--------|-------------|
| ACTIVE | Customer can perform operations |
| INACTIVE | Account is inactive |
| SUSPENDED | Account is suspended |

## Related Documentation

- [Onboarding Process](onboarding) - Complete onboarding flow
- [Document Management](documents) - Customer document handling

