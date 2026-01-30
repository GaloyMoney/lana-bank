---
id: index
title: Gestión de Depósitos
sidebar_position: 1
---

# Sistema de Depósitos y Retiros

El Sistema de Depósitos y Retiros gestiona las cuentas de depósito de clientes y facilita las operaciones de depósito/retiro dentro de la plataforma.

![Arquitectura del Sistema de Depósitos](/img/architecture/deposit-flow-1.png)

## Propósito

El sistema maneja el ciclo de vida completo de los fondos del cliente:
- Creación de cuentas de depósito
- Registro de depósitos
- Procesamiento de retiros
- Flujos de trabajo de aprobación

Todas las operaciones financieras están integradas con Cala Ledger para contabilidad de partida doble.

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                       CoreDeposit                               │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │DepositAccountRepo│  │   DepositRepo   │  │  WithdrawalRepo │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    DepositLedger                         │   │
│  │              (Operaciones contables)                     │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 ApproveWithdrawal                        │   │
│  │              (Proceso de aprobación)                     │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Entidades Principales

### Cuenta de Depósito (DepositAccount)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| id | UUID | Identificador único |
| publicId | String | ID público legible |
| accountHolderId | UUID | ID del cliente titular |
| status | Enum | Estado de la cuenta |
| accountType | Enum | Tipo de cuenta |

### Depósito (Deposit)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| id | UUID | Identificador único |
| depositAccountId | UUID | Cuenta destino |
| amount | UsdCents | Monto en centavos USD |
| reference | String | Referencia externa |
| status | Enum | Estado del depósito |

### Retiro (Withdrawal)

| Campo | Tipo | Descripción |
|-------|------|-------------|
| id | UUID | Identificador único |
| depositAccountId | UUID | Cuenta origen |
| amount | UsdCents | Monto en centavos USD |
| reference | String | Referencia externa |
| status | Enum | Estado del retiro |

## Tipos de Cuenta

| Tipo | Descripción | Uso |
|------|-------------|-----|
| Individual | Cuenta personal | Clientes individuales |
| GovernmentEntity | Cuenta gubernamental | Entidades de gobierno |
| PrivateCompany | Cuenta empresarial | Empresas privadas |
| Bank | Cuenta bancaria | Instituciones financieras |
| FinancialInstitution | Cuenta institucional | Otras instituciones |
| ForeignAgencyOrSubsidiary | Cuenta foránea | Agencias extranjeras |
| NonDomiciledCompany | Cuenta no residente | Empresas no domiciliadas |

## Estados de Cuenta

| Estado | Descripción |
|--------|-------------|
| ACTIVE | Cuenta operativa |
| INACTIVE | Cuenta desactivada |
| FROZEN | Cuenta congelada |

## Documentación Relacionada

- [Operaciones de Depósito](operations) - Depósitos y retiros
