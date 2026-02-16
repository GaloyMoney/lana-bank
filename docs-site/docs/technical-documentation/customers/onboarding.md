---
id: onboarding
title: Onboarding Process
sidebar_position: 2
---

# Customer Onboarding Process

This document describes the complete customer onboarding flow, from initial registration to account activation.

## Onboarding Flow

```mermaid
graph TD
    subgraph S1["1. Customer Creation"]
        CREATE["Admin creates customer"] --> PENDING["Customer in PENDING status"]
    end
    subgraph S2["2. KYC Verification"]
        REQ["Request sent"] --> SUMSUB["Sumsub Verifies"] --> RESULT["Result received"]
    end
    subgraph S3["3. Provisioning"]
        KC["Keycloak user"] --> DEPACC["Deposit account"] --> ACTIVE["Customer ACTIVE"]
    end
    S1 --> S2 --> S3
```

## Step 1: Customer Creation

### From Admin Panel

1. Navigate to **Customers** > **New Customer**
2. Complete basic information:
   - Email
   - Telegram ID (optional)
   - Customer type
3. Click **Create**

## Step 2: KYC Verification

### Starting Verification

1. Navigate to customer detail
2. Click **Start KYC**
3. Sumsub verification link is generated

### KYC Status

| Status | Description | Next Action |
|--------|-------------|-------------|
| NOT_STARTED | KYC not initiated | Start verification |
| PENDING | Verification in progress | Wait for result |
| APPROVED | Identity verified | Proceed to activation |
| REJECTED | Verification failed | Review and retry |
| REVIEW_NEEDED | Manual review required | Review in Sumsub |

## Step 3: Automatic Provisioning

When KYC is approved, automatically:

1. Keycloak user created (customer realm)
2. Welcome email sent with credentials
3. Deposit account created
4. Customer can access portal

## Admin Panel Operations

### Customer List

- Filter by status (Active, Inactive, Pending)
- Search by email or public ID
- Sort by creation date

### Available Actions

| Action | Description | Required Permission |
|--------|-------------|---------------------|
| Create customer | New registration | CUSTOMER_CREATE |
| View customer | Query information | CUSTOMER_READ |
| Start KYC | Begin verification | CUSTOMER_UPDATE |
| Deactivate | Suspend account | CUSTOMER_UPDATE |

## Admin Panel Walkthrough: Customer Creation and KYC

This walkthrough reflects the operator flow used in Cypress manuals and aligns with the customer
domain lifecycle (create -> verify -> activate).

### 1) Create and verify customer basics

**Step 1.** Open the customers list.

![Customers list](/img/screenshots/current/en/customers.cy.ts/2_list_all_customers.png)

**Step 2.** Click **Create**.

![Click create customer](/img/screenshots/current/en/customers.cy.ts/3_click_create_button.png)

**Step 3.** Enter a unique customer email.

![Enter customer email](/img/screenshots/current/en/customers.cy.ts/5_enter_email.png)

**Step 4.** Enter a unique Telegram ID (if used by your process).

![Enter telegram id](/img/screenshots/current/en/customers.cy.ts/6_enter_telegram_id.png)

**Step 5.** Review details before submission.

![Review customer details](/img/screenshots/current/en/customers.cy.ts/7_click_review_details.png)

**Step 6.** Confirm the customer detail page and identity fields.

![Customer details page](/img/screenshots/current/en/customers.cy.ts/10_verify_email.png)

**Step 7.** Verify the customer appears in list views.

![Customer visible in list](/img/screenshots/current/en/customers.cy.ts/11_verify_customer_in_list.png)

### 2) Start and monitor KYC

The system integrates with Sumsub. Operators generate the verification link, then monitor status
changes driven by webhook updates.

**Step 8.** Open customer KYC section and generate verification link.

![Customer KYC detail section](/img/screenshots/current/en/customers.cy.ts/14_customer_kyc_details_page.png)

**Step 9.** Confirm KYC link was created.

![KYC link created](/img/screenshots/current/en/customers.cy.ts/15_kyc_link_created.png)

**Step 10.** Verify final KYC status update.

![KYC status updated](/img/screenshots/current/en/customers.cy.ts/16_kyc_status_updated.png)

