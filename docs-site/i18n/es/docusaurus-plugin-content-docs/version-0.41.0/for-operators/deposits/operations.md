---
id: operations
title: Operaciones de Depósito y Retiro
sidebar_position: 2
---

# Operaciones de Depósito y Retiro

Este documento describe las operaciones de depósito y retiro, incluyendo flujos de trabajo y procedimientos de aprobación.

## Operaciones de Depósito

### Registro de Depósitos

Los depósitos se registran cuando se reciben fondos externos en la cuenta del cliente.

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLUJO DE DEPÓSITO                            │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │ Recepción de │───▶│   Registro   │───▶│  Fondos      │       │
│  │    fondos    │    │  del depósito│    │  disponibles │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
└─────────────────────────────────────────────────────────────────┘
```

### Crear un Depósito

#### Desde el Panel de Administración

1. Navegar a **Clientes** > seleccionar cliente
2. Ir a la cuenta de depósito
3. Hacer clic en **Registrar Depósito**
4. Completar:
   - Monto en USD
   - Referencia externa
5. Confirmar operación

#### Via API GraphQL

```graphql
mutation RecordDeposit($input: DepositRecordInput!) {
  depositRecord(input: $input) {
    deposit {
      id
      amount
      reference
      status
      createdAt
    }
  }
}
```

Variables:
```json
{
  "input": {
    "depositAccountId": "uuid-de-la-cuenta",
    "amount": 100000,
    "reference": "REF-001"
  }
}
```

### Estados del Depósito

| Estado | Descripción |
|--------|-------------|
| PENDING | Depósito registrado, pendiente de confirmación |
| CONFIRMED | Depósito confirmado y acreditado |
| CANCELLED | Depósito cancelado |

## Operaciones de Retiro

### Flujo de Retiro

Los retiros requieren un proceso de aprobación antes de ser ejecutados.

```
┌─────────────────────────────────────────────────────────────────┐
│                    FLUJO DE RETIRO                              │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Solicitud   │───▶│  Aprobación  │───▶│  Ejecución   │       │
│  │  de retiro   │    │  requerida   │    │  del retiro  │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                              │                                   │
│                              ▼                                   │
│                      ┌──────────────┐                           │
│                      │  Rechazado   │                           │
│                      │  (opcional)  │                           │
│                      └──────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

### Iniciar un Retiro

#### Desde el Panel de Administración

1. Navegar a **Clientes** > seleccionar cliente
2. Ir a la cuenta de depósito
3. Hacer clic en **Iniciar Retiro**
4. Completar:
   - Monto en USD
   - Referencia externa
5. El retiro entra en proceso de aprobación

#### Via API GraphQL

```graphql
mutation InitiateWithdrawal($input: WithdrawalInitiateInput!) {
  withdrawalInitiate(input: $input) {
    withdrawal {
      id
      amount
      reference
      status
      createdAt
    }
  }
}
```

### Estados del Retiro

| Estado | Descripción |
|--------|-------------|
| PENDING_APPROVAL | Retiro pendiente de aprobación |
| APPROVED | Retiro aprobado |
| CONFIRMED | Retiro ejecutado y confirmado |
| DENIED | Retiro rechazado |
| CANCELLED | Retiro cancelado |

## Proceso de Aprobación de Retiros

### Integración con Gobernanza

Los retiros están sujetos al sistema de gobernanza con el tipo de proceso `APPROVE_WITHDRAWAL_PROCESS`.

```
┌─────────────────────────────────────────────────────────────────┐
│                 APROBACIÓN DE RETIRO                            │
│                                                                  │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐       │
│  │  Withdrawal  │───▶│  Governance  │───▶│  Approval    │       │
│  │  Initiate    │    │  System      │    │  Process     │       │
│  └──────────────┘    └──────────────┘    └──────────────┘       │
│                              │                                   │
│                              ▼                                   │
│                      ┌──────────────┐                           │
│                      │   Committee  │                           │
│                      │   Decision   │                           │
│                      └──────────────┘                           │
└─────────────────────────────────────────────────────────────────┘
```

### Aprobar un Retiro

1. Navegar a **Aprobaciones Pendientes**
2. Seleccionar el retiro a aprobar
3. Revisar detalles:
   - Cliente
   - Monto
   - Saldo disponible
4. Hacer clic en **Aprobar** o **Rechazar**

### Via API GraphQL

```graphql
mutation ApproveWithdrawal($input: WithdrawalApproveInput!) {
  withdrawalApprove(input: $input) {
    withdrawal {
      id
      status
    }
  }
}
```

## Integración Contable

### Asientos de Depósito

Cuando se registra un depósito, se crean los siguientes asientos:

| Cuenta | Débito | Crédito |
|--------|--------|---------|
| Efectivo (Activo) | X | |
| Depósitos de Clientes (Pasivo) | | X |

### Asientos de Retiro

Cuando se confirma un retiro:

| Cuenta | Débito | Crédito |
|--------|--------|---------|
| Depósitos de Clientes (Pasivo) | X | |
| Efectivo (Activo) | | X |

## Consultas de Saldo

### Saldo de Cuenta

```graphql
query GetAccountBalance($accountId: ID!) {
  depositAccount(id: $accountId) {
    id
    balance {
      available
      pending
      total
    }
  }
}
```

### Historial de Transacciones

```graphql
query GetTransactionHistory($accountId: ID!, $first: Int) {
  depositAccount(id: $accountId) {
    deposits(first: $first) {
      edges {
        node {
          id
          amount
          reference
          status
          createdAt
        }
      }
    }
    withdrawals(first: $first) {
      edges {
        node {
          id
          amount
          reference
          status
          createdAt
        }
      }
    }
  }
}
```

## Permisos Requeridos

| Operación | Permiso |
|-----------|---------|
| Registrar depósito | DEPOSIT_CREATE |
| Ver depósitos | DEPOSIT_READ |
| Iniciar retiro | WITHDRAWAL_CREATE |
| Aprobar retiro | WITHDRAWAL_APPROVE |
| Confirmar retiro | WITHDRAWAL_CONFIRM |

