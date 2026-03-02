---
id: index
title: Sistema de Gobernanza
sidebar_position: 1
---

# Sistema de Gobernanza y Aprobación

El sistema de gobernanza proporciona un mecanismo de aprobación estructurado para operaciones financieras críticas que requieren autorización multipartita antes de su ejecución.

```mermaid
graph LR
    subgraph DomainService["Estructura Interna del Servicio de Dominio"]
        CMD["Comando"] -->|"valida y ejecuta"| AGG["Aggregate Root<br/>(es-entity)"]
        AGG -->|"emite"| EVT["Eventos de Dominio"]
        EVT -->|"persiste en"| REPO["Repositorio"]
        EVT -->|"publica vía"| OUTBOX["Outbox Publisher"]
    end

    subgraph Infrastructure["Infraestructura"]
        REPO -->|"persiste"| PG[("PostgreSQL<br/>Event Store")]
        OUTBOX -->|"escribe"| OE[("outbox_events<br/>Tabla")]
    end
```

## Propósito

El sistema actúa como un guardián para acciones de alto riesgo:
- Propuestas de líneas de crédito
- Desembolsos de préstamos
- Retiros de clientes

## Arquitectura del Sistema

```
┌─────────────────────────────────────────────────────────────────┐
│                    SISTEMA DE GOBERNANZA                        │
│                                                                  │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │ Policy          │  │   Approval      │  │   Committee     │ │
│  │ Definitions     │  │   Processes     │  │   Registry      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Event System                          │   │
│  │              (Outbox Pattern)                            │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                  │
│                              ▼                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                 Domain Integration                       │   │
│  │    (Credit Facilities, Deposits, Withdrawals)           │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Tipos de Procesos de Aprobación

El sistema define tipos específicos de procesos para diferentes categorías de operaciones:

| Tipo de Proceso | Constante | Propósito |
|-----------------|-----------|-----------|
| Propuesta de Línea de Crédito | `APPROVE_CREDIT_FACILITY_PROPOSAL_PROCESS` | Aprobar nuevas solicitudes |
| Desembolso | `APPROVE_DISBURSAL_PROCESS` | Aprobar desembolsos |
| Retiro | `APPROVE_WITHDRAWAL_PROCESS` | Aprobar retiros de clientes |

## Ciclo de Vida del Flujo de Aprobación

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Iniciado   │───▶│  En Proceso  │───▶│   Aprobado   │
│              │    │              │    │              │
└──────────────┘    └──────────────┘    └──────────────┘
                           │
                           ▼
                    ┌──────────────┐
                    │   Rechazado  │
                    │              │
                    └──────────────┘
```

### Estados del Proceso

| Estado | Descripción |
|--------|-------------|
| PENDING | Proceso iniciado, esperando revisión |
| IN_REVIEW | Proceso bajo revisión del comité |
| APPROVED | Proceso aprobado |
| DENIED | Proceso rechazado |

## Componentes del Sistema

### Definiciones de Políticas

Las políticas definen las reglas para cada tipo de aprobación:
- Umbrales de aprobación
- Comités responsables
- Reglas de quórum

### Registro de Comités

Gestiona los comités de aprobación:
- Miembros del comité
- Roles y permisos
- Historial de decisiones

### Procesos de Aprobación

Ejecuta el flujo de aprobación:
- Validación de requisitos
- Recopilación de votos
- Ejecución de la decisión

## Documentación Relacionada

- [Configuración de Comités](committees) - Gestión de comités de aprobación
- [Políticas de Aprobación](policies) - Configuración de políticas

## Recorrido en Panel de Administración: User and Role Management

Governance operations depend on correct user-role assignments. Lana uses role-based access control
where roles map to permission sets, and effective permissions are the union across assigned roles.

**Paso 1.** Open the users list.

![Users list](/img/screenshots/current/es/user.cy.ts/1_users_list.png)

**Paso 2.** Click **Create**.

![Create user button](/img/screenshots/current/es/user.cy.ts/2_click_create_button.png)

**Paso 3.** Enter user email.

![Enter user email](/img/screenshots/current/es/user.cy.ts/3_enter_email.png)

**Paso 4.** Select initial role (example: admin role assignment).

![Assign admin role](/img/screenshots/current/es/user.cy.ts/4_assign_admin_role.png)

**Paso 5.** Submit user creation.

![Submit user creation](/img/screenshots/current/es/user.cy.ts/5_submit_creation.png)

**Paso 6.** Verify creation success.

![Verify user created](/img/screenshots/current/es/user.cy.ts/6_verify_creation.png)

**Paso 7.** Confirm user appears in list.

![User in list](/img/screenshots/current/es/user.cy.ts/7_view_in_list.png)

**Paso 8.** Open role-management for the user.

![Manage user roles](/img/screenshots/current/es/user.cy.ts/8_manage_roles.png)

**Paso 9.** Update role set/permissions.

![Update user roles](/img/screenshots/current/es/user.cy.ts/9_update_roles.png)

**Paso 10.** Verify role update success.

![Verify role update](/img/screenshots/current/es/user.cy.ts/10_verify_update.png)
