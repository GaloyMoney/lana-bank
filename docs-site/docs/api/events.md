---
sidebar_position: 2
title: Domain Events
description: Public domain events published by Lana Bank
---

# Domain Events

Lana Bank publishes domain events via the transactional outbox pattern. These events can be consumed by external systems for integration, analytics, and audit purposes.

All events are serialized as JSON and include metadata for tracing and ordering.

---

## Event Structure

Each event is wrapped in an envelope with the following structure:

```json
{
  "id": "uuid",
  "event_type": "core.credit.facility-activated",
  "payload": { ... },
  "recorded_at": "2024-01-15T10:30:00Z",
  "trace_id": "trace-uuid"
}
```

---

## Access Events

Events related to user and role management.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `UserCreated` | A new user was created | `id`, `email`, `role_id` |
| `UserRemoved` | A user was removed | `id` |
| `UserUpdatedRole` | A user's role was changed | `id`, `role_id` |
| `RoleCreated` | A new role was created | `id`, `name` |
| `RoleGainedPermissionSet` | A role gained permissions | `id`, `permission_set_id` |
| `RoleLostPermissionSet` | A role lost permissions | `id`, `permission_set_id` |

---

## Credit Events

Events related to credit facility lifecycle and operations.

### Facility Lifecycle

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalCreated` | A credit facility proposal was created | `id`, `terms`, `amount`, `created_at` |
| `FacilityProposalApproved` | A proposal was approved | `id` |
| `FacilityActivated` | A credit facility was activated | `id`, `activation_tx_id`, `activated_at`, `amount` |
| `FacilityCompleted` | A credit facility was fully repaid and closed | `id`, `completed_at` |

### Collateral Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PendingCreditFacilityCollateralizationChanged` | Collateralization state changed for pending facility | `id`, `state`, `collateral`, `price`, `recorded_at`, `effective` |
| `FacilityCollateralUpdated` | Collateral amount was updated | `credit_facility_id`, `pending_credit_facility_id`, `ledger_tx_id`, `new_amount`, `abs_diff`, `action`, `recorded_at`, `effective` |
| `FacilityCollateralizationChanged` | Collateralization state changed for active facility | `id`, `customer_id`, `state`, `recorded_at`, `effective`, `collateral`, `outstanding`, `price` |

### Payment Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityRepaymentRecorded` | A repayment was recorded | `credit_facility_id`, `obligation_id`, `obligation_type`, `payment_id`, `amount`, `recorded_at`, `effective` |
| `DisbursalSettled` | A disbursal was settled | `credit_facility_id`, `ledger_tx_id`, `amount`, `recorded_at`, `effective` |
| `AccrualPosted` | Interest accrual was posted | `credit_facility_id`, `ledger_tx_id`, `amount`, `period`, `due_at`, `recorded_at`, `effective` |

### Obligation Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ObligationCreated` | A new obligation was created | `id`, `obligation_type`, `credit_facility_id`, `amount`, `due_at`, `overdue_at`, `defaulted_at`, `recorded_at`, `effective` |
| `ObligationDue` | An obligation became due | `id`, `credit_facility_id`, `obligation_type`, `amount` |
| `ObligationOverdue` | An obligation became overdue | `id`, `credit_facility_id`, `amount` |
| `ObligationDefaulted` | An obligation defaulted | `id`, `credit_facility_id`, `amount` |
| `ObligationCompleted` | An obligation was fully paid | `id`, `credit_facility_id` |

### Liquidation Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PartialLiquidationInitiated` | A partial liquidation was initiated | `liquidation_id`, `credit_facility_id`, `customer_id`, `trigger_price`, `initially_expected_to_receive`, `initially_estimated_to_liquidate` |
| `PartialLiquidationCollateralSentOut` | Collateral was sent for liquidation | `liquidation_id`, `credit_facility_id`, `amount`, `ledger_tx_id`, `recorded_at`, `effective` |
| `PartialLiquidationProceedsReceived` | Liquidation proceeds were received | `liquidation_id`, `credit_facility_id`, `amount`, `payment_id`, `ledger_tx_id`, `recorded_at`, `effective` |
| `PartialLiquidationCompleted` | Liquidation was completed | `liquidation_id`, `credit_facility_id`, `payment_id` |

---

## Custody Events

Events related to Bitcoin custody and wallet management.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `WalletBalanceChanged` | A wallet balance changed | `id`, `new_balance`, `changed_at` |

---

## Customer Events

Events related to customer lifecycle and KYC.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CustomerCreated` | A new customer was created | `id`, `email`, `customer_type` |
| `CustomerAccountKycVerificationUpdated` | KYC verification status changed | `id`, `kyc_verification`, `customer_type` |
| `CustomerEmailUpdated` | Customer email was updated | `id`, `email` |

---

## Deposit Events

Events related to deposit accounts and transactions.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DepositAccountCreated` | A deposit account was created | `id`, `account_holder_id` |
| `DepositInitialized` | A deposit was initialized | `id`, `deposit_account_id`, `amount` |
| `WithdrawalConfirmed` | A withdrawal was confirmed | `id`, `deposit_account_id`, `amount` |
| `DepositReverted` | A deposit was reverted | `id`, `deposit_account_id`, `amount` |
| `DepositAccountFrozen` | A deposit account was frozen | `id`, `account_holder_id` |

---

## Price Events

Events related to BTC/USD price updates.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PriceUpdated` | BTC/USD price was updated | `price`, `timestamp` |

**Event Type:** `core.price.price-updated` (ephemeral event)

---

## Report Events

Events related to report generation.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ReportCreated` | A new report was created | `id` |
| `ReportRunCreated` | A report run was initiated | `id` |
| `ReportRunStateUpdated` | A report run state changed | `id` |

---

## Governance Events

Events related to approval workflows.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | An approval process was concluded | `id`, `process_type`, `approved`, `denied_reason`, `target_ref` |

---

## Event Types Reference

All event types follow the naming convention: `core.<module>.<event-name>`

| Module | Event Type Prefix |
|--------|-------------------|
| Access | `core.access.*` |
| Credit | `core.credit.*` |
| Custody | `core.custody.*` |
| Customer | `core.customer.*` |
| Deposit | `core.deposit.*` |
| Price | `core.price.*` |
| Report | `core.report.*` |
| Governance | `governance.*` |

---

## Consuming Events

Events are published via the transactional outbox and can be consumed through:

1. **Direct database polling** - Query the outbox table
2. **Event streaming** - Integration with message queues (implementation dependent)
3. **ETL pipelines** - Via Meltano extraction

For integration details, contact the platform team.
