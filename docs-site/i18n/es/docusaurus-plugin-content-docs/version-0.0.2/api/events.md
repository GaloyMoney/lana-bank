---
sidebar_position: 2
title: Eventos de Dominio
description: Eventos de dominio publicos publicados por Lana Bank
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

Eventos relacionados con el ciclo de vida y operaciones de facilidades de credito.

### Ciclo de Vida de Facilidad

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalCreated` | Se creo una propuesta de facilidad de credito | `amount`, `created_at`, `id`, `terms` |
| `FacilityActivated` | Se activo una facilidad de credito | `activated_at`, `activation_tx_id`, `amount`, `id` |
| `FacilityCompleted` | Una facilidad de credito fue totalmente pagada y cerrada | `completed_at`, `id` |

### Eventos de Colateral

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PendingCreditFacilityCollateralizationChanged` | Cambio el estado de colateralizacion para facilidad pendiente | `collateral`, `effective`, `id`, `price`, `recorded_at`, `state` |
| `FacilityCollateralUpdated` | Se actualizo el monto del colateral | `abs_diff`, `action`, `credit_facility_id`, `effective`, `ledger_tx_id`, `new_amount`, `pending_credit_facility_id`, `recorded_at` |
| `FacilityCollateralizationChanged` | Cambio el estado de colateralizacion para facilidad activa | `collateral`, `customer_id`, `effective`, `id`, `outstanding`, `price`, `recorded_at`, `state` |

### Eventos de Pago

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DisbursalSettled` | Se liquido un desembolso | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `recorded_at` |
| `AccrualPosted` | Se registro el devengamiento de intereses | `amount`, `credit_facility_id`, `due_at`, `effective`, `ledger_tx_id`, `period`, `recorded_at` |

### Eventos de Obligacion

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ObligationCreated` | Se creo una nueva obligacion | `amount`, `credit_facility_id`, `defaulted_at`, `due_at`, `effective`, `id`, `obligation_type`, `overdue_at`, `recorded_at` |
| `ObligationDue` | Una obligacion vencio | `amount`, `credit_facility_id`, `id`, `obligation_type` |
| `ObligationOverdue` | Una obligacion entro en mora | `amount`, `credit_facility_id`, `id` |
| `ObligationDefaulted` | Una obligacion entro en incumplimiento | `amount`, `credit_facility_id`, `id` |
| `ObligationCompleted` | Una obligacion fue completamente pagada | `credit_facility_id`, `id` |

### Eventos de Liquidacion

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PartialLiquidationInitiated` | Se inicio una liquidacion parcial | `collateral_account_id`, `collateral_in_liquidation_account_id`, `credit_facility_id`, `customer_id`, `initially_estimated_to_liquidate`, `initially_expected_to_receive`, `liquidated_collateral_account_id`, `liquidation_id`, `payment_holding_account_id`, `proceeds_from_liquidation_account_id`, `trigger_price`, `uncovered_outstanding_account_id` |
| `PartialLiquidationCollateralSentOut` | Se envio colateral para liquidacion | `amount`, `credit_facility_id`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at` |
| `PartialLiquidationProceedsReceived` | Se recibieron los ingresos de liquidacion | `amount`, `credit_facility_id`, `effective`, `facility_payment_holding_account_id`, `facility_proceeds_from_liquidation_account_id`, `facility_uncovered_outstanding_account_id`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at` |
| `PartialLiquidationCompleted` | Se completo la liquidacion | `credit_facility_id`, `liquidation_id` |

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `FacilityProposalConcluded` | No description available | `id`, `status` |
| `PendingCreditFacilityCompleted` | No description available | `id`, `recorded_at`, `status` |
| `FacilityPaymentReceived` | No description available | `amount`, `credit_facility_id`, `effective`, `payment_id`, `recorded_at` |
| `FacilityPaymentAllocated` | No description available | `allocation_id`, `amount`, `credit_facility_id`, `effective`, `obligation_id`, `obligation_type`, `recorded_at` |

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
| `CustomerCreated` | Se creo un nuevo cliente | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |
| `CustomerKycUpdated` | No description available | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |
| `CustomerEmailUpdated` | Se actualizo el email del cliente | `entity.customer_type`, `entity.email`, `entity.id`, `entity.kyc_verification` |

---

## Deposit Events

Eventos relacionados con cuentas de deposito y transacciones.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `DepositAccountCreated` | Se creo una cuenta de deposito | `account_holder_id`, `id` |
| `DepositInitialized` | Se inicializo un deposito | `amount`, `deposit_account_id`, `id` |
| `WithdrawalConfirmed` | Se confirmo un retiro | `amount`, `deposit_account_id`, `id` |
| `DepositReverted` | Se revirtio un deposito | `amount`, `deposit_account_id`, `id` |
| `DepositAccountFrozen` | Se congelo una cuenta de deposito | `account_holder_id`, `id` |

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
| `ReportCreated` | Se creo un nuevo reporte | `id` |
| `ReportRunCreated` | Se inicio una ejecucion de reporte | `id` |
| `ReportRunStateUpdated` | Cambio el estado de ejecucion de reporte | `id` |

---

## Governance Events

Eventos relacionados con flujos de aprobacion.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | Se concluyo un proceso de aprobacion | `approved`, `denied_reason`, `id`, `process_type`, `target_ref` |

---

## Referencia de Tipos de Eventos

Todos los tipos de eventos siguen la convencion de nombres: `core.<module>.<event-name>`

| Modulo | Prefijo de Tipo de Evento |
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

## Consumiendo Eventos

Los eventos se publican a traves del outbox transaccional y se pueden consumir mediante:

1. **Consulta directa a la base de datos** - Consultar la tabla de outbox
2. **Streaming de eventos** - Integracion con colas de mensajes (dependiente de la implementacion)
3. **Pipelines ETL** - Via extraccion de Meltano

Para detalles de integracion, contacte al equipo de plataforma.
