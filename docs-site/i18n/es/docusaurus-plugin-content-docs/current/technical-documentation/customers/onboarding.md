---
id: onboarding
title: Proceso de Onboarding
sidebar_position: 2
---

# Proceso de Onboarding de Clientes

Este documento describe el flujo completo de incorporación de clientes, desde el registro inicial hasta la activación de la cuenta.

## Flujo de Onboarding

```
┌────────────────────────────────────────────────────────────────────┐
│                    1. CREACIÓN DEL CLIENTE                         │
│  ┌──────────────┐                                                  │
│  │ Admin crea   │───▶ Cliente en estado PENDING                    │
│  │   cliente    │                                                  │
│  └──────────────┘                                                  │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                    2. VERIFICACIÓN KYC                             │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐         │
│  │ Solicitud    │───▶│   Sumsub     │───▶│  Resultado   │         │
│  │   enviada    │    │  Verifica    │    │   recibido   │         │
│  └──────────────┘    └──────────────┘    └──────────────┘         │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌────────────────────────────────────────────────────────────────────┐
│                    3. APROVISIONAMIENTO                            │
│  ┌──────────────┐    ┌──────────────┐    ┌──────────────┐         │
│  │ Usuario en   │───▶│  Cuenta de   │───▶│   Cliente    │         │
│  │  Keycloak    │    │   depósito   │    │    ACTIVO    │         │
│  └──────────────┘    └──────────────┘    └──────────────┘         │
└────────────────────────────────────────────────────────────────────┘
```

## Paso 1: Creación del Cliente

### Desde el Panel de Administración

1. Navegar a **Clientes** > **Nuevo Cliente**
2. Completar información básica:
   - Email
   - Telegram ID (opcional)
   - Tipo de cliente
3. Hacer clic en **Crear**

### Datos Requeridos

| Campo | Tipo | Requerido | Descripción |
|-------|------|-----------|-------------|
| email | String | Sí | Email único del cliente |
| telegramHandle | String | No | ID de Telegram para notificaciones |
| customerType | Enum | Sí | Clasificación del cliente |

### Via API GraphQL

```graphql
mutation CreateCustomer($input: CustomerCreateInput!) {
  customerCreate(input: $input) {
    customer {
      id
      publicId
      email
      status
    }
  }
}
```

## Paso 2: Verificación KYC

### Inicio de la Verificación

Una vez creado el cliente, se puede iniciar la verificación KYC:

1. Navegar al detalle del cliente
2. Hacer clic en **Iniciar KYC**
3. Se genera un enlace de verificación de Sumsub

### Estados de KYC

| Estado | Descripción | Siguiente Acción |
|--------|-------------|------------------|
| NOT_STARTED | KYC no iniciado | Iniciar verificación |
| PENDING | Verificación en progreso | Esperar resultado |
| APPROVED | Identidad verificada | Proceder a activación |
| REJECTED | Verificación fallida | Revisar y reintentar |
| REVIEW_NEEDED | Requiere revisión manual | Revisar en Sumsub |

### Integración con Sumsub

El sistema se integra con Sumsub para verificación de identidad:

1. **Generación de enlace**: Se crea un enlace único para el cliente
2. **Verificación**: El cliente completa el proceso en Sumsub
3. **Webhook**: Sumsub notifica el resultado vía webhook
4. **Actualización**: El estado del cliente se actualiza automáticamente

### Monitoreo del Estado

```graphql
query GetCustomerKycStatus($id: ID!) {
  customer(id: $id) {
    id
    kycStatus
    applicant {
      status
      reviewResult
      createdAt
    }
  }
}
```

## Paso 3: Aprovisionamiento Automático

### Usuario en Keycloak

Cuando el KYC es aprobado, automáticamente:

1. Se crea un usuario en Keycloak (realm customer)
2. Se envía email de bienvenida con credenciales
3. El cliente puede acceder al portal

### Cuenta de Depósito

Simultáneamente se crea:

1. Cuenta de depósito en el sistema
2. Cuentas contables en el libro mayor
3. Relación cliente-cuenta establecida

### Eventos Generados

| Evento | Disparado Por | Consecuencia |
|--------|---------------|--------------|
| KycApproved | Webhook Sumsub | Inicia aprovisionamiento |
| UserCreated | Job de onboarding | Usuario listo en Keycloak |
| DepositAccountCreated | Job de onboarding | Cuenta lista para operar |
| CustomerActivated | Completar onboarding | Cliente puede operar |

## Gestión de Errores

### KYC Rechazado

Si el KYC es rechazado:

1. Revisar motivo en Sumsub
2. Contactar al cliente si es necesario
3. Reiniciar proceso de verificación si aplica

### Errores de Aprovisionamiento

Si falla el aprovisionamiento:

1. Verificar logs del sistema
2. Revisar estado de Keycloak
3. Ejecutar job de aprovisionamiento manualmente si es necesario

## Operaciones del Panel de Administración

### Lista de Clientes

- Filtrar por estado (Activo, Inactivo, Pendiente)
- Buscar por email o ID público
- Ordenar por fecha de creación

### Detalle del Cliente

- Ver información completa del perfil
- Ver estado KYC y documentos
- Ver cuentas asociadas
- Ver historial de operaciones

### Acciones Disponibles

| Acción | Descripción | Permisos Requeridos |
|--------|-------------|---------------------|
| Crear cliente | Nuevo registro | CUSTOMER_CREATE |
| Ver cliente | Consultar información | CUSTOMER_READ |
| Iniciar KYC | Comenzar verificación | CUSTOMER_UPDATE |
| Desactivar | Suspender cuenta | CUSTOMER_UPDATE |

## Recorrido en Panel de Administración: Creación de Cliente y KYC

Este recorrido refleja el flujo operativo usado en los manuales de Cypress y coincide con el
ciclo de vida de cliente en dominio (crear -> verificar -> activar).

### 1) Crear y validar datos base del cliente

**Paso 1.** Abre la lista de clientes.

![Lista de clientes](/img/screenshots/current/es/customers.cy.ts/2_list_all_prospects.png)

**Paso 2.** Haz clic en **Crear**.

![Botón crear cliente](/img/screenshots/current/es/customers.cy.ts/3_click_create_button.png)

**Paso 3.** Se abre el formulario de creación con el campo de email listo.

![Formulario de creación de cliente](/img/screenshots/current/es/customers.cy.ts/4_verify_email_input_visible.png)

**Paso 4.** Ingresa un correo único.

![Ingresar correo](/img/screenshots/current/es/customers.cy.ts/5_enter_email.png)

**Paso 5.** Ingresa un ID de Telegram único (si aplica en tu operación).

![Ingresar Telegram ID](/img/screenshots/current/es/customers.cy.ts/6_enter_telegram_handle.png)
