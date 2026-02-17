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
| `FacilityProposalCreated` | A credit facility proposal was created | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | A credit facility was activated | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | A credit facility was fully repaid and closed | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

### Collateral Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PendingCreditFacilityCollateralizationChanged` | Collateralization state changed for pending facility | `collateral`, `effective`, `id`, `price`, `recorded_at`, `state` |
| `FacilityCollateralUpdated` | Collateral amount was updated | `entity.adjustment`, `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.pending_credit_facility_id` |
| `FacilityCollateralizationChanged` | Collateralization state changed for active facility | `collateral`, `customer_id`, `effective`, `id`, `outstanding`, `price`, `recorded_at`, `state` |

### Payment Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DisbursalSettled` | A disbursal was settled | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement` |
| `AccrualPosted` | Interest accrual was posted | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |

### Liquidation Events

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PartialLiquidationInitiated` | A partial liquidation was initiated | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `PartialLiquidationCollateralSentOut` | Collateral was sent for liquidation | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at` |
| `PartialLiquidationProceedsReceived` | Liquidation proceeds were received | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at` |
| `PartialLiquidationCompleted` | Liquidation was completed | `credit_facility_id`, `liquidation_id` |

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalConcluded` | No description available | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | No description available | `entity.amount`, `entity.collateralization_state`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |

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
