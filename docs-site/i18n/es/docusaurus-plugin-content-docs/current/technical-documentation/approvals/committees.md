---
id: committees
title: Comités de Aprobación
sidebar_position: 2
---

# Configuración de Comités de Aprobación

Este documento describe cómo configurar y gestionar los comités de aprobación en el sistema de gobernanza.

## Concepto de Comité

Un comité es un grupo de usuarios autorizados para tomar decisiones sobre operaciones específicas. Cada comité tiene:

- **Miembros**: Usuarios con derecho a votar
- **Quórum**: Número mínimo de votos requeridos
- **Tipo de proceso**: Categoría de operaciones que puede aprobar

## Arquitectura de Comités

```
┌─────────────────────────────────────────────────────────────────┐
│                    REGISTRO DE COMITÉS                          │
│                                                                  │
│  ┌─────────────────┐                                            │
│  │ Committee       │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │  Members    │ │                                            │
│  │ │  - User A   │ │                                            │
│  │ │  - User B   │ │                                            │
│  │ │  - User C   │ │                                            │
│  │ └─────────────┘ │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │  Quorum: 2  │ │                                            │
│  │ └─────────────┘ │                                            │
│  │ ┌─────────────┐ │                                            │
│  │ │Process Type │ │                                            │
│  │ └─────────────┘ │                                            │
│  └─────────────────┘                                            │
└─────────────────────────────────────────────────────────────────┘
```

## Tipos de Comités

### Comité de Crédito

Responsable de aprobar:
- Propuestas de líneas de crédito
- Desembolsos de préstamos

### Comité de Operaciones

Responsable de aprobar:
- Retiros de clientes
- Operaciones especiales

## Gestión de Comités

### Crear un Comité

#### Desde el Panel de Administración

1. Navegar a **Configuración** > **Comités**
2. Hacer clic en **Nuevo Comité**
3. Configurar:
   - Nombre del comité
   - Tipo de proceso asociado
   - Quórum requerido
4. Agregar miembros
5. Guardar configuración

#### Via API GraphQL

```graphql
mutation CreateCommittee($input: CommitteeCreateInput!) {
  committeeCreate(input: $input) {
    committee {
      id
      name
      processType
      quorum
    }
  }
}
```

### Agregar Miembros

```graphql
mutation AddCommitteeMember($input: CommitteeMemberAddInput!) {
  committeeMemberAdd(input: $input) {
    committee {
      id
      members {
        id
        email
        role
      }
    }
  }
}
```

### Remover Miembros

```graphql
mutation RemoveCommitteeMember($input: CommitteeMemberRemoveInput!) {
  committeeMemberRemove(input: $input) {
    committee {
      id
      members {
        id
        email
      }
    }
  }
}
```

## Configuración de Quórum

El quórum define el número mínimo de votos necesarios para tomar una decisión.

### Reglas de Quórum

| Configuración | Descripción |
|---------------|-------------|
| Mayoría simple | Más del 50% de los miembros |
| Unanimidad | Todos los miembros deben votar |
| Número fijo | Cantidad específica de votos |

### Modificar Quórum

```graphql
mutation UpdateCommitteeQuorum($input: CommitteeUpdateInput!) {
  committeeUpdate(input: $input) {
    committee {
      id
      quorum
    }
  }
}
```

## Proceso de Votación

### Flujo de Votación

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│  Solicitud   │───▶│   Votación   │───▶│   Decisión   │
│  enviada     │    │   activa     │    │   tomada     │
└──────────────┘    └──────────────┘    └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │  Cada miembro│
                    │  emite voto  │
                    └──────────────┘
```

### Emitir un Voto

1. Navegar a **Aprobaciones Pendientes**
2. Seleccionar la solicitud
3. Revisar detalles
4. Hacer clic en **Aprobar** o **Rechazar**

#### Via API GraphQL

```graphql
mutation CastVote($input: VoteCastInput!) {
  voteCast(input: $input) {
    vote {
      id
      decision
      comment
      createdAt
    }
  }
}
```

## Consultas de Comités

### Listar Comités

```graphql
query ListCommittees {
  committees {
    edges {
      node {
        id
        name
        processType
        quorum
        memberCount
      }
    }
  }
}
```

### Detalle de Comité

```graphql
query GetCommittee($id: ID!) {
  committee(id: $id) {
    id
    name
    processType
    quorum
    members {
      id
      email
      role
      addedAt
    }
  }
}
```

### Historial de Decisiones

```graphql
query GetCommitteeDecisions($committeeId: ID!, $first: Int) {
  committee(id: $committeeId) {
    decisions(first: $first) {
      edges {
        node {
          id
          processType
          outcome
          votes {
            member {
              email
            }
            decision
          }
          completedAt
        }
      }
    }
  }
}
```

## Permisos Requeridos

| Operación | Permiso |
|-----------|---------|
| Crear comité | COMMITTEE_CREATE |
| Ver comités | COMMITTEE_READ |
| Modificar comité | COMMITTEE_UPDATE |
| Eliminar comité | COMMITTEE_DELETE |
| Emitir voto | VOTE_CREATE |

## Mejores Prácticas

### Configuración de Comités

1. **Separación de responsabilidades**: Crear comités específicos para cada tipo de operación
2. **Quórum adecuado**: Balancear seguridad con eficiencia operativa
3. **Documentación**: Mantener registro de cambios en la composición

### Gestión de Miembros

1. **Revisión periódica**: Auditar membresías regularmente
2. **Capacitación**: Asegurar que los miembros entiendan sus responsabilidades
3. **Respaldo**: Tener miembros suplentes disponibles

