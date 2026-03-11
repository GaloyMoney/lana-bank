---
sidebar_position: 2
title: Eventos de Dominio
description: Eventos de dominio públicos publicados por Lana Bank
slug: /apis/events
---

# Eventos de Dominio

Lana Bank publica eventos de dominio mediante el patrón transactional outbox. Estos eventos pueden ser consumidos por sistemas externos para integración, análisis y fines de auditoría.

Todos los eventos se serializan como JSON e incluyen metadatos para rastreo y ordenamiento.

---

## Estructura del Evento

Cada evento se envuelve en un sobre con la siguiente estructura:

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
|-------|-------------|----------------|
| `UserCreated` | Se creó un nuevo usuario | `entity.email`, `entity.id`, `entity.role_id` |
| `RoleCreated` | Se creó un nuevo rol | `entity.id`, `entity.name` |

---

## Eventos de Crédito

Eventos relacionados con el ciclo de vida y operaciones de las líneas de crédito.

| Evento | Descripción | Campos del Payload |
|-------|-------------|----------------|
| `FacilityProposalCreated` | Se creó una propuesta de línea de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityProposalConcluded` | Se concluyó una propuesta de línea de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCollateralizationChanged` | Cambió el estado de colateralización de la línea pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | Se completó una línea de crédito pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | Se activó una línea de crédito | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | Una línea de crédito fue completamente reembolsada y cerrada | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCollateralizationChanged` | Cambió el estado de colateralización de la línea activa | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `DisbursalSettled` | Se liquidó un desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `DisbursalApprovalConcluded` | Se concluyó un proceso de aprobación de desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `AccrualPosted` | Se registró el devengo de intereses | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |
| `FacilityMatured` | No hay descripción disponible | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `PartialLiquidationInitiated` | Se inició una liquidación parcial | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

---

## Eventos de CreditCollateral

Eventos relacionados con la gestión y liquidación de garantías de líneas de crédito.

| Evento | Descripción | Campos de Datos |
|-------|-------------|----------------|
| `CollateralUpdated` | Se actualizó el monto de la garantía | `entity.adjustment`, `entity.amount`, `entity.id`, `entity.secured_loan_id` |
| `LiquidationCollateralSentOut` | Se envió la garantía para liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationProceedsReceived` | Se recibieron los fondos de la liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationCompleted` | Se completó la liquidación | `liquidation_id`, `secured_loan_id` |

---

## Eventos de CreditCollection

Eventos relacionados con obligaciones de líneas de crédito y cobro de pagos.

| Evento | Descripción | Campos de Datos |
|-------|-------------|----------------|
| `PaymentCreated` | Se creó un pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.recorded_at` |
| `PaymentAllocationCreated` | Se creó una asignación de pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.obligation_id`, `entity.obligation_type`, `entity.recorded_at` |
| `ObligationCreated` | Se creó una nueva obligación | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDue` | Una obligación venció | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationOverdue` | Una obligación quedó vencida | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDefaulted` | Una obligación entró en incumplimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationCompleted` | Se pagó completamente una obligación | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |

---

## Eventos de Custody

Eventos relacionados con la custodia de Bitcoin y la gestión de billeteras.

| Evento | Descripción | Campos de Datos |
|-------|-------------|----------------|
| `WalletBalanceUpdated` | No hay descripción disponible | `entity.address`, `entity.balance`, `entity.id`, `entity.network` |

---

## Eventos de Cliente

Eventos relacionados con el ciclo de vida del cliente y KYC.

| Evento | Descripción | Campos de Datos |
|-------|-------------|----------------|
| `CustomerCreated` | Se creó un nuevo cliente | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerFrozen` | Se congeló una cuenta de cliente, bloqueando las operaciones financieras | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerUnfrozen` | Se descongeló una cuenta de cliente previamente congelada, restaurando las operaciones normales | `entity.id`, `entity.party_id`, `entity.status` |
| `CustomerClosed` | Se cerró la cuenta del cliente | `entity.id`, `entity.party_id`, `entity.status` |
| `PartyCreated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `PartyEmailUpdated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `ProspectCreated` | Se creó un nuevo prospecto para la incorporación | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycStarted` | Un prospecto inició la verificación KYC | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycPending` | La verificación KYC de un prospecto está pendiente de revisión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectKycDeclined` | Se rechazó la verificación KYC de un prospecto | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectConverted` | Un prospecto se convirtió en cliente | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |
| `ProspectClosed` | Se cerró un prospecto sin conversión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage` |

---

## Eventos de Depósito

Eventos relacionados con cuentas de depósito y transacciones.

| Evento | Descripción | Campos de Datos |
|-------|-------------|----------------|
| `DepositAccountCreated` | Se creó una cuenta de depósito | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | Se inicializó un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | Se confirmó un retiro | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `WithdrawalApprovalConcluded` | No hay descripción disponible | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `DepositReverted` | Se revirtió un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |

---

## Eventos de Precio

Eventos relacionados con actualizaciones de precio BTC/USD.

| Evento | Descripción | Campos de Carga Útil |
|-------|-------------|----------------|
| `PriceUpdated` | El precio BTC/USD fue actualizado | `price`, `timestamp` |

---

## Eventos de Reportes

Eventos relacionados con la generación de reportes.

| Evento | Descripción | Campos de Carga Útil |
|-------|-------------|----------------|
| `ReportRunCreated` | Se inició una ejecución de reporte | `entity` |
| `ReportRunStateUpdated` | El estado de una ejecución de reporte cambió | `entity` |

---

## Eventos de Gobernanza

Eventos relacionados con flujos de trabajo de aprobación.

| Evento | Descripción | Campos de Carga Útil |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | Un proceso de aprobación fue concluido | `entity.id`, `entity.process_type`, `entity.status`, `entity.target_ref` |

---

## Eventos de Tiempo

Eventos relacionados con el procesamiento de fin de día.

| Evento | Descripción | Campos de Carga Útil |
|-------|-------------|----------------|
| `EndOfDay` | Se alcanzó el fin de día para la zona horaria configurada | `closing_time`, `day`, `timezone` |

---

## Referencia de Tipos de Eventos

Todos los tipos de eventos siguen la convención de nomenclatura: `core.<módulo>.<nombre-evento>`

| Módulo | Prefijo de Tipo de Evento |
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

## Consumo de Eventos

Los eventos se publican a través del outbox transaccional y pueden consumirse mediante:

1. **Sondeo directo de base de datos** - Consultar la tabla outbox
2. **Transmisión de eventos** - Integración con colas de mensajes (dependiente de la implementación)
3. **Canalizaciones ETL** - Mediante extracción Meltano

Para detalles de integración, contacte al equipo de plataforma.
