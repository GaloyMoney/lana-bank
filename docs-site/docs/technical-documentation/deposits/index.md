---
id: index
title: Deposit Management
sidebar_position: 1
---

# Deposit and Withdrawal System

The Deposit and Withdrawal System manages customer deposit accounts and facilitates deposit/withdrawal operations within the platform.

## Purpose

The system handles the complete customer funds lifecycle:
- Deposit account creation
- Deposit recording
- Withdrawal processing
- Approval workflows

All financial operations are integrated with Cala Ledger for double-entry accounting.

## Main Entities

### Deposit Account

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| publicId | String | Readable public ID |
| accountHolderId | UUID | Customer ID |
| status | Enum | Account status |
| accountType | Enum | Account type |

### Deposit

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| depositAccountId | UUID | Target account |
| amount | UsdCents | Amount in USD cents |
| reference | String | External reference |
| status | Enum | Deposit status |

### Withdrawal

| Field | Type | Description |
|-------|------|-------------|
| id | UUID | Unique identifier |
| depositAccountId | UUID | Source account |
| amount | UsdCents | Amount in USD cents |
| reference | String | External reference |
| status | Enum | Withdrawal status |

## Account Types

| Type | Description | Usage |
|------|-------------|-------|
| Individual | Personal account | Individual customers |
| GovernmentEntity | Government account | Government entities |
| PrivateCompany | Business account | Private companies |
| Bank | Bank account | Financial institutions |
| FinancialInstitution | Institutional account | Other institutions |
| ForeignAgencyOrSubsidiary | Foreign account | Foreign agencies |
| NonDomiciledCompany | Non-resident account | Non-domiciled companies |

## Account Status

| Status | Description |
|--------|-------------|
| ACTIVE | Operational account |
| INACTIVE | Deactivated account |
| FROZEN | Frozen account |

## Related Documentation

- [Deposit Operations](operations) - Deposits and withdrawals

## Admin Panel Walkthrough: Deposit Account Setup

Deposit accounts are required before transaction operations can start. In onboarding flows, the
operator may need to create one from the customer profile when none exists yet.

**Step 1.** Detect missing deposit account from customer detail banner.

![No deposit account banner](/img/screenshots/current/en/customers.cy.ts/customer_no_deposit_account_banner.png)

**Step 2.** Open the create deposit account dialog.

![Create deposit account dialog](/img/screenshots/current/en/customers.cy.ts/customer_create_deposit_account_dialog.png)

**Step 3.** Confirm deposit account creation succeeds.

![Deposit account created](/img/screenshots/current/en/customers.cy.ts/customer_deposit_account_created.png)

Operational checks after creation:
- account status is `ACTIVE`,
- customer-to-account association is correct,
- account is available for deposit/withdrawal initiation.

