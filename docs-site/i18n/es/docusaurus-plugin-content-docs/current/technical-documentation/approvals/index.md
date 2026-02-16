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

## Recorrido en Panel de Administración: Gestión de Usuarios y Roles

Las operaciones de gobernanza dependen de asignaciones correctas de rol. Lana usa RBAC, donde los
roles agrupan permission sets y los permisos efectivos se componen por unión.

**Paso 1.** Abre la lista de usuarios.

![Lista de usuarios](/img/screenshots/current/es/user.cy.ts/1_users_list.png)

**Paso 2.** Haz clic en **Crear**.

![Botón crear usuario](/img/screenshots/current/es/user.cy.ts/2_click_create_button.png)

**Paso 3.** Ingresa correo del usuario.

![Ingresar correo usuario](/img/screenshots/current/es/user.cy.ts/3_enter_email.png)

**Paso 4.** Selecciona rol inicial (ejemplo: admin).

![Asignar rol admin](/img/screenshots/current/es/user.cy.ts/4_assign_admin_role.png)

**Paso 5.** Envía creación de usuario.

![Enviar creación usuario](/img/screenshots/current/es/user.cy.ts/5_submit_creation.png)

**Paso 6.** Verifica creación exitosa.

![Verificar usuario creado](/img/screenshots/current/es/user.cy.ts/6_verify_creation.png)

**Paso 7.** Confirma que aparece en la lista.

![Usuario en lista](/img/screenshots/current/es/user.cy.ts/7_view_in_list.png)

**Paso 8.** Abre gestión de roles del usuario.

![Gestionar roles](/img/screenshots/current/es/user.cy.ts/8_manage_roles.png)

**Paso 9.** Actualiza el set de roles/permisos.

![Actualizar roles](/img/screenshots/current/es/user.cy.ts/9_update_roles.png)

**Paso 10.** Verifica éxito de actualización.

![Verificar actualización de roles](/img/screenshots/current/es/user.cy.ts/10_verify_update.png)

