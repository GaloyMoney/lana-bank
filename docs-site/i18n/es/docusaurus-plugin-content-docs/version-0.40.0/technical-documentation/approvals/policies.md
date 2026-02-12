---
id: policies
title: Políticas de Aprobación
sidebar_position: 3
---

# Configuración de Políticas de Aprobación

Este documento describe cómo configurar las políticas que rigen los procesos de aprobación en el sistema de gobernanza.

## Concepto de Política

Una política define las reglas y condiciones bajo las cuales se puede aprobar una operación:

- **Tipo de proceso**: Categoría de operación
- **Umbrales**: Límites para diferentes niveles de aprobación
- **Reglas de escalamiento**: Cuándo escalar a comités superiores

## Arquitectura de Políticas

```
┌─────────────────────────────────────────────────────────────────┐
│                    SISTEMA DE POLÍTICAS                         │
│                                                                  │
│  ┌─────────────────┐                                            │
│  │ ApprovalPolicy  │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │ ProcessType │ │                                            │
│  │ └─────────────┘ │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │ Thresholds  │ │                                            │
│  │ │  - Low      │ │                                            │
│  │ │  - Medium   │ │                                            │
│  │ │  - High     │ │                                            │
│  │ └─────────────┘ │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │ Committees  │ │                                            │
│  │ └─────────────┘ │                                            │
│  └─────────────────┘                                            │
└─────────────────────────────────────────────────────────────────┘
```

## Tipos de Políticas

### Política de Líneas de Crédito

Define reglas para aprobar propuestas de crédito:

| Nivel | Monto | Aprobación Requerida |
|-------|-------|----------------------|
| Bajo | < $10,000 | 1 aprobador |
| Medio | $10,000 - $100,000 | 2 aprobadores |
| Alto | > $100,000 | Comité completo |

### Política de Desembolsos

Define reglas para aprobar desembolsos:

| Nivel | Monto | Aprobación Requerida |
|-------|-------|----------------------|
| Bajo | < $5,000 | Automático |
| Medio | $5,000 - $50,000 | 1 aprobador |
| Alto | > $50,000 | 2 aprobadores |

### Política de Retiros

Define reglas para aprobar retiros:

| Nivel | Monto | Aprobación Requerida |
|-------|-------|----------------------|
| Bajo | < $1,000 | Automático |
| Medio | $1,000 - $10,000 | 1 aprobador |
| Alto | > $10,000 | Comité de operaciones |

## Configuración de Políticas

### Crear una Política

#### Via API GraphQL

```graphql
mutation CreateApprovalPolicy($input: ApprovalPolicyCreateInput!) {
  approvalPolicyCreate(input: $input) {
    policy {
      id
      processType
      thresholds {
        level
        amount
        requiredApprovals
      }
    }
  }
}
```

### Definir Umbrales

```graphql
mutation UpdatePolicyThresholds($input: PolicyThresholdUpdateInput!) {
  policyThresholdUpdate(input: $input) {
    policy {
      id
      thresholds {
        level
        amount
        requiredApprovals
        committeeId
      }
    }
  }
}
```

## Reglas de Escalamiento

### Flujo de Escalamiento

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Nivel 1    │───▶│   Nivel 2    │───▶│   Nivel 3    │
│   (Auto)     │    │ (Aprobador)  │    │  (Comité)    │
└──────────────┘    └──────────────┘    └──────────────┘
```

### Condiciones de Escalamiento

| Condición | Acción |
|-----------|--------|
| Monto excede umbral | Escalar al siguiente nivel |
| Tiempo excedido | Notificar y escalar |
| Rechazado en nivel inferior | Escalar para revisión |

## Validaciones de Política

### Pre-condiciones

Antes de iniciar un proceso de aprobación:

1. **Verificar elegibilidad**: El cliente cumple requisitos
2. **Validar límites**: La operación está dentro de límites permitidos
3. **Confirmar documentación**: Documentos requeridos están disponibles

### Durante el Proceso

1. **Verificar quórum**: Suficientes aprobadores disponibles
2. **Validar votos**: Los votos son de miembros autorizados
3. **Controlar tiempo**: El proceso no ha expirado

## Integración con Dominio

### Líneas de Crédito

```
┌─────────────────────────────────────────────────────────────────┐
│                 INTEGRACIÓN CON CRÉDITO                         │
│                                                                  │
│  CreditFacility.propose() ───▶ GovernanceSystem.startProcess() │
│                                        │                        │
│                                        ▼                        │
│                              ApprovalPolicy.evaluate()          │
│                                        │                        │
│                                        ▼                        │
│                              Committee.requestVotes()           │
│                                        │                        │
│                                        ▼                        │
│  CreditFacility.approve() ◀─── GovernanceSystem.complete()     │
└─────────────────────────────────────────────────────────────────┘
```

### Depósitos y Retiros

```
┌─────────────────────────────────────────────────────────────────┐
│                 INTEGRACIÓN CON DEPÓSITOS                       │
│                                                                  │
│  Withdrawal.initiate() ───▶ GovernanceSystem.startProcess()    │
│                                        │                        │
│                                        ▼                        │
│                              ApprovalPolicy.evaluate()          │
│                                        │                        │
│                                        ▼                        │
│                              Committee.requestVotes()           │
│                                        │                        │
│                                        ▼                        │
│  Withdrawal.approve() ◀─── GovernanceSystem.complete()         │
└─────────────────────────────────────────────────────────────────┘
```

## Ejecución Basada en Jobs

El sistema de gobernanza utiliza jobs para ejecutar las decisiones:

```
┌─────────────────────────────────────────────────────────────────┐
│                    JOBS DE GOBERNANZA                           │
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │ Process         │    │ Execute         │                    │
│  │ Approval Job    │───▶│ Decision Job    │                    │
│  └─────────────────┘    └─────────────────┘                    │
│                                │                                │
│                                ▼                                │
│                         ┌─────────────────┐                    │
│                         │ Notify          │                    │
│                         │ Stakeholders    │                    │
│                         └─────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

## Consultas de Políticas

### Listar Políticas

```graphql
query ListApprovalPolicies {
  approvalPolicies {
    edges {
      node {
        id
        processType
        isActive
        thresholds {
          level
          amount
        }
      }
    }
  }
}
```

### Detalle de Política

```graphql
query GetApprovalPolicy($id: ID!) {
  approvalPolicy(id: $id) {
    id
    processType
    thresholds {
      level
      amount
      requiredApprovals
      committee {
        id
        name
      }
    }
    createdAt
    updatedAt
  }
}
```

## Permisos Requeridos

| Operación | Permiso |
|-----------|---------|
| Crear política | POLICY_CREATE |
| Ver políticas | POLICY_READ |
| Modificar política | POLICY_UPDATE |
| Eliminar política | POLICY_DELETE |

## Auditoría de Políticas

Todas las modificaciones a políticas se registran en el sistema de auditoría:

- Quién realizó el cambio
- Qué se modificó
- Cuándo se realizó
- Valores anteriores y nuevos

