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

| Evento | Descripción | Campos de Payload |
|--------|-------------|--------------------|
| `FacilityProposalCreated` | Se creó una propuesta de facilidad de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityProposalConcluded` | Se concluyó una propuesta de facilidad de crédito | `entity.amount`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCollateralizationChanged` | Cambió el estado de colateralización para una facilidad pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `PendingCreditFacilityCompleted` | Se completó una facilidad de crédito pendiente | `entity.amount`, `entity.collateralization`, `entity.completed_at`, `entity.created_at`, `entity.customer_id`, `entity.id`, `entity.status`, `entity.terms` |
| `FacilityActivated` | Se activó una facilidad de crédito | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCompleted` | Una facilidad de crédito fue reembolsada y cerrada por completo | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `FacilityCollateralizationChanged` | Cambió el estado de colateralización para una facilidad activa | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |
| `DisbursalSettled` | Se liquidó un desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `DisbursalApprovalConcluded` | Se concluyó un proceso de aprobación de desembolso | `entity.amount`, `entity.credit_facility_id`, `entity.id`, `entity.settlement`, `entity.status` |
| `AccrualPosted` | Se registró el devengo de intereses | `entity.credit_facility_id`, `entity.due_at`, `entity.id`, `entity.period`, `entity.posting` |
| `PartialLiquidationInitiated` | Se inició una liquidación parcial | `entity.activated_at`, `entity.activation_tx_id`, `entity.amount`, `entity.collateral_id`, `entity.collateralization`, `entity.completed_at`, `entity.customer_id`, `entity.id`, `entity.liquidation_trigger` |

---

## Eventos de Colateral de Crédito

Eventos relacionados con la gestión y liquidación del colateral de las facilidades de crédito.

| Evento | Descripción | Campos de Payload |
|--------|-------------|--------------------|
| `CollateralUpdated` | Se actualizó el monto del colateral | `entity.adjustment`, `entity.amount`, `entity.id`, `entity.secured_loan_id` |
| `LiquidationCollateralSentOut` | El colateral fue enviado para liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationProceedsReceived` | Se recibieron los fondos de la liquidación | `amount`, `effective`, `ledger_tx_id`, `liquidation_id`, `payment_id`, `recorded_at`, `secured_loan_id` |
| `LiquidationCompleted` | Se completó la liquidación | `liquidation_id`, `secured_loan_id` |

---

## Eventos de Recaudación de Crédito

Eventos relacionados con obligaciones de facilidades de crédito y recaudación de pagos.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `PaymentCreated` | Se creó un pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.recorded_at` |
| `PaymentAllocationCreated` | Se creó una asignación de pago | `entity.amount`, `entity.beneficiary_id`, `entity.effective`, `entity.id`, `entity.obligation_id`, `entity.obligation_type`, `entity.recorded_at` |
| `ObligationCreated` | Se creó una nueva obligación | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDue` | Una obligación llegó a su vencimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationOverdue` | Una obligación se encuentra vencida | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationDefaulted` | Una obligación entró en incumplimiento | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |
| `ObligationCompleted` | Una obligación fue totalmente pagada | `entity.beneficiary_id`, `entity.defaulted_at`, `entity.due_at`, `entity.effective`, `entity.id`, `entity.initial_amount`, `entity.obligation_type`, `entity.outstanding_amount`, `entity.overdue_at`, `entity.recorded_at` |

---

## Eventos de Custodia

Eventos relacionados con la custodia de Bitcoin y la gestión de billeteras.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `WalletBalanceUpdated` | No hay descripción disponible | `entity.address`, `entity.balance`, `entity.id`, `entity.network` |

---

## Eventos de Cliente

Eventos relacionados con el ciclo de vida del cliente y KYC.

| Event | Description | Payload Fields |
|-------|-------------|----------------|
| `CustomerCreated` | Se creó un nuevo cliente | `entity.id`, `entity.kyc_verification`, `entity.party_id` |
| `CustomerKycUpdated` | No hay descripción disponible | `entity.id`, `entity.kyc_verification`, `entity.party_id` |
| `PartyCreated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `PartyEmailUpdated` | No hay descripción disponible | `entity.customer_type`, `entity.email`, `entity.id` |
| `ProspectCreated` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycStarted` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycPending` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectKycDeclined` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectConverted` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |
| `ProspectClosed` | No hay descripción disponible | `entity.id`, `entity.kyc_status`, `entity.party_id`, `entity.stage`, `entity.status` |

---

## Eventos de Depósito

Eventos relacionados con cuentas de depósito y transacciones.

| Evento | Descripción | Campos de Payload |
|-------|-------------|----------------|
| `DepositAccountCreated` | Se creó una cuenta de depósito | `entity.account_holder_id`, `entity.id` |
| `DepositInitialized` | Se inició un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |
| `WithdrawalConfirmed` | Se confirmó un retiro | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `WithdrawalApprovalConcluded` | No hay descripción disponible | `entity.amount`, `entity.deposit_account_id`, `entity.id`, `entity.status` |
| `DepositReverted` | Se revirtió un depósito | `entity.amount`, `entity.deposit_account_id`, `entity.id` |

---

## Eventos de Precio

Eventos relacionados con actualizaciones del precio de BTC/USD.

| Evento | Descripción | Campos de Payload |
|-------|-------------|----------------|
| `PriceUpdated` | Se actualizó el precio BTC/USD | `price`, `timestamp` |

---

## Eventos de Reporte

Eventos relacionados con la generación de reportes.

| Evento | Descripción | Campos de Payload |
|-------|-------------|----------------|
| `ReportRunCreated` | Se inició una ejecución de reporte | `entity` |
| `ReportRunStateUpdated` | Cambió el estado de ejecución de reporte | `entity` |

---

## Eventos de Gobernanza

Eventos relacionados con flujos de aprobación.

| Evento | Descripción | Campos de Payload |
|-------|-------------|----------------|
| `ApprovalProcessConcluded` | Se concluyó un proceso de aprobación | `entity.id`, `entity.process_type`, `entity.status`, `entity.target_ref` |

---

## Eventos de Tiempo

Eventos relacionados con el procesamiento de fin de día.

| Evento | Descripción | Campos de Payload |
|-------|-------------|----------------|
| `EndOfDay` | Se alcanzó el fin de día para la zona horaria configurada | `closing_time`, `day`, `timezone` |

---

## Referencia de Tipos de Evento

Todos los tipos de evento siguen la convención de nombres: `core.<module>.<event-name>`

| Módulo | Prefijo de tipo de evento |
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

## Consumiendo eventos

Los eventos se publican a traves del outbox transaccional y se pueden consumir mediante:

1. **Consulta directa a la base de datos** - Consultar la tabla de outbox
2. **Streaming de eventos** - Integracion con colas de mensajes (dependiente de la implementacion)
3. **Pipelines ETL** - Via extraccion de Meltano

Para detalles de integracion, contacte al equipo de plataforma.
