---
sidebar_position: 2
title: Domain Events
description: Public domain events published by Lana Bank
slug: /apis/events
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

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalCreated` | A credit facility proposal was created | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityProposalConcluded` | A credit facility proposal was concluded | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCollateralizationChanged` | Collateralization state changed for pending facility | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | A pending credit facility was completed | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | A credit facility was activated | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | A credit facility was fully repaid and closed | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCollateralizationChanged` | Collateralization state changed for active facility | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `DisbursalSettled` | A disbursal was settled | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `DisbursalApprovalConcluded` | A disbursal approval process was concluded | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `AccrualPosted` | Interest accrual was posted | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |
| `FacilityMatured` | No description available | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `PartialLiquidationInitiated` | A partial liquidation was initiated | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

---

## CreditCollateral Events

Events related to credit facility collateral management and liquidation.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CollateralUpdated` | Collateral amount was updated | `entity.adjustment`, `entity.amount`, `entity.id`, `entity.secured_loan_id` |
| `LiquidationCollateralSentOut` | Collateral was sent for liquidation | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationProceedsReceived` | Liquidation proceeds were received | `amount`, `collateral_id`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationCompleted` | Liquidation was completed | `liquidation_id`, `secured_loan_id` |

---

## CreditCollection Events

Events related to credit facility obligations and payment collection.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PaymentCreated` | A payment was created | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.recorded_at` |
| `PaymentAllocationCreated` | A payment allocation was created | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.obligation_id`, `entity.obligation_type`, `entity.recorded_at` |
| `ObligationCreated` | A new obligation was created | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDue` | An obligation became due | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationOverdue` | An obligation became overdue | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDefaulted` | An obligation defaulted | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationCompleted` | An obligation was fully paid | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |

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
| `CustomerCreated` | A new customer was created | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerFrozen` | A customer account was frozen, blocking financial operations | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerUnfrozen` | A previously frozen customer account was unfrozen, restoring normal operations | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerClosed` | Customer account was closed | `entity.id`, `entity.party_id`, `entity.status` |
| `PartyCreated` | No description available | `entity.customer_type`, `entity.email`, `entity.id` |
| `PartyEmailUpdated` | No description available | `entity.customer_type`, `entity.email`, `entity.id` |
| `ProspectCreated` | A new prospect was created for onboarding | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycStarted` | A prospect started KYC verification | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycPending` | A prospect's KYC verification is pending review | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycDeclined` | A prospect's KYC verification was declined | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectConverted` | A prospect was converted to a customer | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectClosed` | A prospect was closed without converting | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |

---

## Deposit Events

Events related to deposit accounts and transactions.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DepositAccountCreated` | A deposit account was created | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | A deposit was initialized | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | A withdrawal was confirmed | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `WithdrawalApprovalConcluded` | No description available | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
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

## Time Events

Events related to end-of-day processing.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `EndOfDay` | End of day was reached for the configured timezone | `closing_time`, `day`, `timezone` |

---

## Event Types Reference

All event types follow the naming convention: `core.<module>.<event-name>`

| Module | Event Type Prefix |
|--------|-------------------|
| Access | `core.access.*` |
| Credit | `core.credit.*` |
| CreditCollateral | `core.credit-collateral.*` |
| CreditCollection | `core.credit-collection.*` |
| Custody | `core.custody.*` |
| Customer | `core.customer.*` |
| Deposit | `core.deposit.*` |
| Price | `core.price.*` |
| Report | `core.report.*` |
| Governance | `governance.*` |
| Time | `core.time.*` |

---

## Consuming Events

Events are published via the transactional outbox and can be consumed through:

1. **Direct database polling** - Query the outbox table
2. **Event streaming** - Integration with message queues (implementation dependent)
3. **ETL pipelines** - Via Meltano extraction

For integration details, contact the platform team.
