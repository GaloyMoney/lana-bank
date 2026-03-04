---
id: realtime-subscriptions
title: Suscripciones en Tiempo Real
sidebar_position: 5
---

# Suscripciones en Tiempo Real

Lana proporciona notificaciones en tiempo real a través de **suscripciones GraphQL** sobre WebSocket. En lugar de consultar la API repetidamente para detectar cambios, tu aplicación puede suscribirse a eventos específicos y recibir actualizaciones en el momento en que ocurren.

## Cómo Funciona

Las suscripciones GraphQL utilizan una conexión WebSocket persistente para enviar eventos del servidor a tu cliente.

**Endpoint:** `ws://admin.localhost:4455/graphql` (desarrollo) o `wss://<tu-dominio>/graphql` (producción)

**Protocolo:** GraphQL sobre WebSocket (`graphql-transport-ws`)

**Autenticación:** La conexión WebSocket requiere la misma autenticación JWT que las consultas GraphQL regulares. Pasa el token de autorización como parámetro de conexión al iniciar el handshake de WebSocket.

### Ciclo de vida de la conexión

1. Abre una conexión WebSocket al endpoint de GraphQL
2. Envía el mensaje `connection_init` con tu token de autenticación
3. Envía un mensaje `subscribe` con tu consulta de suscripción
4. Recibe eventos a medida que ocurren
5. Envía `complete` para cancelar la suscripción, o cierra la conexión

## Suscripciones Persistentes

Las suscripciones persistentes entregan eventos de manera confiable a través del patrón outbox. Los eventos sobreviven a reinicios del servidor y se garantiza su entrega. Úsalas para eventos de negocio críticos.

### Actualización de KYC del Cliente

Se dispara cuando el estado de verificación KYC de un cliente cambia (por ejemplo, de `PENDING_VERIFICATION` a `VERIFIED` o `REJECTED`).

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `kycVerification` | `KycVerification!` | Nuevo estado de verificación: `PENDING_VERIFICATION`, `VERIFIED` o `REJECTED` |
| `customer` | `Customer!` | El objeto completo del cliente con datos actualizados |

```graphql
subscription CustomerKycUpdated($customerId: UUID!) {
  customerKycUpdated(customerId: $customerId) {
    kycVerification
    customer {
      customerId
      email
      level
    }
  }
}
```

### Actualización de Colateralización de Facilidad de Crédito Pendiente

Se dispara cuando el nivel de colateralización de una facilidad de crédito pendiente cambia debido a movimientos de precio o depósitos de colateral.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `state` | `PendingCreditFacilityCollateralizationState!` | `FULLY_COLLATERALIZED` o `UNDER_COLLATERALIZED` |
| `collateral` | `Satoshis!` | Monto actual de colateral en satoshis |
| `price` | `UsdCents!` | Precio BTC/USD al momento de la actualización |
| `recordedAt` | `Timestamp!` | Cuándo se registró el evento |
| `effective` | `Date!` | Fecha efectiva del cambio de colateralización |
| `pendingCreditFacility` | `PendingCreditFacility!` | El objeto completo de la facilidad de crédito pendiente |

```graphql
subscription PendingFacilityCollateral($id: UUID!) {
  pendingCreditFacilityCollateralizationUpdated(pendingCreditFacilityId: $id) {
    state
    collateral
    price
    recordedAt
    effective
    pendingCreditFacility {
      pendingCreditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Facilidad de Crédito Pendiente Completada

Se dispara cuando una facilidad de crédito pendiente transiciona a un estado terminal (aprobada y activada, o denegada).

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `status` | `PendingCreditFacilityStatus!` | `PENDING_COLLATERALIZATION` o `COMPLETED` |
| `recordedAt` | `Timestamp!` | Cuándo se registró la completación |
| `pendingCreditFacility` | `PendingCreditFacility!` | El objeto completo de la facilidad de crédito pendiente |

```graphql
subscription PendingFacilityCompleted($id: UUID!) {
  pendingCreditFacilityCompleted(pendingCreditFacilityId: $id) {
    status
    recordedAt
    pendingCreditFacility {
      pendingCreditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Propuesta de Facilidad de Crédito Concluida

Se dispara cuando un proceso de aprobación para una propuesta de facilidad de crédito alcanza una decisión final (aprobada o denegada).

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `status` | `CreditFacilityProposalStatus!` | Estado final: `APPROVED`, `DENIED`, `CUSTOMER_DENIED`, etc. |
| `creditFacilityProposal` | `CreditFacilityProposal!` | El objeto completo de la propuesta |

```graphql
subscription ProposalConcluded($proposalId: UUID!) {
  creditFacilityProposalConcluded(creditFacilityProposalId: $proposalId) {
    status
    creditFacilityProposal {
      creditFacilityProposalId
      facilityAmount
      status
    }
  }
}
```

### Actualización de Colateralización de Facilidad de Crédito

Se dispara cuando el nivel de colateralización de una facilidad de crédito activa cambia debido a movimientos de precio, cambios de colateral o cambios en el saldo pendiente.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `state` | `CollateralizationState!` | `FULLY_COLLATERALIZED`, `UNDER_MARGIN_CALL_THRESHOLD`, `UNDER_LIQUIDATION_THRESHOLD`, `NO_COLLATERAL` o `NO_EXPOSURE` |
| `collateral` | `Satoshis!` | Monto actual de colateral en satoshis |
| `outstandingInterest` | `UsdCents!` | Intereses acumulados pendientes |
| `outstandingDisbursal` | `UsdCents!` | Principal desembolsado pendiente |
| `recordedAt` | `Timestamp!` | Cuándo se registró el evento |
| `effective` | `Date!` | Fecha efectiva del cambio |
| `price` | `UsdCents!` | Precio BTC/USD al momento de la actualización |
| `creditFacility` | `CreditFacility!` | El objeto completo de la facilidad de crédito |

```graphql
subscription FacilityCollateral($facilityId: UUID!) {
  creditFacilityCollateralizationUpdated(creditFacilityId: $facilityId) {
    state
    collateral
    outstandingInterest
    outstandingDisbursal
    price
    recordedAt
    effective
    creditFacility {
      creditFacilityId
      status
      facilityAmount
    }
  }
}
```

## Suscripciones Efímeras

Las suscripciones efímeras entregan eventos transitorios solo mientras un cliente está activamente suscrito. Los eventos que ocurren mientras está desconectado no se reproducen. Úsalas para actualizaciones de interfaz y notificaciones no críticas.

### Exportación CSV de Cuenta Contable Cargada

Se dispara cuando una exportación CSV de transacciones de una cuenta contable termina de cargarse y está lista para descargar.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `documentId` | `UUID!` | ID del documento CSV generado, usado para generar un enlace de descarga |

```graphql
subscription CsvExportReady($accountId: UUID!) {
  ledgerAccountCsvExportUploaded(ledgerAccountId: $accountId) {
    documentId
  }
}
```

### Precio en Tiempo Real Actualizado

Se dispara cada vez que cambia el tipo de cambio BTC/USD. No requiere argumentos.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `usdCentsPerBtc` | `UsdCents!` | Precio actual de 1 BTC en centavos de USD |

```graphql
subscription PriceUpdates {
  realtimePriceUpdated {
    usdCentsPerBtc
  }
}
```

### Ejecución de Reporte Actualizada

Se dispara cuando se crea una ejecución de reporte o su estado cambia (por ejemplo, de `QUEUED` a `RUNNING` a `SUCCESS` o `FAILED`). No requiere argumentos — entrega actualizaciones para todas las ejecuciones de reportes.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `reportRunId` | `UUID!` | ID de la ejecución de reporte que fue actualizada |

```graphql
subscription ReportUpdates {
  reportRunUpdated {
    reportRunId
  }
}
```

## Mejores Prácticas

- **Manejo de reconexión**: Las conexiones WebSocket pueden caerse. Implementa reconexión automática con retroceso exponencial en tu cliente.
- **Procesamiento idempotente**: Las suscripciones persistentes pueden re-entregar eventos en casos límite. Diseña tus handlers para procesar de forma segura el mismo evento más de una vez.
- **Usa suscripciones persistentes para flujos críticos**: Los cambios de KYC de clientes, las transiciones de estado de facilidades de crédito y las actualizaciones de colateralización se entregan de manera confiable. Confía en estas para integraciones críticas de negocio.
- **Usa suscripciones efímeras para la interfaz**: Las actualizaciones de precio y las notificaciones de exportación CSV son más adecuadas para retroalimentación de interfaz en tiempo real, no para procesamiento duradero.
- **Suscríbete a entidades específicas**: La mayoría de las suscripciones aceptan un ID de entidad para filtrar eventos. Suscríbete solo a las entidades que necesitas en lugar de procesar todos los eventos del lado del cliente.
