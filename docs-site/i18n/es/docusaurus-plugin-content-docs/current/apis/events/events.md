---
sidebar_position: 2
title: Eventos de Dominio
description: Eventos de dominio publicos publicados por Lana Bank
slug: /apis/events
---

# Eventos de Dominio

Lana Bank publica eventos de dominio a traves del patron de outbox transaccional. Estos eventos pueden ser consumidos por sistemas externos para integracion, analitica y auditoria.

Todos los eventos se serializan como JSON e incluyen metadatos para trazabilidad y ordenamiento.

---

## Estructura del Evento

Cada evento esta envuelto en un sobre con la siguiente estructura:

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

Eventos relacionados con la gestion de usuarios y roles.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `UserCreated` | Se creo un nuevo usuario | `entity.email`, `entity.id`, `entity.role_id` |
| `RoleCreated` | Se creo un nuevo rol | `entity.id`, `entity.name` |

---

## Credit Events

Eventos relacionados con el ciclo de vida y operaciones de una línea de crédito.

| Evento | Descripción | Campos del payload |
|-------|-------------|--------------------|
| `FacilityProposalCreated` | Se creó una propuesta de línea de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityProposalConcluded` | Se concluyó una propuesta de línea de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCollateralizationChanged` | Cambió el estado de colateralización de una línea de crédito pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | Se completó una línea de crédito pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | Se activó una línea de crédito | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | Una línea de crédito fue totalmente pagada y cerrada | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCollateralizationChanged` | Cambió el estado de colateralización de una línea de crédito activa | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `DisbursalSettled` | Un desembolso fue liquidado | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `DisbursalApprovalConcluded` | Se concluyó el proceso de aprobación de un desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `AccrualPosted` | Se registró la acumulación de intereses | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |
| `FacilityMatured` | No hay descripción disponible | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `PartialLiquidationInitiated` | Se inició una liquidación parcial | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

---

## CreditCollateral Events

CoreCreditCollateralEvent module_description

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CollateralUpdated` | CollateralUpdated | `entity.adjustment`, `entity.amount`, `entity.id`, `entity.secured_loan_id` |
| `LiquidationCollateralSentOut` | LiquidationCollateralSentOut | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationProceedsReceived` | LiquidationProceedsReceived | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationCompleted` | LiquidationCompleted | `liquidation_id`, `secured_loan_id` |

---

## CreditCollection Events

CoreCreditCollectionEvent module_description

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PaymentCreated` | PaymentCreated | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.recorded_at` |
| `PaymentAllocationCreated` | PaymentAllocationCreated | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.obligation_id`, `entity.obligation_type`, `entity.recorded_at` |
| `ObligationCreated` | Se creo una nueva obligacion | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDue` | Una obligacion vencio | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationOverdue` | Una obligacion entro en mora | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDefaulted` | Una obligacion entro en incumplimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationCompleted` | Una obligacion fue completamente pagada | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |

---

## Custody Events

Eventos relacionados con custodia de Bitcoin y gestion de billeteras.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `WalletBalanceUpdated` | No description available | `entity.address`, `entity.balance`, `entity.id`, `entity.network` |

---

## Customer Events

Eventos relacionados con el ciclo de vida del cliente y KYC.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CustomerCreated` | Se creo un nuevo cliente | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerFrozen` | Se congelo una cuenta de cliente, bloqueando operaciones financieras | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerUnfrozen` | Se descongelo una cuenta de cliente previamente congelada, restaurando operaciones normales | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerClosed` | La cuenta del cliente fue cerrada | `entity.id`, `entity.party_id`, `entity.status` |
| `PartyCreated` | No description available | `entity.customer_type`, `entity.email`, `entity.id` |
| `PartyEmailUpdated` | No description available | `entity.customer_type`, `entity.email`, `entity.id` |
| `ProspectCreated` | Se creó un nuevo prospecto para la incorporación | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycStarted` | Un prospecto inició la verificación KYC | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycPending` | La verificación KYC de un prospecto está pendiente de revisión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycDeclined` | La verificación KYC de un prospecto fue rechazada | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectConverted` | Un prospecto fue convertido en cliente | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectClosed` | Un prospecto fue cerrado sin conversión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |

---

## Deposit Events

Eventos relacionados con cuentas de deposito y transacciones.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DepositAccountCreated` | Se creo una cuenta de deposito | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | Se inicializo un deposito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | Se confirmo un retiro | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `WithdrawalApprovalConcluded` | No description available | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `DepositReverted` | Se revirtio un deposito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |

---

## Price Events

Eventos relacionados con actualizaciones de precio BTC/USD.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PriceUpdated` | Se actualizo el precio BTC/USD | `price`, `timestamp` |

---

## Report Events

Eventos relacionados con generacion de reportes.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ReportRunCreated` | Se inicio una ejecucion de reporte | `entity` |
| `ReportRunStateUpdated` | Cambio el estado de ejecucion de reporte | `entity` |

---

## Governance Events

Eventos relacionados con flujos de aprobacion.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | Se concluyo un proceso de aprobacion | `entity.id`, `entity.process_type`, `entity.status`, `entity.target_ref` |

---

## Time Events

CoreTimeEvent module_description

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `EndOfDay` | EndOfDay | `closing_time`, `day`, `timezone` |

---

## Referencia de Tipos de Eventos

Todos los tipos de eventos siguen la convencion de nombres: `core.<module>.<event-name>`

| Modulo | Prefijo de Tipo de Evento |
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

## Consumiendo Eventos

Los eventos se publican a traves del outbox transaccional y se pueden consumir mediante:

1. **Consulta directa a la base de datos** - Consultar la tabla de outbox
2. **Streaming de eventos** - Integracion con colas de mensajes (dependiente de la implementacion)
3. **Pipelines ETL** - Via extraccion de Meltano

Para detalles de integracion, contacte al equipo de plataforma.
