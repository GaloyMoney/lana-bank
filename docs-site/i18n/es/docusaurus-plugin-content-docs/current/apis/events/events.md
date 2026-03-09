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

Eventos relacionados con el ciclo de vida y las operaciones de las facilidades de crédito.

| Evento | Descripción | Campos de carga útil |
|-------|-------------|----------------------|
| `FacilityProposalCreated` | Se creó una propuesta de facilidad de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityProposalConcluded` | Se finalizó una propuesta de facilidad de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCollateralizationChanged` | Se modificó el estado de colateralización de una facilidad pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | Se completó una facilidad de crédito pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | Se activó una facilidad de crédito | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | Se liquidó y cerró una facilidad de crédito | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCollateralizationChanged` | Se modificó el estado de colateralización para una facilidad activa | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `DisbursalSettled` | Se liquidó un desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `DisbursalApprovalConcluded` | Se concluyó el proceso de aprobación del desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `AccrualPosted` | Se registró la acumulación de intereses | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |
| `PartialLiquidationInitiated` | Se inició una liquidación parcial | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

---

## Eventos de CreditCollateral

Eventos relacionados con la gestión de garantías de facilidades de crédito y su liquidación.

| Evento | Descripción | Campos de carga útil |
|-------|-------------|----------------------|
| `CollateralUpdated` | Se actualizó el monto de la garantía | `entity.adjustment`, `entity.amount`, `entity.id`, `entity.secured_loan_id` |
| `LiquidationCollateralSentOut` | Se envió la garantía para liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationProceedsReceived` | Se recibieron los fondos por la liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationCompleted` | Se completó la liquidación | `liquidation_id`, `secured_loan_id` |

---

## Eventos de CreditCollection

Eventos relacionados con las obligaciones de los créditos y la recaudación de pagos.

| Evento | Descripción | Campos de datos |
|--------|-------------|----------------|
| `PaymentCreated` | Se creó un pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.recorded_at` |
| `PaymentAllocationCreated` | Se creó una asignación de pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.obligation_id`, `entity.obligation_type`, `entity.recorded_at` |
| `ObligationCreated` | Se creó una nueva obligación | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDue` | Una obligación llegó a su fecha de vencimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationOverdue` | Una obligación se encuentra vencida | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDefaulted` | Una obligación ha caído en incumplimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationCompleted` | Una obligación fue saldada por completo | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |

---

## Eventos de Custodia

Eventos relacionados con la custodia de Bitcoin y la gestión de billeteras.

| Evento | Descripción | Campos de datos |
|--------|-------------|----------------|
| `WalletBalanceUpdated` | No hay descripción disponible | `entity.address`, `entity.balance`, `entity.id`, `entity.network` |

---

## Eventos de Cliente

Eventos relacionados con el ciclo de vida del cliente y KYC.

| Evento | Descripción | Campos de datos |
|--------|-------------|----------------|
| `CustomerCreated` | Se creó un nuevo cliente | `entity.id`, `entity.kyc_verification`, `entity.party_id` |
| `CustomerKycUpdated` | No hay descripción disponible | `entity.id`, `entity.kyc_verification`, `entity.party_id` |
| `PartyCreated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `PartyEmailUpdated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `ProspectCreated` | Se creó un nuevo prospecto para onboarding | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycStarted` | Un prospecto inició la verificación KYC | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycPending` | La verificación KYC de un prospecto está pendiente de revisión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycDeclined` | Se rechazó la verificación KYC de un prospecto | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectConverted` | Un prospecto fue convertido en cliente | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectClosed` | Un prospecto fue cerrado sin conversión | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |

---

## Eventos de Depósito

Eventos relacionados con cuentas y transacciones de depósito.

| Evento | Descripción | Campos del Contenido |
|--------|-------------|---------------------|
| `DepositAccountCreated` | Se creó una cuenta de depósito | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | Se inició un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | Se confirmó un retiro | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `WithdrawalApprovalConcluded` | No hay descripción disponible | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `DepositReverted` | Se revirtió un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |

---

## Eventos de Precio

Eventos relacionados con actualizaciones del precio BTC/USD.

| Evento | Descripción | Campos del Contenido |
|--------|-------------|---------------------|
| `PriceUpdated` | El precio BTC/USD fue actualizado | `price`, `timestamp` |

---

## Eventos de Reportes

Eventos relacionados con la generación de reportes.

| Evento | Descripción | Campos del Contenido |
|--------|-------------|---------------------|
| `ReportRunCreated` | Se inició una ejecución de reporte | `entity` |
| `ReportRunStateUpdated` | El estado de la ejecución del reporte cambió | `entity` |

---

## Eventos de Gobernanza

Eventos relacionados con los flujos de trabajo de aprobación.

| Evento | Descripción | Campos del Contenido |
|--------|-------------|---------------------|
| `ApprovalProcessConcluded` | Se concluyó un proceso de aprobación | `entity.id`, `entity.process_type`, `entity.status`, `entity.target_ref` |

---

## Eventos de Tiempo

Eventos relacionados con el procesamiento de fin de día.

| Evento | Descripción | Campos del Contenido |
|--------|-------------|---------------------|
| `EndOfDay` | Se alcanzó el fin de día para la zona horaria configurada | `closing_time`, `day`, `timezone` |

---

## Referencia de Tipos de Eventos

Todos los tipos de eventos siguen la convención de nombres: `core.<module>.<event-name>`

| Módulo | Prefijo del Tipo de Evento |
|--------|---------------------------|
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

## Consumo de eventos

Los eventos se publican a traves del outbox transaccional y se pueden consumir mediante:

1. **Consulta directa a la base de datos** - Consultar la tabla de outbox
2. **Streaming de eventos** - Integracion con colas de mensajes (dependiente de la implementacion)
3. **Pipelines ETL** - Via extraccion de Meltano

Para detalles de integracion, contacte al equipo de plataforma.
