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
| `UserCreated` | A new user was created | `entity.email`, `entity.id`, `entity.role_id` |
| `RoleCreated` | A new role was created | `entity.id`, `entity.name` |

---

## Credit Events

Events related to credit facility lifecycle and operations.

### Facility Lifecycle

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalCreated` | A credit facility proposal was created | `amount`, `created_at`, `id`, `terms` |
| `FacilityActivated` | A credit facility was activated | `activated_at`, `activation_tx_id`, `amount`, `id` |
| `FacilityCompleted` | A credit facility was fully repaid and closed | `completed_at`, `id` |

### Collateral Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PendingCreditFacilityCollateralizationChanged` | Collateralization state changed for pending facility | `collateral`, `effective`, `id`, `price`, `recorded_at`, `state` |
| `FacilityCollateralUpdated` | Collateral amount was updated | `abs_diff`, `credit_facility_id`, `direction`, `effective`, `ledger_tx_id`, `new_amount`, `pending_credit_facility_id`, `recorded_at` |
| `FacilityCollateralizationChanged` | Collateralization state changed for active facility | `collateral`, `customer_id`, `effective`, `id`, `outstanding`, `price`, `recorded_at`, `state` |

### Payment Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DisbursalSettled` | A disbursal was settled | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `recorded_at` |
| `AccrualPosted` | Interest accrual was posted | `amount`, `credit_facility_id`, `due_at`, `effective`, `ledger_tx_id`, `period`, `recorded_at` |

### Obligation Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ObligationCreated` | A new obligation was created | `amount`, `credit_facility_id`, `defaulted_at`, `due_at`, `effective`, `id`, `obligation_type`, `overdue_at`, `recorded_at` |
| `ObligationDue` | An obligation became due | `amount`, `credit_facility_id`, `id`, `obligation_type` |
| `ObligationOverdue` | An obligation became overdue | `amount`, `credit_facility_id`, `id` |
| `ObligationDefaulted` | An obligation defaulted | `amount`, `credit_facility_id`, `id` |
| `ObligationCompleted` | An obligation was fully paid | `credit_facility_id`, `id` |

### Liquidation Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PartialLiquidationInitiated` | A partial liquidation was initiated | `collateral_account_id`, `collateral_id`, `collateral_in_liquidation_account_id`, `credit_facility_id`, `customer_id`, `initially_estimated_to_liquidate`, `initially_expected_to_receive`, `liquidated_collateral_account_id`, `liquidation_id`, `payment_holding_account_id`, `proceeds_from_liquidation_account_id`, `trigger_price`, `uncovered_outstanding_account_id` |
| `PartialLiquidationCollateralSentOut` | Collateral was sent for liquidation | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at` |
| `PartialLiquidationProceedsReceived` | Liquidation proceeds were received | `amount`, `credit_facility_id`, `effective`, `facility_payment_holding_account_id`, `facility_proceeds_from_liquidation_account_id`, `facility_uncovered_outstanding_account_id`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at` |
| `PartialLiquidationCompleted` | Liquidation was completed | `credit_facility_id`, `liquidation_id` |

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalConcluded` | No description available | `id`, `status` |
| `PendingCreditFacilityCompleted` | No description available | `id`, `recorded_at`, `status` |
| `FacilityPaymentReceived` | No description available | `amount`, `credit_facility_id`, `effective`, `payment_id`, `recorded_at` |
| `FacilityPaymentAllocated` | No description available | `allocation_id`, `amount`, `credit_facility_id`, `effective`, `obligation_id`, `obligation_type`, `recorded_at` |

---

## Custody Events

Events related to Bitcoin custody and wallet management.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `WalletBalanceUpdated` | No description available | `entity.address`, `entity.balance`, `entity.id`, `entity.network` |

---

## Customer Events

Events related to customer lifecycle and KYC.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CustomerCreated` | A new customer was created | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |
| `CustomerKycUpdated` | No description available | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |
| `CustomerEmailUpdated` | Customer email was updated | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |

---

## Deposit Events

Events related to deposit accounts and transactions.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DepositAccountCreated` | A deposit account was created | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | A deposit was initialized | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | A withdrawal was confirmed | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `DepositReverted` | A deposit was reverted | `entity.amount`, `entity.deposit_account_id`, `entity.id` |

---

## Price Events

Events related to BTC/USD price updates.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PriceUpdated` | BTC/USD price was updated | `price`, `timestamp` |

---

## Report Events

Events related to report generation.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ReportRunCreated` | A report run was initiated | `entity` |
| `ReportRunStateUpdated` | A report run state changed | `entity` |

---

## Governance Events

Events related to approval workflows.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | An approval process was concluded | `entity.id`, `entity.process_type`, `entity.status`, `entity.target_ref` |

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
