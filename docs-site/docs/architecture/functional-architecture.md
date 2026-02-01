---
id: functional-architecture
title: Functional Architecture
sidebar_label: Functional Architecture
sidebar_position: 1
description: Technical documentation of Lana Bank's functional architecture, including application architecture, integrations, security, and infrastructure.
---

# Lana Bank Functional Architecture

## Table of Contents

1. [General Description](#1-general-description)
2. [Application Architecture](#2-application-architecture)
3. [Communication Flows](#3-communication-flows)
4. [Integrations with External Systems](#4-integrations-with-external-systems)
5. [Authentication and Security Flows](#5-authentication-and-security-flows)
6. [Network Segmentation by Environment](#6-network-segmentation-by-environment)
7. [Security Zones](#7-security-zones)
8. [Audit](#8-audit)
9. [Bitcoin-Backed Loan Flow](#9-bitcoin-backed-loan-flow)
10. [Portability and Vendor Lock-in](#10-portability-and-vendor-lock-in)
11. [Servers / Instances](#11-servers--instances)
12. [Operating Systems](#12-operating-systems)
13. [Databases](#13-databases)
14. [Middleware / Integration](#14-middleware--integration)
15. [External Services](#15-external-services)

---

## 1. General Description

This document describes the logical architecture of Lana Bank, including the internal architecture of the application, integrations with external systems, authentication and security flows, network segmentation by environment, and security zones.

### 1.1 General Overview

Lana Bank is a banking core application specialized in **Bitcoin-backed lending**. The architecture follows the principles of **Domain-Driven Design (DDD)** and **Hexagonal Architecture**, clearly separating the domain, application, and infrastructure layers.

The backend is developed in Rust, using PostgreSQL as the main database and **Cala Ledger** as the double-entry accounting engine with strong consistency guarantees. The web frontends are built with Next.js and TypeScript, consuming GraphQL APIs exposed by the backend. For reporting and analytics, there is a data pipeline based on Meltano that extracts information to BigQuery, where data is transformed with dbt.

---

## 2. Application Architecture

### 2.1 Banking Core Modules

The core modules implement the bank's business logic, following Event Sourcing principles where each entity maintains its state as a sequence of immutable events.

#### 2.1.1 Credit

The credit module is the heart of the system, managing the complete lifecycle of Bitcoin-backed loans. A credit facility goes through a well-defined lifecycle that begins when an operator creates a **CreditFacilityProposal** for a customer. This proposal automatically enters an approval process managed by the governance module; members of the assigned committee must vote to approve it.

Once approved, the proposal transforms into a **PendingCreditFacility**. At this stage, the customer must deposit the required Bitcoin collateral. If the facility has an assigned custodian, custodian webhooks automatically keep the collateral balance synchronized. If there is no custodian (manual mode), an operator can update the collateral directly. The system continuously monitors the collateralization ratio (CVL - Collateral Value to Loan) comparing it with the current Bitcoin price.

The facility is automatically activated when the CVL reaches the initial threshold configured in the terms. **TermValues** define all loan parameters: the annual interest rate, duration (classified as short or long term depending on whether it exceeds 12 months), interest accrual intervals (daily or monthly), the initial commission (one-time fee), and three critical CVL thresholds that must maintain a strict hierarchy: the initial CVL must be greater than the margin call CVL, which in turn must be greater than the liquidation CVL. The disbursement policy is also configured, which can be single or multiple.

With the **CreditFacility** active, the customer can request **Disbursals**. Each disbursement goes through its own approval process. When executed, the funds are credited to the customer's deposit account and an **Obligation** is created representing the debt. Obligations have a state cycle: they begin as "not yet due", move to "due" on the due date, can become "overdue" if not paid on time, enter "liquidation" if the delinquency persists, and finally be marked as "defaulted".

The system executes periodic jobs for interest accrual. **InterestAccrualCycles** calculate interest according to configured intervals and generate new obligations for accrued interest. When the customer makes a **Payment**, the system automatically allocates funds to pending obligations in priority order through **PaymentAllocation**, typically prioritizing the oldest obligations and interest over principal.

If the CVL falls below the margin call threshold, the facility enters an alert state. If it falls below the liquidation threshold, a **LiquidationProcess** is initiated where the bank can execute the collateral to recover the debt. The system implements a hysteresis buffer to avoid frequent oscillations between states when the CVL is near the thresholds.

#### 2.1.2 Deposit

The deposits module manages the accounts where customers maintain their funds in USD. When a **DepositAccount** is created for a customer, the system automatically generates the corresponding accounting accounts in the ledger. The accounting categorization depends on the customer type: accounts for individuals, government entities, private companies, banks, financial institutions, and non-domiciled companies are grouped under different nodes of the chart of accounts.

**Deposits** represent fund inflows to the account and are recorded immediately. **Withdrawals** follow a more controlled flow: when initiated, funds are reserved in accounting and an approval process is created. The assigned committee must approve the withdrawal before it executes. If approved, funds leave the account; if rejected or cancelled, the reserve is reversed. There is also the possibility of reversing already recorded deposits when necessary.

Accounts can be in different states that affect permitted operations. An **active** account allows all normal operations. A **frozen** account prevents new operations but keeps the balance visible; this is useful for compliance situations where funds need to be temporarily blocked. A **closed** account is permanent and is only permitted if the balance is zero. The module also supports bulk updating of the state of all accounts for a customer, for example when their KYC verification changes.

The account history can be consulted through the ledger, showing all transactions that have affected the balance. The module calculates available balance considering pending approval withdrawals.

#### 2.1.3 Customer

This module manages information about the bank's customers and is fundamental for regulatory compliance. Each customer is created with a specific type that determines their accounting and regulatory treatment: **Individual** for natural persons, **GovernmentEntity** for government entities, **PrivateCompany** for private companies, **Bank** for banks, **FinancialInstitution** for other financial institutions, **ForeignAgencyOrSubsidiary** for foreign agencies, and **NonDomiciledCompany** for non-domiciled companies.

The KYC verification process integrates with SumSub. A customer begins in **PendingVerification** state. When SumSub notifies via webhook that verification was successful, the customer moves to **Verified** with a KYC level (Basic or Advanced). If verification fails, they remain in **Rejected** state. The system can be configured to require verification before allowing the creation of deposit accounts or credit facilities.

The module manages documents associated with the customer, storing them in the cloud and allowing generation of temporary download links. Documents can be archived or deleted as needed.

For compliance with inactive account regulations, the system tracks the last activity of each customer. A periodic job automatically classifies customers according to their activity: **Active** if they have had recent activity (less than one year), **Inactive** if they have been between one and ten years without activity, and **Suspended** if they exceed ten years. This classification can affect the state of their deposit accounts.

#### 2.1.4 Custody

The custody module provides an abstraction over multiple Bitcoin custody providers, allowing the bank to work with different custodians according to their operational and regulatory needs. The system is designed with a plugin pattern where each **Custodian** implements a common interface. Currently **BitGo** and **Komainu** are implemented, but the architecture allows adding new custodians without modifying the rest of the system.

In each deployment, multiple custodians can be configured and activated simultaneously. When a credit facility is created, you can specify which custodian will manage that particular facility's collateral. This allows, for example, using different custodians for different customer segments or jurisdictions.

Each custodian manages **Wallets** that are assigned to credit facilities to receive Bitcoin collateral. Custodians notify the system about changes in wallet balances through webhooks. When a notification arrives, the system updates the **Collateral** associated with the corresponding facility and recalculates the CVL. This automatic synchronization is critical for maintaining an accurate view of risk in real time.

Custodian webhooks are received at provider-specific endpoints and are cryptographically validated before processing. The configuration for each custodian includes the necessary API credentials and keys to verify webhook authenticity. Sensitive keys are stored encrypted.

#### 2.1.5 Accounting

The accounting module implements a complete double-entry accounting system, fundamental for any regulated financial institution. It uses **Cala Ledger** as the underlying engine, a specialized Rust crate that provides predefined transaction templates and ACID consistency guarantees for all accounting operations.

The **ChartOfAccounts** defines the bank's hierarchical account structure. It can be imported from CSV files and supports a tree structure with multiple levels. Each tree node can be an individual account or a group that aggregates its children's accounts. The chart of accounts integrates with other modules: customer deposit accounts, credit facilities, and collateral accounts are automatically created as children of appropriate nodes according to customer and product type.

Each **LedgerAccount** has a normal balance type (debit or credit) and can maintain balances in multiple currencies (USD and BTC). **LedgerTransactions** represent accounting movements that always maintain balance: total debits equal total credits. The system automatically records transactions for each business operation: deposits, withdrawals, disbursements, loan payments, interest accrual, and collateral updates.

For financial reports, the module generates the **TrialBalance** which lists all accounts with their debit and credit balances, useful for verifying that the books balance. The **BalanceSheet** presents the bank's financial position organizing assets, liabilities, and equity. The **ProfitAndLoss** shows income (mainly loan interest) minus expenses to calculate the period's result.

The system supports multiple **FiscalYears** and allows querying balances and reports for specific date ranges. It also allows **ManualTransactions** for accounting adjustments that do not originate from automated system operations.

#### 2.1.6 Governance

The governance system provides a flexible framework for implementing multi-signature approval flows on sensitive operations. It is designed to adapt to different organizational structures and regulatory requirements.

**Committees** represent groups of people authorized to make decisions about certain types of operations. A committee can have any number of members, typically system users with specific roles. The same user can belong to multiple committees.

**Policies** define the approval rules for each type of process. A policy specifies which committee is responsible for approving that type of operation and what the required threshold is: the minimum number of affirmative votes needed to approve. For example, a policy for disbursement approval could require 2 of 3 members of the credit committee.

When an operation requiring approval is initiated, the system automatically creates an **ApprovalProcess** linked to the corresponding policy. The process begins in pending state and records committee member votes. A member can vote to approve or to deny (with a mandatory reason). When the approval threshold is reached, the process is marked as approved and an **ApprovalProcessConcluded** event is emitted. If any member denies, the process immediately terminates as rejected.

Approval process conclusion events are consumed by jobs that execute the approved operation or handle the rejection. This design decouples the approval flow from execution, allowing approvals to be processed asynchronously.

#### 2.1.7 Access

The access module implements role-based access control (RBAC) for all system operators. **Users** represent the people who operate the bank through the Admin Panel. Each user has a unique identifier that links with the external authentication system.

**Roles** group permission sets and are assigned to users. A user can have multiple roles, and their effective permissions are the union of permissions from all their roles. **PermissionSets** are named collections of specific permissions that facilitate configuration of common roles.

The permission system is granular: each operation in each module has an associated permission. For example, there are separate permissions for reading customers, creating customers, approving KYC, viewing credit facilities, initiating disbursements, etc. Before executing any operation, the system verifies that the user has the corresponding permission and records the action in the audit log.

The authorization system uses **Casbin**, a flexible access control engine, with policies stored in PostgreSQL for persistence and synchronization between instances. The RBAC model follows a three-level structure: User → Role → PermissionSet → Permissions (Object + Action).

Each module defines its own permission sets that group related actions. Typical permission sets follow a viewer/writer pattern. The system includes predefined roles like Admin (full access), Bank Manager (similar to admin but without access to access management or custody), and Accountant (focused on accounting and viewing functions).

Permissions are managed dynamically through the API and changes persist immediately to the database, reloading on each permission verification, ensuring updates are effective without restarting the system.

#### 2.1.8 Price

This module obtains and manages Bitcoin prices, a critical function for a bank offering BTC-collateralized loans. The system integrates with Bitfinex to obtain real-time prices through its API.

When a new price is obtained, the module publishes a **CorePriceEvent** that other modules consume. The credit module is the main consumer: it uses the price to calculate the CVL of all active facilities and determine if any have fallen below margin call or liquidation thresholds. Price changes can trigger state updates in facilities and potentially initiate liquidation processes.

#### 2.1.9 Report

The reports module coordinates the generation of regulatory and operational reports. It defines **Report** types that specify what data to include and in what format. Each report execution is registered as a **ReportRun** with its state (pending, executing, completed, failed) and generated files.

Report generation integrates with the data pipeline: transformed data in BigQuery feeds the final reports. The system can integrate with external reporting systems according to the regulatory needs of each jurisdiction where the bank operates.

#### 2.1.10 Support Modules

In addition to the main modules, there are support modules: **document-storage** for cloud document storage, **public-id** for generating readable public identifiers for entities, and **core-money** which defines monetary primitives (UsdCents, Satoshis) used throughout the system.

### 2.2 Application Layer

The `lana/` directory contains the application layer that orchestrates core modules and exposes functionality externally.

#### 2.2.1 GraphQL Servers

The system exposes two independent GraphQL servers. The **admin-server** serves the administration panel used by bank operators, while the **customer-server** serves the customer portal. Both servers include integrated playground for development and receive webhooks from external services.

#### 2.2.2 Application Services

The main service **lana-app** orchestrates the initialization of all modules and provides the unified entry point. **lana-cli** offers a command-line interface for administrative operations.

Specialized services exist for different functions: **notification** handles email sending, **pdf-generation** generates PDF contracts, **customer-sync** and **deposit-sync** synchronize data with external systems, **user-onboarding** manages operator registration, and **dashboard** calculates aggregated metrics. For development and testing, **sim-bootstrap** allows initializing simulation data.

#### 2.2.3 Event System

The **lana-events** module defines the unified enum **LanaEvent** that groups all system domain events, allowing the outbox system and jobs to process events from any module uniformly.

### 2.3 Web Frontends

#### 2.3.1 Admin Panel

The Admin Panel is the main interface for bank operators and staff. It allows managing customers and their KYC processes, administering credit facilities at all stages, approving disbursements and withdrawals, and managing deposit accounts. It also provides access to complete accounting visualization (balance, income statement, trial balance), configuration of committees and approval policies, user and role management, and generation of regulatory reports.

#### 2.3.2 Customer Portal

The Customer Portal is oriented toward bank customers. Currently it offers read-only functionality, allowing visualization of credit facilities, disbursement status, and transaction history. The architecture allows extending it in the future to support customer-side operations.

#### 2.3.3 Shared Web

The **shared-web** module contains UI components shared between both portals, ensuring visual consistency and reducing code duplication.

---

## 3. Communication Flows

### 3.1 Event Sourcing and Domain Events

The system uses **Event Sourcing** as a central architectural pattern. Each entity receives commands that generate events, these events are persisted in the database as the single source of truth, and the entity's current state is reconstructed by applying the sequence of events.

This design provides complete auditability (each change is recorded), the ability to reconstruct state at any point in time, and the possibility of adding new projections over historical data.

Communication between modules occurs through public events. Each module defines its own events in a specific enum (for example, **CoreCreditEvent** for the credit module). A **Publisher** associated with each module transforms internal entity events into public events that other modules can consume.

Typical public events include: creation and approval of credit proposals, activation and completion of facilities, collateralization changes, settled disbursements, interest accrual, creation and transition of obligations between states (due, overdue, defaulted), registered payments, and liquidation processes. Each event includes timestamps of when it was registered and when it was effective, allowing precise state reconstructions at any moment.

### 3.2 Outbox Pattern

For integrations with external systems requiring delivery guarantees, the system implements the **Outbox Pattern**. When a module needs to publish an event, it persists it in an outbox table within the same database transaction as the business operation. This guarantees atomicity: either both (the operation and the event) persist, or neither.

PostgreSQL NOTIFY immediately informs listeners when there are new events, avoiding the need for polling.

The system supports two types of events in the outbox. **Persistent events** have a unique identifier, a monotonically increasing global sequence number, the payload serialized as JSON, tracing context for distributed correlation, and timestamp of when it was registered. **Ephemeral events** have no sequence and are used for real-time notifications that don't need durability.

This design guarantees **at-least-once delivery**: an external system can consume events with certainty that it won't lose any, although it might receive duplicates that it must handle idempotently.

### 3.3 Asynchronous Jobs System

Operations that should not block the main flow are executed through an asynchronous jobs system. Workers run as separate processes from the main server, allowing scaling of processing independently from API servers.

Jobs can be scheduled in various ways: execute immediately, schedule for a specific future date/time, or reschedule upon completion to execute again. This flexibility is essential for the banking system's temporal flows. For example, when an obligation is created, a job is scheduled for the due date. When that job executes, if the obligation is not paid, it marks it as "due" and schedules the next job for the delinquency date. The chain continues: due → overdue → liquidation → defaulted, each transition precisely scheduled according to the facility's terms.

For interest accrual, a job processes each daily accrual and automatically reschedules for the next day. When an accrual period ends (typically at month-end), it schedules an accrual cycle job that consolidates interest and creates the corresponding obligation.

Other jobs process outbox event streams continuously, maintaining their execution state (the last processed event) and rescheduling immediately when there are no new events to continue listening.

### 3.4 Incoming Webhooks

External services notify the system through webhooks. **SumSub** sends notifications about the KYC verification lifecycle to `/webhook/sumsub`. When a customer completes their verification, SumSub notifies the result (approved or rejected). The system processes this notification and updates the customer's KYC state, which can unlock the creation of deposit accounts or credit facilities according to configuration.

**Bitcoin custodians** (BitGo, Komainu) notify wallet events to `/webhook/custodian/[provider]`. Each provider has its own webhook format that the system normalizes. Typical events include Bitcoin deposits to collateral wallets. When a notification arrives, the system verifies its authenticity (typically via HMAC), identifies the affected wallet, updates the corresponding collateral balance, and recalculates the CVL of the associated credit facility. If the new CVL crosses any configured threshold, the collateralization state is updated and corresponding events are published.

This webhook flow is critical for real-time risk management. Without it, the system would depend on periodic polling and could have delayed visibility of collateral changes, increasing risk during Bitcoin price drops.

### 3.5 GraphQL API Flow

Web client requests follow this flow: the client sends a GraphQL request with a JWT token. Middleware extracts the subject from the token and injects it into the context. The resolver invokes the corresponding use case in lana-app, which first verifies RBAC permissions and then executes the operation in the appropriate core module. Generated events are published, and the response returns to the client.

---

## 4. Integrations with External Systems

The application is designed to integrate with various external services that provide specialized functionalities. These services are not part of the deployed infrastructure but are critical components of the operational ecosystem.

**It is important to emphasize that these services must be configured externally** by the client or operations team. The application simply expects to receive the credentials, tokens, endpoints, and other configuration information necessary to integrate with these services. The application does not manage the creation, configuration, or administration of accounts in these external services; it only consumes their APIs and services once they are configured and available.

### 4.1 KYC/KYB and AML (Know Your Customer / Know Your Business / Anti-Money Laundering)

#### 4.1.1 Sumsub

Sumsub is used for managing KYC (Know Your Customer) and KYB (Know Your Business) processes and data. This external service handles identity verification of customers and companies, including:

- Identity document validation
- Biometric verification
- Corporate document verification
- Regulatory compliance
- Customer and company onboarding
- Continuous verification

Sumsub also satisfies AML (Anti-Money Laundering) needs in addition to providing KYC/KYB capabilities. Sumsub includes money laundering detection and prevention functionalities, such as:

- Sanctions list verification (OFAC, UN, etc.)
- Suspicious transaction analysis
- Behavioral pattern monitoring
- Automatic regulatory reports
- Compliance systems integration

The application integrates with Sumsub through its REST API. To configure the integration, it is necessary to configure an account in the Sumsub service, obtain API credentials (API key, API secret), configure the corresponding endpoints (may vary by region), and provide these credentials and endpoints as part of the environment configuration.

The integration flow works as follows: the application sends verification requests to Sumsub through its API. Sumsub processes the requests and performs the necessary verifications. Results from onboarding and continuous verification processes are received via webhooks at the `/webhook/sumsub` endpoint. When a customer completes their verification, SumSub notifies the result (approved or rejected), and the system processes this notification updating the customer's KYC state, which can unlock the creation of deposit accounts or credit facilities according to configuration.

The architecture is also prepared to integrate additional AML systems if necessary. AML integrations typically include the functionalities mentioned above. The application can integrate with AML service providers through REST APIs or through integration with third-party systems. Configuration would follow the same pattern as other external integrations: credentials and endpoints are provided as part of the environment configuration.

### 4.2 Payment Gateways

**Important note:** Payment gateway integrations are not implemented in the current version of Lana. However, because Lana is modular in design, the architecture anticipates that these elements will eventually be added according to business needs.

The application is designed to integrate with external payment gateways to process financial transactions. Although specific gateways may vary by client and region, the architecture supports integration with multiple providers.

The application is designed to support various types of integration:

- Card payment processing (debit/credit)
- Bank transfers (ACH, wire transfers, etc.)
- Mobile payment processing
- Integration with clearing and settlement systems

Payment gateways would integrate via REST or SOAP APIs. API credentials, endpoints, and specific configurations would be provided as part of the environment configuration. The application is designed to support multiple gateways simultaneously, allowing transaction routing according to business rules.

All communications with payment gateways would use TLS/SSL for encryption in transit. Sensitive credentials would be stored as secrets in Kubernetes and injected into application containers via environment variables or mounted volumes.

### 4.3 BCR (Central Reserve Bank)

**Important note:** Integration with the Central Reserve Bank (BCR) is not implemented in the current version of Lana. However, because Lana is modular in design, the architecture anticipates that this integration will eventually be added according to business needs.

The application is designed to include support for operations with the Central Reserve Bank (BCR), which is El Salvador's central bank. This integration would be critical for regulatory banking operations.

The system is designed to support various types of operations with the BCR:

- Deposits at the BCR (local and foreign currency)
- Repo operations with the BCR
- Financing operations with the BCR
- Regulatory reports and compliance
- Liquidity operations

Integration with the BCR would be done through standard banking communication systems (typically SWIFT, financial messaging systems, or BCR-specific APIs). Configuration would include BCR systems access credentials, communication endpoints, digital certificates for authentication, and message format configuration (ISO 20022, proprietary formats, etc.).

BCR operations would be processed through dedicated workers that would handle asynchronous communication and response processing. Operations data would be recorded in the main database and integrated with the accounting system.

### 4.4 Regulatory Data Sources

**Important note:** Integrations with regulatory data sources are not implemented in the current version of Lana. However, because Lana is modular in design, the architecture anticipates that these elements will eventually be added according to business needs.

The application is designed to integrate with multiple regulatory data sources for compliance and reporting. These would include:

- Central bank reporting systems
- Credit information systems
- Public registries (company registry, property registry, etc.)
- Government identity verification systems
- Financial information exchange systems

Integrations with regulatory data sources would be done through:

- REST or SOAP APIs provided by regulatory bodies
- Financial messaging systems (SWIFT, proprietary systems)
- Batch files for data exchange
- Web portals with authentication and automated scraping (when necessary)

Application workers would process integrations with regulatory systems asynchronously. Received data would be validated, transformed, and stored in the database. Regulatory reports would be generated automatically according to requirements and sent through appropriate channels.

### 4.5 Observability

#### 4.5.1 Honeycomb

Honeycomb is used for aggregation and exploitation of OpenTelemetry data, as well as for generating alerts that integrate with pager/on-call management software. The system uses the OpenTelemetry (OTEL) protocol to send metrics, logs, and traces from the OpenTelemetry Collector to Honeycomb.

The OpenTelemetry Collector is configured with Honeycomb's API key and dataset. Data is sent automatically via the OTEL protocol. Although Honeycomb is currently used, the application uses the standard OTEL protocol, which allows migrating to other compatible providers (Datadog, New Relic, Grafana Cloud, etc.) without significant modifications.

The system is instrumented to provide complete visibility of its behavior in production. OpenTelemetry captures traces of all operations, from receiving an HTTP request to the final response. Each significant operation creates a span with relevant attributes. Spans propagate through asynchronous calls and between services, allowing reconstruction of the complete flow of an operation.

Traces are exported to Honeycomb, where they can be analyzed to identify bottlenecks, errors, and usage patterns. Propagation of tracing context through the outbox allows correlating the original operation with its subsequent asynchronous processing.

Logging uses Rust's **tracing** crate, which provides structured logs with levels (error, warn, info, debug, trace) and typed fields. Logs are emitted in JSON format in production, facilitating their indexing and search. Each log entry automatically includes the context of the current span, connecting it with the distributed trace.

### 4.6 Reporting Data Storage

#### 4.6.1 BigQuery

BigQuery is used as analytical and reporting data storage. The system uses BigQuery to store transformed data from PostgreSQL operational databases, allowing analysis and reporting without impacting transactional database performance.

The application uses BigQuery in conjunction with ETL tools (Meltano) and data transformation (dbt) to load and transform data from PostgreSQL to BigQuery. Meltano extracts data from multiple sources: the main extractor **tap-postgres** obtains events and entities from the banking core, and additional extractors obtain historical prices from Bitfinex and KYC verification data from SumSub.

Data is loaded into BigQuery, where dbt transforms it through layers: staging (raw data cleaning), intermediate (business logic), and outputs (final reports). The system generates regulatory reports that can integrate with external systems according to each jurisdiction's needs.

Configuration includes service account JSON, project ID, and dataset names. **It is important to note that, although BigQuery is currently used, the application can be refactored to perform the same work in other analytical databases.** ETL and transformation code can be adapted to work with alternatives such as Amazon Redshift, Snowflake, Azure Synapse Analytics, or even on-premise analytical databases.

---

## 5. Authentication and Security Flows

### 5.1 IAM (Identity and Access Management)

#### 5.1.1 Keycloak

Keycloak acts as the central identity and access (IAM) server integrated with the application. It provides:

- User and role management
- Authentication through multiple methods (username/password, OAuth2, OIDC)
- Role-based authorization (RBAC)
- Single Sign-On (SSO)
- Session management
- Integration with external identity providers (Google, etc.)

**Federated Nature and External Employee Authentication:**

Due to its federated nature, Keycloak is designed to delegate authentication of internal users (employees) to external identity systems. **The employee authentication backend is expected to come externally.** For example, if the institution uses Azure Active Directory (Azure AD), Keycloak should be integrated with Azure AD so that Keycloak delegates authentication to Azure AD. This is a deployment detail that must be addressed in each case according to the institution's needs and existing identity systems.

**Configurability:**

Keycloak is highly configurable and the configuration described below is a suggestion that can be adapted to each deployment's needs. Realms, clients, authentication flows, and identity providers can be configured according to each client's specific requirements.

As a suggestion, three realms are configured:

- **Internal Realm:** For internal users and application services
- **Customer Realm:** For application customers
- **Data-Dagster Realm:** For access to data tools (Dagster)

Similarly, three application clients are suggested:

- **internal-service-account:** For internal application services
- **customer-service-account:** For the customer portal
- **oauth2-proxy:** For OAuth2 Proxy authentication

The authentication flow for internal users works as follows: when a user accesses the Admin Panel (`admin.{domain}`), the application redirects to Keycloak for authentication. Keycloak can delegate authentication to an external identity provider (e.g., Azure AD, LDAP, etc.) or validate credentials directly. After successful authentication, Keycloak generates JWT tokens used to authenticate requests to the GraphQL API. Finally, Oathkeeper validates JWT tokens before allowing access to resources.

For customers, the authentication flow is similar: when a customer accesses the Customer Portal (`app.{domain}`), the application redirects to Keycloak (Customer Realm) for authentication. Keycloak validates credentials and generates JWT tokens, which are used to authenticate requests to the public API. Oathkeeper validates JWT tokens before allowing access to resources.

The described authentication flows are examples and may vary according to each deployment's specific configuration, especially regarding integration with external identity providers for internal users.

#### 5.1.2 Oathkeeper

Oathkeeper acts as an authentication and authorization proxy, providing:

- JWT token validation
- Routing of authenticated requests
- Token mutation (claims transformation)
- Access rules based on URL and HTTP method
- High availability (2 replicas by default)

Several access rules are configured:

- **admin-api:** Protects the Admin Panel GraphQL endpoint, requires JWT authentication
- **admin-ui:** Protects the Admin Panel interface, allows access without authentication (authentication handled by the application)
- **customer-ui:** Protects the Customer Portal, allows access without authentication (authentication handled by the application)
- **customer-api:** Protects the Customer Portal public API, requires JWT authentication

The validation flow works as follows: when a client sends a request with a JWT token in the Authorization header, Oathkeeper extracts and validates the JWT token against Keycloak's JWKS. Oathkeeper verifies that the token has not expired and that the issuer is valid, then applies authorization rules according to URL and method. If authorization is successful, Oathkeeper mutates the token (optional) and forwards the request to the upstream service.

#### 5.1.3 OAuth2 Proxy

OAuth2 Proxy provides OAuth2/OIDC authentication for applications that don't support native authentication. It is primarily used to protect access to Dagster.

The authentication flow with OAuth2 Proxy works like this: when a user accesses Dagster (`dagster.{domain}`), OAuth2 Proxy intercepts the request and checks if there is a valid session. If there is no session, OAuth2 Proxy redirects to Keycloak for authentication. The user authenticates in Keycloak (can use Google as identity provider), and Keycloak redirects back to OAuth2 Proxy with an authorization code. OAuth2 Proxy exchanges the code for tokens and creates a session, finally allowing access to Dagster with authentication headers.

### 5.2 WAF (Web Application Firewall)

The system uses NGINX Ingress Controller as the entry point, which provides WAF capabilities through various functionalities.

**Geo-blocking:** Allows blocking unsupported countries configured via GeoIP2. The GeoIP2 database is automatically updated from a GCS bucket, and blocking rules are configured via NGINX maps.

**Rate Limiting:** Includes request limitation per minute per host, simultaneous connection limitation, and configuration per host (customer portal, admin panel, dagster).

**Additional Protection:** There is the possibility of configuring WAF via NGINX annotations, integration with external WAF services (Cloudflare, AWS WAF, Azure WAF, etc.), and protection against common attacks (DDoS, SQL injection, XSS, etc.).

### 5.3 Firewalls

#### 5.3.1 Firewall Rules in GCP

Firewall rules in GCP include:

- **Intra-cluster Egress:** Allows communication between pods and with the master (TCP, UDP, ICMP, SCTP, ESP, AH protocols) to Master CIDR, Cluster subnet, Pods range, and Services range
- **Webhook Ingress:** Allows the master to call webhooks in pods (ports 8443, 443) from Master CIDR
- **DMZ to Nodes:** Allows access from bastion to cluster nodes (all protocols) from DMZ subnet

#### 5.3.2 Network Security Groups in Azure

Network Security Groups (NSG) in Azure provide firewall rules per subnet:

- **PostgreSQL NSG:** Allows only traffic from VirtualNetwork to port 5432
- **Cluster NSG:** Controls traffic to and from Kubernetes nodes
- **DMZ NSG:** Controls access to bastion hosts

### 5.4 Encryption in Transit

For external communications, all publicly exposed services use HTTPS/TLS. SSL/TLS certificates are automatically managed by cert-manager, which can use Let's Encrypt (for public certificates) or an internal CA (for private certificates). Certificates are automatically renewed before expiration.

For internal communications, PostgreSQL databases require SSL/TLS for all connections (sslmode = "require" in Azure). Communications between services within the cluster can use mTLS (mutual TLS) via service mesh (optional). Communications with external services (Sumsub, payment gateways, etc.) use HTTPS/TLS.

Secure protocols and versions are used: TLS 1.2 or higher for all connections, secure cipher suites configured in NGINX Ingress, and Perfect Forward Secrecy (PFS) enabled.

### 5.5 Encryption at Rest

Managed databases (Cloud SQL, Azure PostgreSQL) use encryption at rest provided by the cloud provider. In GCP, Cloud SQL uses automatic encryption of data at rest. In Azure, Azure PostgreSQL Flexible Server uses automatic encryption with Microsoft-managed keys or customer-managed keys (CMK). Backups are also encrypted.

Objects stored in GCS buckets (documents, reports, etc.) use encryption at rest. Encryption can be managed by Google or via customer-managed keys (CMEK).

Kubernetes secrets are stored encrypted in etcd. In GCP, etcd is encrypted via Google-managed keys. In Azure, etcd is encrypted via Microsoft-managed keys. Sensitive secrets (passwords, API keys, etc.) are stored as Kubernetes Secrets and injected into containers.

Persistent volumes use encryption provided by the cloud provider. In GCP, Persistent Volumes use automatic encryption. In Azure, Managed Disks use automatic encryption.

### 5.6 VPN (Virtual Private Network)

**Important note:** VPN configuration is a deployment detail, not part of the Lana application. What is presented below are suggestions and architectural options that may be useful for different scenarios. It is the operator's responsibility to make final decisions and designs that fit their deployment's specific needs, including security, compliance, and organizational requirements.

The system can support multiple VPN options for administrative and employee access, depending on the configuration chosen by the operator.

#### 5.6.1 VPN Site-to-Site

One option is to configure VPN between the office/corporate network and the VPC/VNet via Cloud VPN or Partner VPN in GCP, or VPN Gateway (Site-to-Site) in Azure. Advantages include direct access to internal resources without exposing services to the Internet, no public IPs required for internal services, and centralized access control. Employees connected to the corporate network would access automatically.

#### 5.6.2 VPN Client (Point-to-Site)

Another option is client VPN configuration for remote access, which presents differences depending on the cloud provider: Cloud VPN doesn't support P2S natively in GCP, requiring third-party solution, while in Azure you can use VPN Gateway (Point-to-Site) with OpenVPN or IKEv2. Advantages include access from any location, certificate or username/password authentication, and no corporate network required. Remote employees would connect via VPN client.

#### 5.6.3 Bastion Host with VPN

An alternative is VPN configuration to the bastion host with port forwarding, which would work as follows: employee connects to VPN, VPN terminates at bastion host, and employee accesses internal services through the bastion. Advantages include granular access control, centralized auditing, and no changes to main infrastructure required.

#### 5.6.4 Access via Bastion (SSH Tunneling)

Another option is SSH tunnel configuration through the bastion host for administrative access and debugging. For example, tunneling to PostgreSQL database via `ssh -L localhost:5432:db-internal-ip:5432 bastion-host`. Advantages include secure access to internal resources without exposing them to the Internet.

The operator must evaluate these options and select or design the remote access solution that best fits their specific security, compliance, and operational requirements.

### 5.7 Certificates

cert-manager automatically manages SSL/TLS certificates. Certificates are created as Kubernetes resources (Certificates), cert-manager requests certificates from Let's Encrypt or internal CA according to configuration, certificates are automatically renewed before expiration, and are stored as Kubernetes Secrets.

Certificates for communication with databases and internal services can be managed by cert-manager or provided manually. Certificates for authentication with external services (BCR, regulatory systems) are provided as part of the environment configuration.

---

## 6. Network Segmentation by Environment

The architecture implements complete isolation between different environments (DEV, QA, UAT, PROD). **Environments do not share any infrastructure resources.**

Each environment has:

- Its own completely isolated VPC/VNet
- Its own Kubernetes cluster
- Its own database instances
- Its own load balancers and public IPs
- Its own credentials and secrets
- Its own domains and SSL/TLS certificates

There is no direct network connectivity between environments. There is no VPC/VNet peering between environments. There are no network routes that allow communication between environments. Each environment is completely independent and isolated from the others.

---

## 7. Security Zones

The architecture implements a security zones model that segments infrastructure according to exposure level and security requirements.

### 7.1 Public Zone

The public zone contains services that are exposed to the Internet and publicly accessible.

**Components:**
- **Load Balancer:** Public IP provided by the cloud provider
- **NGINX Ingress Controller:** Entry point for all HTTP/HTTPS traffic
- **Customer Portal:** Publicly accessible frontend (`app.{domain}`)
- **SSL/TLS Certificates:** Managed by cert-manager (Let's Encrypt or internal CA)

**Security Characteristics:**
- TLS/SSL required for all connections (HTTPS)
- Geo-blocking configured to block unsupported countries
- Rate limiting configured per host
- WAF capabilities via NGINX or external services
- Anomalous traffic monitoring and alerts
- Authentication required for access to sensitive functionalities

Traffic flow follows this route:
```
Internet Client → Load Balancer (Public IP) → NGINX Ingress Controller → Application Services
```

### 7.2 Private Zone

The private zone contains services that are not exposed to the Internet and are only accessible from within the VPC/VNet.

**Components:**
- **Kubernetes Cluster:** Application nodes and pods
- **Backend Services:** Internal APIs, workers, processing services
- **Admin Panel:** Administrative panel (accessible only via VPN or private network)
- **PostgreSQL Databases:** Database instances with private access only

**Security Characteristics:**
- No public IPs (nodes without public IPs, enable_private_nodes = true in GCP)
- Access only from within the VPC/VNet
- Network Policies enabled (Calico in GCP, Azure Network Policy in Azure)
- Firewall rules restricting communication between components
- TLS/SSL for internal communications
- Authentication and authorization via Keycloak and Oathkeeper

Traffic flow follows this route:
```
Internal Services → Network Policies → Application Services → Databases (PostgreSQL)
```

### 7.3 Administration Zone

The administration zone contains resources for administrative access and infrastructure management.

**Components:**
- **Bastion Hosts:** Hosts in DMZ subnet for administrative access
- **Kubernetes API:** Private cluster endpoint (not accessible from Internet)
- **Management Tools:** Helm, kubectl, CI/CD tools

**Security Characteristics:**
- Bastion hosts in isolated DMZ subnet
- Kubernetes API access restricted to bastion hosts and authorized networks
- Strong authentication required (SSH keys, certificates)
- Administrative access auditing
- Access via VPN or SSH tunneling
- Regular rotation of credentials and keys

Access flow follows this route:
```
Administrator → VPN/SSH → Bastion Host → Kubernetes API / Internal Services
```

### 7.4 Backups Zone

The backups zone contains systems and storage for data backups.

**Components:**
- **Database Backups:** Automatic backups managed by the cloud provider
- **Backup Storage:** GCS buckets or Azure Blob Storage for backups
- **Point-in-Time Recovery:** Enabled for critical databases

**Security Characteristics:**
- Backups encrypted at rest
- Geo-redundant backups (multiregion) for critical redundancy
- Configurable retention (7-35 days depending on environment)
- Restricted access to backups (only authorized services)
- Automatic rotation of old backups
- Periodic restoration tests

In GCP Cloud SQL, automatic backups are enabled, point-in-time recovery is enabled, and backups are multiregion. In Azure PostgreSQL, automatic backups have configurable retention and geo-redundant backups are optional.

### 7.5 Monitoring Zone

The monitoring zone contains observability, logging, and alerting systems.

**Components:**
- **OpenTelemetry Collector:** Collects metrics, logs, and traces
- **Honeycomb:** Telemetry data aggregation and analysis
- **Alert Systems:** Integration with pager/on-call systems (Zenduty, PagerDuty, etc.)
- **Application Logs:** Kubernetes pod and service logs

**Security Characteristics:**
- Encrypted communication with external monitoring services (TLS/SSL)
- API keys stored as Kubernetes Secrets
- Restricted access to dashboards and monitoring data
- Configurable log retention
- Sensitive data anonymization in logs

Data flow follows this route:
```
Applications → OpenTelemetry Collector → Honeycomb → Alerts → Pager/OnCall Systems
```

### 7.6 Communication Between Zones

**Important note:** Much of the details about communication between zones, administrative access, backup storage, and monitoring are deployment details, not part of the Lana application. What is presented below are suggestions and architectural considerations. The final design, implementation, and operation of these aspects is the responsibility of the deployment operator, who must adapt them to their specific security, compliance, and operational requirements.

#### 7.6.1 Zone Definitions

**Public Zone:** Contains web applications and integrations/APIs accessible from WAN (Wide Area Network). Access to these applications is controlled from INGRESS, which acts as the entry point and applies authentication, authorization, and security rules.

**Private Zone:** All application services are in the private network. These services are not directly exposed to the Internet and are only accessible from within the VPC/VNet or through controlled access mechanisms.

#### 7.6.2 Communication from Public Zone to Private Zone

Communication from the Public Zone to the Private Zone is done via HTTP/HTTPS traffic from the Internet passing through the Load Balancer, then the NGINX Ingress Controller, and finally the Application Services. Authentication and authorization is performed via Oathkeeper before accessing private services.

#### 7.6.3 Access to Private Zone (Deployment Detail)

How to organize administrative and employee access to the Private Zone (via Bastion hosts, VPN, SSH tunneling, etc.) is a deployment detail that must be designed and implemented by the operator according to their specific needs. The operator must consider factors such as security requirements, organizational policies, compliance, and remote access preferences.

#### 7.6.4 Backup Storage (Deployment Detail)

How to store backups securely and privately is a deployment detail. The operator must design and implement the backup strategy that best fits their requirements, including considerations about encryption, geographic redundancy, retention, and restricted access.

#### 7.6.5 Monitoring (Deployment Detail)

The configuration and operation of monitoring, observability, and alerting systems is a deployment detail. The operator must select and implement the monitoring tools and services that best fit their needs, including considerations about where to store metrics and logs, how to configure alerts, and what level of observability is required.

#### 7.6.6 Communication Restrictions

Important restrictions exist in zone design: typically there is no direct communication from the public zone to the administration zone, no direct communication from the public zone to the backups zone, and communication between zones is controlled by firewall rules and network policies. However, the specific design of these restrictions and controls is the responsibility of the deployment operator.

---

## 8. Audit

The audit system is a cross-cutting component that records all actions performed in the system, providing complete traceability for regulatory compliance and incident investigation.

### 8.1 Audit Entry Structure

Each audit entry captures four fundamental dimensions:

- **Subject:** Who performed the action. Can be a user identified by their UUID (when operating through the Admin Panel) or the system itself (for automatic operations like interest accrual jobs or webhook processing).

- **Object:** What the action was performed on. Objects are typed and can refer to specific entities (a particular customer, a specific credit facility) or complete categories (all customers, all facilities). The format includes the entity type and its identifier, for example `customer/550e8400-e29b-41d4-a716-446655440000` or `credit-facility/all`.

- **Action:** What type of operation was attempted. Actions are categorized by module and entity, following a format like `customer:read`, `credit-facility:create`, `withdrawal:approve`. Each module defines its own possible actions.

- **Authorized:** Whether the operation was permitted or denied. The system records even failed access attempts, which allows detecting patterns of unauthorized attempts.

Additionally, each entry has a timestamp of when it was registered and a unique sequential identifier.

### 8.2 Integration with Operations Flow

The audit system is directly integrated into the authorization flow. When a user attempts to perform an operation, the permission system (RBAC) verifies if they have the necessary permissions and simultaneously records the audit entry. This integration guarantees that no operation, successful or failed, escapes the record.

For operations occurring within database transactions, the system supports transactional audit recording: the audit entry is inserted in the same transaction as the business operation, guaranteeing consistency. If the transaction fails, the audit entry also rolls back.

System operations (not initiated by users) are recorded with a special "system" subject, allowing distinction between human and automated actions. This is important for operations like automatic obligation transition to "due" or "overdue" state, collateral synchronization from custodian webhooks, or interest accrual.

### 8.3 Correlation with Tracing

The audit system integrates with the distributed tracing context. When an audit entry is recorded, it is associated with the current OpenTelemetry span. This allows correlating a specific audit entry with the complete trace of the operation, including all internal calls, database queries, and communications with external services that occurred as part of that operation.

### 8.4 Audit Log Query

The audit log is queryable through the Admin Panel GraphQL API, allowing authorized operators (with the `audit:list` permission) to search and filter entries. Pagination is cursor-based to efficiently handle large data volumes. Entries are ordered by ID descending, showing most recent first.

---

## 9. Bitcoin-Backed Loan Flow

To illustrate how modules interact, this is the typical flow of a loan:

### 9.1 1. Proposal and Approval

1. **PROPOSAL:** Customer requests a credit proposal, which enters an approval process managed by the governance module.

2. **APPROVAL:** The assigned committee votes to approve the proposal. When the approval threshold is reached, a **PendingCreditFacility** is created.

### 9.2 2. Collateralization and Activation

3. **COLLATERALIZATION:** The customer deposits Bitcoin as collateral through the configured custodian. Custodian webhooks automatically keep the collateral balance synchronized, and the system recalculates the CVL.

4. **ACTIVATION:** When the CVL reaches the initial threshold configured in the terms, the facility is automatically activated.

### 9.3 3. Disbursements and Loan Life

5. **DISBURSEMENT:** The customer can request disbursements, each going through its own approval process. When executed, funds are credited to the customer's deposit account and an **Obligation** is created representing the debt.

6. **LOAN LIFE:** Periodic jobs calculate and record accrued interest according to configured intervals, generating new obligations for accrued interest.

7. **PAYMENTS:** When the customer makes a **Payment**, the system automatically allocates funds to pending obligations in priority order via **PaymentAllocation**, typically prioritizing the oldest obligations and interest over principal.

8. **CLOSURE:** When all obligations are settled, the facility can be closed and Bitcoin collateral is released to the customer.

### 9.4 4. CVL Monitoring and Risk Management

Throughout the entire lifecycle, the system continuously monitors the CVL. If it falls below the margin call threshold, the facility enters an alert state. If it falls below the liquidation threshold, a **LiquidationProcess** is initiated where the bank can execute the collateral to recover the debt.

### 9.5 Additional Notes

#### 9.5.1 Client Configuration

Different clients can have specific configurations of external integrations, security zones, and network segmentation according to their regulatory and business requirements.

#### 9.5.2 Updates and Maintenance

Updates to security components (Keycloak, Oathkeeper, cert-manager, etc.) are managed via Helm charts and applied in a controlled manner in each environment.

#### 9.5.3 Regulatory Compliance

The architecture is designed to comply with banking regulatory requirements, including:

- Data isolation by environment
- Data encryption in transit and at rest
- Access auditing and logging
- Backups and disaster recovery
- Integration with regulatory systems

---

## 10. Portability and Vendor Lock-in

### 10.1 Kubernetes Cluster Portability

The application is designed to deploy on a Kubernetes cluster **agnostic to any cloud provider**. There is no vendor lock-in from using proprietary services from any provider, as the architecture uses standard Kubernetes components and services that can be replaced by equivalent alternatives. Some services not managed via Kubernetes, like Postgres databases, can be deployed on generic hosts.

The application **can be deployed on-premise** without significant modifications. However, the physical infrastructure manager will need to address critical aspects that cloud providers manage automatically: implementing equivalent backup strategies, ensuring high availability through hardware and component redundancy, implementing data replication to geographically separated locations (offsite replication), and managing hardware maintenance, updates, and monitoring.

Main components are portable and can run in any Kubernetes-compatible environment:

- Kubernetes cluster (any distribution: GKE, AKS, EKS, Rancher, k3s, etc.)
- PostgreSQL (Cloud SQL, Azure PostgreSQL, or on-premise managed instances)
- Ingress Controller (NGINX Ingress)
- Helm Charts (Kubernetes standard)
- Containerized applications (Docker)

Provider-specific components can be replaced: Load Balancers with on-premise or alternative solutions, Persistent Volumes can use any Kubernetes-compatible storage class, and VPC/VNet can be replaced by physical networks or SDN (Software Defined Networking).

---

## 11. Servers / Instances

### 11.1 Instance Types

**Currently**, infrastructure is deployed on two cloud providers: **Google Cloud Platform (GCP)** and **Microsoft Azure**. For these two providers we can offer very specific advice and detailed configurations, as they are the environments in which we have direct operational experience. However, the architecture is portable and the application can be adjusted for other providers (like AWS, Oracle Cloud, etc.) or on-premise systems.

#### 11.1.1 Application Instances (Kubernetes Nodes)

In GCP, Kubernetes nodes use the **n2-standard-4** machine type by default, providing 4 vCPUs, 16 GB of RAM, and 100 GB of disk (pd-standard). The cluster is configured with autoscaling allowing between 1 and 3 nodes (configurable per environment). Nodes use the COS_CONTAINERD image (Container-Optimized OS with containerd) and are automatically distributed across multiple zones within the region for redundancy.

In Azure, nodes use **Standard_DS2_v2** by default (2 vCPUs, 7 GB RAM, premium SSD disk) or **Standard_B1s** for development/staging environments (1 vCPU, 1 GB RAM). Autoscaling also allows between 1 and 3 nodes according to configuration.

#### 11.1.2 Database Instances

Databases use a single instance per database architecture, designed for vertical scaling (increasing CPU, RAM, and storage) instead of horizontal scaling. It is recommended to activate cloud provider autoscaling options to expand storage proportionally to database growth.

In GCP, instances use Cloud SQL for PostgreSQL (Enterprise Edition) with a default tier of **db-custom-1-3840** (1 vCPU, 3.75 GB RAM). Storage starts at 100 GB and must be expanded according to usage. High availability is configurable via `highly_available = true/false`, allowing ZONAL mode (no redundancy) or REGIONAL (with redundancy between zones).

In Azure, instances use Azure Database for PostgreSQL Flexible Server with default SKU **GP_Standard_D2s_v3** (2 vCPUs, 8 GB RAM). Storage also starts at 100 GB and is expandable. High availability is configured via `geo_redundant_backup_enabled` and instances are located in Zone 1 by default (configurable).

#### 11.1.3 Bastion Instances

Bastion hosts provide secure administrative access to infrastructure. In GCP they use **e2-small** machine type (2 shared vCPUs, 2 GB RAM) with Ubuntu 22.04 LTS. In Azure they use **Standard_DS1_v2** (1 vCPU, 3.5 GB RAM, 7 GB SSD) also with Ubuntu 22.04 LTS.

#### 11.1.4 Application Instances (Stateless)

Stateless elements (backend server, auth server, application front servers, workers, etc.) run as pods in Kubernetes and can scale horizontally. It is recommended to start with 1 replica per service and increase the number of replicas according to needed load.

Workers have resources configured with requests of 1000m CPU (1 core) and 1000Mi-1500Mi memory, with limits of 2000m-3000m CPU and 3000Mi-4000Mi memory, depending on the environment.

### 11.2 Estimated Storage

Kubernetes nodes use 100 GB of system disk per node (pd-standard in GCP), resulting in total estimated storage of 100-300 GB depending on the number of nodes.

For databases, it is recommended to start with 100 GB per instance. Storage is proportional to database growth, and **it is recommended to activate cloud provider autoscaling options** to expand storage automatically with application usage. In GCP Cloud SQL there is no specific limit configured in code (depends on tier), while in Azure PostgreSQL it is configurable starting with 100 GB.

Persistent volumes are created as needed for specific applications (for example, Meltano). It is important to note that all persistence is managed with PostgreSQL; no other persistent storage systems like Redis or MongoDB are used.

### 11.3 Redundancy

Kubernetes clusters in GCP automatically distribute nodes across multiple zones within the region. In Azure, nodes are distributed in Availability Sets/Zones according to configuration. Both providers have auto-repair enabled, but auto-upgrade is disabled to allow controlled manual upgrades.

For databases, **it is critical to activate multiregion redundant backups** to avoid data loss that could be disastrous for bank operations. In GCP, this is achieved through high availability with `availability_type = "REGIONAL"` (when enabled), point-in-time recovery enabled, automatic backups enabled, and multiregion backup configuration for critical redundancy. In Azure, `geo_redundant_backup_enabled = true` must be enabled, with configurable backup retention between 7-35 days.

Application services run as Kubernetes deployments with multiple replicas when necessary. For example, Oathkeeper has 2 replicas by default for high availability.

---

## 12. Operating Systems

### 12.1 Compatible and Certified Versions

The system is designed to run in Linux environments. Everything is managed with Docker images that use Nix to create deterministic environments, ensuring reproducibility and consistency between different environments.

Kubernetes nodes in GCP use Container-Optimized OS (COS) with containerd, with specific version managed by GKE and compatible with Kubernetes 1.32.9-gke.1548000 (default version). In Azure, nodes use Ubuntu (version managed by AKS) compatible with Kubernetes 1.30.9 (default version).

Bastion hosts use **Ubuntu 22.04 LTS** (Jammy Jellyfish), certified and tested on both providers. In GCP the `ubuntu-2204-lts` image is used and in Azure `0001-com-ubuntu-server-jammy`.

CI/CD containers use Ubuntu as base (without specifying specific LTS version in Dockerfile) and also use Docker with Nix for deterministic environments.

---

## 13. Databases

### 13.1 Type and Version

The system exclusively uses **PostgreSQL** as the database management system. The recommended version varies by provider: in GCP **PostgreSQL 17** (POSTGRES_17) is used as the default version, although PostgreSQL 15 (POSTGRES_15) is used in Lana Bank staging. In Azure, the default version is **PostgreSQL 16** (16), although PostgreSQL 14 is also supported.

In GCP, Cloud SQL for PostgreSQL (Enterprise Edition) is used, while in Azure, Azure Database for PostgreSQL Flexible Server is used.

### 13.2 Security Parameters

All database instances are configured with **private access only**. Public IPv4 access is disabled (`ipv4_enabled = false`) and all instances are connected to private VPC/VNet. SSL/TLS is required for all connections (`sslmode = "require"` in Azure).

Administrator users are automatically generated with random 20-character passwords. Application users are created per database with specific permissions, and by default have no permissions to create databases (`user_can_create_db = false`).

Detailed logging is optional (`enable_detailed_logging`). When enabled, `log_statement = "all"` (logs all SQL statements) and `log_lock_waits = "on"` (logs lock waits) are configured. Standard logging is enabled by default.

### 13.3 Replication

Logical replication can be enabled via `replication = true`. In GCP it requires `cloudsql.logical_decoding = "on"` and `cloudsql.enable_pglogical = "on"`, while in Azure it requires `wal_level = "logical"`.

It is not strictly necessary to use read replicas, but they are advised in case data query needs appear from external applications, with the objective of not overloading database write instances. In GCP, read replicas are supported via `provision_read_replica = true`, which can be public or private (`public_read_replica`). In Azure there is no explicit read replica configuration in current code.

### 13.4 Backup

In GCP Cloud SQL, automatic backups are enabled by default (`enabled = true`), along with point-in-time recovery enabled (`point_in_time_recovery_enabled = true`). Retention is managed by GCP (typically 7 days for automatic backups) and frequency is daily.

In Azure PostgreSQL, automatic backups are enabled with configurable retention between 7-35 days via `backup_retention_days`. Geo-redundant backups are optional via `geo_redundant_backup_enabled`, and frequency is managed by Azure.

### 13.5 Databases per Application

All persistence is managed with PostgreSQL.

For Lana Bank, main databases include **lana-bank** (main application database), **meltano** (for ETL and data pipelines), **airflow** (for workflow orchestration), **dagster** (for data management), and **keycloak** (for authentication and authorization).

Each application can have multiple PostgreSQL instances: one instance for Lana Bank, one instance for Meltano (which can include multiple databases), and one instance for Keycloak.

---

## 14. Middleware / Integration

### 14.1 Kubernetes and Orchestration

The system uses Kubernetes for container orchestration. In GCP version 1.32.9-gke.1548000 (by default) is used and in Azure version 1.30.9 (by default). Network Policies are enabled (Calico in GCP, Azure Network Policy in Azure). In GCP, Binary Authorization and Shielded Nodes (Secure Boot and Integrity Monitoring) are also enabled. Helm version 3.x is installed on bastion hosts for chart management.

### 14.2 Ingress and Load Balancing

The system uses **NGINX Ingress Controller** (chart ingress-nginx version 4.14.0 from repository https://kubernetes.github.io/ingress-nginx) to provide ingress controller for Kubernetes. The ingress service is configured as LoadBalancer type, exposing a public IP that receives HTTP/HTTPS traffic from the Internet.

### 14.3 SSL/TLS Certificates

SSL/TLS certificates are automatically managed by **cert-manager** (chart version v1.19.1 from repository https://charts.jetstack.io), which can use Let's Encrypt or an internal CA according to configuration.

### 14.4 Authentication and Authorization

The system uses **Keycloak** (chart keycloakx version 7.1.1 from repository https://codecentric.github.io/helm-charts) as the identity and access (IAM) server, with a dedicated PostgreSQL database.

**Oathkeeper** (chart version 0.49.2 from repository https://k8s.ory.sh/helm/charts) acts as authentication and authorization proxy with 2 replicas by default for high availability.

**OAuth2 Proxy** (chart version 7.13.0 from repository https://oauth2-proxy.github.io/manifests) provides OAuth2 authentication proxy.

### 14.5 Observability and Monitoring

**OpenTelemetry Collector** (chart version 0.138.1 from repository https://open-telemetry.github.io/opentelemetry-helm-charts) collects metrics, logs, and traces, integrated with Honeycomb for data analysis.

### 14.6 Data Pipeline and ETL

**Dagster** (chart version 1.12.1 from repository https://dagster-io.github.io/helm) orchestrates data pipelines with a dedicated PostgreSQL database.

**Meltano** provides ETL and data management, also with a dedicated PostgreSQL database and Airflow integration for orchestration.

**Airflow** orchestrates workflows using PostgreSQL (shared with Meltano or dedicated according to configuration).

### 14.7 PostgreSQL (Helm Chart)

For in-cluster PostgreSQL (as opposed to managed Cloud SQL/Azure PostgreSQL instances), the Bitnami PostgreSQL chart version 16.4.13 (repository https://charts.bitnami.com/bitnami) with the image `bitnamilegacy/postgresql:14.5.0-debian-11-r35` is used.

### 14.8 Application Dependencies

The project uses Semantic Versioning (SemVer) for all application, chart, and dependency versions.

For Lana Bank, the application version is 0.12.3 with chart version 0.1.1-dev. Dependencies include PostgreSQL (Bitnami) 16.4.13, Oathkeeper 0.49.2, Keycloakx 7.1.1, Dagster 1.12.1, and OAuth2 Proxy 7.13.0.

For Galoy Dependencies, the chart version is 0.10.20-dev with dependencies including cert-manager v1.19.1, ingress-nginx 4.14.0, kube-monkey 1.5.2, and opentelemetry-collector 0.138.1.

### 14.9 Recommended Configuration

Pod resources vary by component. Workers have resources defined in section 11.1.4. The Ingress Controller has resources defined in `ingress-scaling.yml`, the OpenTelemetry Collector in `otel-scaling.yml`, and Kube Monkey uses minimal resources (5m CPU, 25Mi memory).

### 14.10 Network Architecture (Networking)

#### 14.10.1 Network Topology

In GCP, the private network (VPC) uses REGIONAL routing mode with name `{name_prefix}-vpc` and auto-create subnets disabled for manual control. The DMZ subnet (`{name_prefix}-dmz`) uses CIDR `{network_prefix}.0.0/24` (example: 10.1.0.0/24) for bastion hosts and administrative access, with Private Google Access enabled. The cluster subnet (`{name_prefix}-cluster`) uses CIDR `{network_prefix}.0.0/17` (example: 10.1.0.0/17) for Kubernetes nodes, with secondary IP ranges for pods (192.168.0.0/18) and services (192.168.64.0/18), also with Private Google Access enabled. Optionally there is a subnet for Docker Host (`{name_prefix}-docker-host`) with CIDR 10.2.0.0/24 for CI/CD Docker hosts.

In Azure, the virtual network (VNet) uses name `{name_prefix}-vnet` with address space `{network_prefix}.0.0/15` (example: 10.1.0.0/15). The DMZ subnet (`{name_prefix}-dmz`) uses CIDR `{network_prefix}.0.0/24` for bastion hosts. The cluster subnet (`{name_prefix}-cluster`) hosts Kubernetes nodes (AKS) with Service CIDR 192.168.64.0/18 and DNS Service IP 192.168.64.10. The PostgreSQL subnet (`{name_prefix}-postgres`) uses CIDR `{network_prefix}.3.0/24` (example: 10.1.3.0/24) with delegation to Microsoft.DBforPostgreSQL/flexibleServers and an associated Network Security Group with rules for PostgreSQL (port 5432).

#### 14.10.2 Connectivity and NAT

In GCP, Cloud NAT is enabled to allow Internet egress from private subnets. The router (named `{name_prefix}-router`) uses NAT IP Allocation AUTO_ONLY and applies to ALL_SUBNETWORKS_ALL_IP_RANGES with BGP ASN 64514. VPC Peering is configured for GCP managed services with a /16 range reserved for Google services via servicenetworking.googleapis.com.

In Azure, Network Security Groups (NSG) provide firewall rules to control traffic, with PostgreSQL NSG allowing traffic from VirtualNetwork to port 5432. Private DNS Zones are used for name resolution of managed services, including privatelink.postgres.database.azure.com for PostgreSQL.

#### 14.10.3 Firewall Rules

In GCP, firewall rules include Intra-cluster Egress allowing communication between pods and with the master (TCP, UDP, ICMP, SCTP, ESP, AH protocols) to Master CIDR, Cluster subnet, Pods range, and Services range. Webhook Ingress allows the master to call webhooks in pods (ports 8443, 443) from Master CIDR. DMZ to Nodes allows access from bastion to cluster nodes (all protocols) from DMZ subnet.

In Azure, Network Security Groups provide security rules per subnet, with PostgreSQL allowing only traffic from VirtualNetwork.

#### 14.10.4 Private Clusters

The Kubernetes API uses private endpoints (not accessible from Internet). In GCP it is configured with `enable_private_endpoint = true` and in Azure with `private_cluster_enabled = true`. API access is restricted to bastion hosts (DMZ subnet) and authorized networks (master authorized networks in GCP). Nodes have no public IPs: in GCP with `enable_private_nodes = true` and in Azure via nodes in private subnet.

### 14.11 Access from WAN and VPN

#### 14.11.1 Public Access to Frontends (WAN)

The Ingress architecture uses NGINX Ingress Controller with LoadBalancer service type, exposing a public IP provided by the cloud provider that receives HTTP/HTTPS traffic from the Internet.

WAN traffic flow follows this sequence:
```
Internet Client → Load Balancer (Public IP) → Ingress Controller (NGINX) → Application Services (according to routing rules)
```

The Ingress configuration includes TLS/SSL with certificates managed by cert-manager (Let's Encrypt or internal CA). Configured hosts include Customer Portal (e.g., `app.example.com`), Admin Panel (e.g., `admin.example.com`), and Dagster (e.g., `dagster.example.com`). Authentication is integrated with OAuth2 Proxy and Oathkeeper, and rate limiting is configured per host (requests per minute, connections).

WAN access security includes geo-blocking (blocking unsupported countries configured in NGINX), OAuth2/OIDC authentication for access to administrative panels, TLS for all connections (HTTPS), and possibility of configuring WAF via NGINX annotations or external services.

#### 14.11.2 Access via VPN (Employees)

Several options exist for VPN access:

**Option 1: VPN Site-to-Site** - VPN configuration between the office/corporate network and the VPC/VNet. In GCP Cloud VPN or Partner VPN is used, and in Azure VPN Gateway (Site-to-Site). Advantages include direct access to internal resources without exposing services to Internet, no public IPs required for internal services, and centralized access control. Employees connected to the corporate network access automatically.

**Option 2: VPN Client (Point-to-Site)** - Client VPN configuration for remote access. In GCP, Cloud VPN doesn't support P2S natively and requires a third-party solution. In Azure, VPN Gateway (Point-to-Site) with OpenVPN or IKEv2 is used. Advantages include access from any location, certificate or username/password authentication, and no corporate network required. Remote employees connect via VPN client.

**Option 3: Bastion Host with VPN** - VPN configuration to the bastion host with port forwarding. The flow is: employee connects to VPN, VPN terminates at bastion host, and employee accesses internal services through the bastion. Advantages include granular access control, centralized auditing, and no changes to main infrastructure required.

**Option 4: Access via Bastion (SSH Tunneling)** - SSH tunnel configuration through the bastion host for administrative access and debugging. For example, tunneling to PostgreSQL database via `ssh -L localhost:5432:db-internal-ip:5432 bastion-host`.

#### 14.11.3 Security Recommendations

For WAN access, it is recommended to always use HTTPS/TLS, implement rate limiting, configure WAF (Web Application Firewall), monitor and alert on anomalous traffic, and implement strong authentication (2FA) for administrative panels.

For VPN access, it is recommended to use strong authentication (certificates + 2FA), implement network segmentation (access only to necessary resources), monitor VPN connections, rotate credentials and certificates regularly, and consider Zero Trust Network Access (ZTNA) for more granular access.

The recommended hybrid architecture exposes only services requiring public access (customer portal) as public frontends. Backends and admin panels have access only via VPN or private network. Databases are never exposed to Internet, with access only from applications within the cluster and administrators via VPN + bastion.

---

## 15. External Services

The application is designed to integrate with various external services that provide specialized functionalities. These services are not part of the deployed infrastructure but are critical components of the operational ecosystem.

**It is important to emphasize that these services must be configured externally** by the client or operations team. Lana Bank simply expects to receive the credentials, tokens, endpoints, and other configuration information necessary to integrate with these services. The application does not manage the creation, configuration, or administration of accounts in these external services; it only consumes their APIs and services once they are configured and available.

### 15.1 Sumsub for KYC/KYB

Sumsub is used for managing KYC (Know Your Customer) and KYB (Know Your Business) processes and data. This external service handles identity verification of customers and companies, including document validation, biometric verification, and regulatory compliance.

To integrate Sumsub, it is necessary to configure an account in the Sumsub service and obtain API credentials (API key, API secret), as well as the corresponding endpoints. Lana Bank expects to receive these credentials and endpoints as part of the environment configuration, and integrates with Sumsub through its API to send verification requests and receive results from onboarding and continuous verification processes.

### 15.2 Honeycomb for Observability

Honeycomb is used for aggregation and exploitation of OpenTelemetry data, as well as for generating alerts that integrate with pager/on-call management software. The system uses the OpenTelemetry (OTEL) protocol to send metrics, logs, and traces from the OpenTelemetry Collector to Honeycomb.

To integrate Honeycomb, it is necessary to configure an account in the service and obtain the API key and corresponding dataset. Lana Bank expects to receive these credentials as part of the environment configuration. Once configured, the OpenTelemetry Collector automatically sends telemetry data to the service.

It is important to note that, although Honeycomb is currently used, the application uses the standard OTEL protocol, which allows migrating to other providers that support OpenTelemetry without significant modifications to the application. Compatible alternative providers include Datadog, New Relic, Grafana Cloud, and other services that support the OTEL protocol. In all cases, external service configuration (account creation, credential obtaining, dataset configuration, etc.) must be done externally before providing credentials to Lana Bank.

Honeycomb provides data analysis capabilities through advanced queries, anomaly detection, and custom dashboard creation. Additionally, Honeycomb's alerting system integrates with pager/on-call management systems (such as PagerDuty, Opsgenie, or Zenduty) to notify the operations team about incidents and detected anomalies. Configuration of these alert integrations must also be done externally in the Honeycomb service.

### 15.3 BigQuery for Reporting Data Storage

BigQuery is used as analytical and reporting data storage. The system uses BigQuery to store transformed data from PostgreSQL operational databases, allowing analysis and reporting without impacting transactional database performance.

The application uses BigQuery in conjunction with ETL tools (Meltano) and data transformation (dbt) to load and transform data from PostgreSQL to BigQuery. The system creates datasets in BigQuery to store transformed data, and uses BigQuery connections to Cloud SQL to read data directly from PostgreSQL when necessary.

To integrate BigQuery, it is necessary to configure the service in GCP (dataset creation, permissions configuration, service account creation, etc.) and provide Lana Bank with the necessary credentials, including the service account JSON, project ID, and dataset names. The application expects to receive these credentials as part of the environment configuration.

**It is important to note that, although BigQuery is currently used, the application can be refactored to perform the same work in other analytical databases.** ETL and transformation code can be adapted to work with alternatives such as Amazon Redshift, Snowflake, Azure Synapse Analytics, or even on-premise analytical databases. The data architecture is designed so that the analytical storage layer can be swapped without significantly affecting business logic, although it will require development work to adapt connectors and transformations to the chosen new platform.

In Azure environments, where BigQuery is not available, native alternatives such as Azure Synapse Analytics or Azure Data Factory can be used to perform similar analytical storage and processing functions.

### 15.4 Additional Notes

#### 15.4.1 Configuration per Environment

Different environments will have different needs and will have to adjust to the same base architecture. Resource values, replica counts, and specific configurations may vary according to each environment's needs.

#### 15.4.2 Updates

Kubernetes versions are updated manually. Application and chart versions are managed via vendir and references to external repositories. Database updates must be carefully planned due to possible downtime. All versions follow Semantic Versioning (SemVer).
