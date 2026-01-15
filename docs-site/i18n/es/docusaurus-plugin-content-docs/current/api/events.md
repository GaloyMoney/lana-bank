---
sidebar_position: 2
title: Eventos de Dominio
description: Eventos de dominio públicos publicados por Lana Bank
---

# Eventos de Dominio

Lana Bank publica eventos de dominio mediante el patrón de outbox transaccional. Estos eventos pueden ser consumidos por sistemas externos para integración, análisis y propósitos de auditoría.

Todos los eventos se serializan como JSON e incluyen metadatos para trazabilidad y ordenamiento.

---

## Estructura de Eventos

Cada evento está envuelto en un sobre con la siguiente estructura:

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

## Eventos de Acceso

Eventos relacionados con la gestión de usuarios y roles.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `UserCreated` | Se creó un nuevo usuario | `id`, `email`, `role_id` |
| `UserRemoved` | Se eliminó un usuario | `id` |
| `UserUpdatedRole` | Se cambió el rol de un usuario | `id`, `role_id` |
| `RoleCreated` | Se creó un nuevo rol | `id`, `name` |
| `RoleGainedPermissionSet` | Un rol obtuvo permisos | `id`, `permission_set_id` |
| `RoleLostPermissionSet` | Un rol perdió permisos | `id`, `permission_set_id` |

---

## Eventos de Crédito

Eventos relacionados con el ciclo de vida y operaciones de líneas de crédito.

### Ciclo de Vida de Facilidad

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `FacilityProposalCreated` | Se creó una propuesta de línea de crédito | `id`, `terms`, `amount`, `created_at` |
| `FacilityProposalApproved` | Se aprobó una propuesta | `id` |
| `FacilityActivated` | Se activó una línea de crédito | `id`, `activation_tx_id`, `activated_at`, `amount` |
| `FacilityCompleted` | Se pagó completamente y cerró una línea de crédito | `id`, `completed_at` |

### Eventos de Colateral

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `PendingCreditFacilityCollateralizationChanged` | Cambió el estado de colateralización de facilidad pendiente | `id`, `state`, `collateral`, `price`, `recorded_at`, `effective` |
| `FacilityCollateralUpdated` | Se actualizó el monto de colateral | `credit_facility_id`, `pending_credit_facility_id`, `ledger_tx_id`, `new_amount`, `abs_diff`, `action`, `recorded_at`, `effective` |
| `FacilityCollateralizationChanged` | Cambió el estado de colateralización de facilidad activa | `id`, `customer_id`, `state`, `recorded_at`, `effective`, `collateral`, `outstanding`, `price` |

### Eventos de Pago

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `FacilityRepaymentRecorded` | Se registró un pago | `credit_facility_id`, `obligation_id`, `obligation_type`, `payment_id`, `amount`, `recorded_at`, `effective` |
| `DisbursalSettled` | Se liquidó un desembolso | `credit_facility_id`, `ledger_tx_id`, `amount`, `recorded_at`, `effective` |
| `AccrualPosted` | Se registró devengamiento de intereses | `credit_facility_id`, `ledger_tx_id`, `amount`, `period`, `due_at`, `recorded_at`, `effective` |

### Eventos de Obligación

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `ObligationCreated` | Se creó una nueva obligación | `id`, `obligation_type`, `credit_facility_id`, `amount`, `due_at`, `overdue_at`, `defaulted_at`, `recorded_at`, `effective` |
| `ObligationDue` | Una obligación venció | `id`, `credit_facility_id`, `obligation_type`, `amount` |
| `ObligationOverdue` | Una obligación entró en mora | `id`, `credit_facility_id`, `amount` |
| `ObligationDefaulted` | Una obligación cayó en incumplimiento | `id`, `credit_facility_id`, `amount` |
| `ObligationCompleted` | Se pagó completamente una obligación | `id`, `credit_facility_id` |

### Eventos de Liquidación

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `PartialLiquidationInitiated` | Se inició una liquidación parcial | `liquidation_id`, `credit_facility_id`, `customer_id`, `trigger_price`, `initially_expected_to_receive`, `initially_estimated_to_liquidate` |
| `PartialLiquidationCollateralSentOut` | Se envió colateral para liquidación | `liquidation_id`, `credit_facility_id`, `amount`, `ledger_tx_id`, `recorded_at`, `effective` |
| `PartialLiquidationProceedsReceived` | Se recibieron fondos de liquidación | `liquidation_id`, `credit_facility_id`, `amount`, `payment_id`, `ledger_tx_id`, `recorded_at`, `effective` |
| `PartialLiquidationCompleted` | Se completó la liquidación | `liquidation_id`, `credit_facility_id`, `payment_id` |

---

## Eventos de Custodia

Eventos relacionados con la custodia de Bitcoin y gestión de billeteras.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `WalletBalanceChanged` | Cambió el saldo de una billetera | `id`, `new_balance`, `changed_at` |

---

## Eventos de Cliente

Eventos relacionados con el ciclo de vida del cliente y KYC.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `CustomerCreated` | Se creó un nuevo cliente | `id`, `email`, `customer_type` |
| `CustomerAccountKycVerificationUpdated` | Cambió el estado de verificación KYC | `id`, `kyc_verification`, `customer_type` |
| `CustomerEmailUpdated` | Se actualizó el email del cliente | `id`, `email` |

---

## Eventos de Depósito

Eventos relacionados con cuentas de depósito y transacciones.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `DepositAccountCreated` | Se creó una cuenta de depósito | `id`, `account_holder_id` |
| `DepositInitialized` | Se inicializó un depósito | `id`, `deposit_account_id`, `amount` |
| `WithdrawalConfirmed` | Se confirmó un retiro | `id`, `deposit_account_id`, `amount` |
| `DepositReverted` | Se revirtió un depósito | `id`, `deposit_account_id`, `amount` |
| `DepositAccountFrozen` | Se congeló una cuenta de depósito | `id`, `account_holder_id` |

---

## Eventos de Precio

Eventos relacionados con actualizaciones de precio BTC/USD.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `PriceUpdated` | Se actualizó el precio BTC/USD | `price`, `timestamp` |

**Tipo de Evento:** `core.price.price-updated` (evento efímero)

---

## Eventos de Reportes

Eventos relacionados con la generación de reportes.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `ReportCreated` | Se creó un nuevo reporte | `id` |
| `ReportRunCreated` | Se inició una ejecución de reporte | `id` |
| `ReportRunStateUpdated` | Cambió el estado de ejecución de reporte | `id` |

---

## Eventos de Gobernanza

Eventos relacionados con flujos de trabajo de aprobación.

| Evento | Descripción | Campos del Payload |
|--------|-------------|-------------------|
| `ApprovalProcessConcluded` | Se concluyó un proceso de aprobación | `id`, `process_type`, `approved`, `denied_reason`, `target_ref` |

---

## Referencia de Tipos de Eventos

Todos los tipos de eventos siguen la convención de nombres: `core.<módulo>.<nombre-evento>`

| Módulo | Prefijo de Tipo de Evento |
|--------|---------------------------|
| Access | `core.access.*` |
| Credit | `core.credit.*` |
| Custody | `core.custody.*` |
| Customer | `core.customer.*` |
| Deposit | `core.deposit.*` |
| Price | `core.price.*` |
| Report | `core.report.*` |
| Governance | `governance.*` |

---

## Consumo de Eventos

Los eventos se publican mediante el outbox transaccional y pueden consumirse a través de:

1. **Consulta directa a base de datos** - Consultar la tabla de outbox
2. **Streaming de eventos** - Integración con colas de mensajes (dependiente de la implementación)
3. **Pipelines ETL** - Mediante extracción de Meltano

Para detalles de integración, contacte al equipo de plataforma.
