---
id: system-architecture
title: System Architecture
sidebar_position: 2
---

# System Architecture

This document describes Lana's system architecture, including layers, components, and data flow.

```mermaid
graph TD
    subgraph Client["Client Layer"]
        CP["Customer Portal<br/>(Next.js)"]
        AP["Admin Panel<br/>(Next.js)"]
    end

    subgraph Gateway["API Gateway Layer"]
        OAT["Oathkeeper<br/>(Port 4455)"]
        KC["Keycloak<br/>(Port 8081)"]
    end

    subgraph App["Application Layer (Rust)"]
        CS["customer-server<br/>(GraphQL API)<br/>Port 5254"]
        AS["admin-server<br/>(GraphQL API)<br/>Port 5253"]
        LA["lana-app<br/>(Business Logic Layer)"]
    end

    subgraph Domain["Domain Layer"]
        CC["core-credit"]
        CD["core-deposit"]
        CCU["core-customer"]
        CA["core-accounting"]
        GOV["governance"]
        CUS["core-custody"]
    end

    subgraph Infra["Infrastructure Layer"]
        PG["PostgreSQL"]
        CALA["cala-ledger"]
        EXT["External APIs<br/>(BitGo, Sumsub)"]
    end

    CP --> OAT
    AP --> OAT
    OAT --> CS
    OAT --> AS
    CS --> LA
    AS --> LA
    LA --> CC
    LA --> CD
    LA --> CCU
    LA --> CA
    LA --> GOV
    LA --> CUS
    CC --> PG
    CD --> PG
    CCU --> PG
    CA --> CALA
    CUS --> EXT
    CALA --> PG
```

## System Layer Overview

Lana follows a layered architecture that separates concerns and enables maintainability:

```mermaid
graph TD
    subgraph ClientLayer["Client Layer"]
        AP["Admin Panel<br/>(Next.js)"]
        CPO["Customer Portal<br/>(Next.js)"]
        EAPI["External APIs"]
    end

    subgraph GatewayLayer["API Gateway Layer"]
        OAT["Oathkeeper<br/>(Port 4455)"]
        KC["Keycloak<br/>(Port 8081)"]
    end

    subgraph AppLayer["Application Layer"]
        ASRV["admin-server<br/>(GraphQL)"]
        CSRV["customer-server<br/>(GraphQL)"]
        LCLI["lana-cli"]
        LAPP["lana-app<br/>(Business Logic Orchestrator)"]
        ASRV --> LAPP
        CSRV --> LAPP
        LCLI --> LAPP
    end

    subgraph DomainLayer["Domain Layer"]
        CUST["Customer"]
        CRED["Credit"]
        DEP["Deposit"]
        GOV["Governance"]
        ACCT["Accounting"]
    end

    subgraph InfraLayer["Infrastructure Layer"]
        PG["PostgreSQL"]
        CALA["Cala Ledger"]
        EXT["External APIs<br/>(BitGo, Sumsub)"]
    end

    ClientLayer --> GatewayLayer
    GatewayLayer --> AppLayer
    LAPP --> DomainLayer
    DomainLayer --> InfraLayer
```

## Client Layer

### Admin Panel

Web application for bank operations:
- Customer management
- Credit administration
- Financial reports
- Configuration

### Customer Portal

Client-facing interface:
- Account view
- Credit requests
- Transaction history
- Documents

## API Gateway Layer

### Oathkeeper (Port 4455)

Handles JWT validation and request routing:
- Validates tokens issued by Keycloak
- Routes requests to appropriate servers
- Enforces authentication policies

### Keycloak (Port 8081)

Identity and access management:
- Two realms: `admin` and `customer`
- OAuth 2.0 / OpenID Connect
- User authentication and session management

## Application Layer

```mermaid
graph TD
    subgraph CLI["lana-cli Process"]
        MAIN["main()<br/>lana/cli/src/lib.rs:64-105"]
        RUNCMD["run_cmd()<br/>lana/cli/src/lib.rs:154-254"]
        MAIN --> RUNCMD
    end

    RUNCMD -->|"tokio::spawn"| ASRUN
    RUNCMD -->|"tokio::spawn"| CSRUN

    subgraph AdminServer["admin-server (Port 5253)"]
        ASRUN["run()<br/>lana/admin-server/src/lib.rs:28-70"]
        ASGQL["graphql_handler()<br/>lana/admin-server/src/lib.rs:79-136"]
        AS_SCHEMA["Schema"]
        ASRUN --> ASGQL --> AS_SCHEMA
    end

    subgraph CustServer["customer-server (Port 5254)"]
        CSRUN["run()<br/>lana/customer-server/src/lib.rs:26-66"]
        CSGQL["graphql_handler()<br/>lana/customer-server/src/lib.rs:75-132"]
        CS_SCHEMA["Schema"]
        CSRUN --> CSGQL --> CS_SCHEMA
    end

    subgraph LanaApp["lana-app"]
        INIT["LanaApp::init()<br/>lana/app/Cargo.toml:1-77"]
        AGG["Aggregates domain services"]
        INIT --> AGG
    end

    AS_SCHEMA --> INIT
    CS_SCHEMA --> INIT
```

### admin-server

GraphQL API for administrative operations:
- Full system access
- RBAC-based authorization
- Connects to admin Keycloak realm

### customer-server

GraphQL API for customer operations:
- Limited scope to customer's own data
- Simplified interface
- Connects to customer Keycloak realm

### lana-cli

Command-line tool for:
- Starting servers
- Running migrations
- Administrative tasks
- Batch operations

### lana-app

Central business logic orchestrator:
- Initializes all domain services
- Coordinates cross-domain operations
- Manages application lifecycle

## Domain Layer

Implements core business logic using Domain-Driven Design:

| Domain | Purpose |
|--------|---------|
| Customer | Customer lifecycle and KYC |
| Credit | Credit facilities and disbursements |
| Deposit | Deposit accounts and withdrawals |
| Governance | Multi-party approval workflows |
| Accounting | Financial period management |

## Infrastructure Layer

### PostgreSQL

Primary data store:
- Event storage
- Entity state
- Audit logs

### Cala Ledger

Double-entry accounting system:
- Account hierarchy
- Transaction recording
- Balance calculation

### External Integrations

- **BitGo/Komainu**: Cryptocurrency custody
- **Sumsub**: KYC verification
- **SMTP**: Email notifications

## Data Flow

### Request Processing

```mermaid
graph LR
    REQ["Client Request"] --> OAT["Oathkeeper"] --> JWT["JWT Validation"] --> GQL["GraphQL Server"] --> DOM["Domain Services"] --> CALA["Cala Ledger"] --> RESP["Response"]
```

### Event Flow

```mermaid
graph LR
    EVT["Domain Event"] --> OUT["Outbox Table"] --> PROC["Event Processor"] --> DEP["Dependent Domains"] --> NOTIF["External Notifications"]
```

## Key Architectural Decisions

### Event Sourcing

All state changes are captured as events:
- Complete audit trail
- Temporal queries
- Event replay capability

### Hexagonal Architecture

Clean separation of concerns:
- Domain logic isolated from infrastructure
- Adapter pattern for external services
- Testable business logic

### CQRS Pattern

Command Query Responsibility Segregation:
- Optimized read paths
- Separate write operations
- Eventual consistency where appropriate

