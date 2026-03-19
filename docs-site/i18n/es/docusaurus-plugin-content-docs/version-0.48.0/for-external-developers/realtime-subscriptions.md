---
id: realtime-subscriptions
title: Suscripciones en Tiempo Real
sidebar_position: 5
---

# Suscripciones en Tiempo Real

Lana proporciona notificaciones en tiempo real a través de **suscripciones GraphQL** sobre WebSocket. En lugar de sondear la API para detectar cambios, su aplicación puede suscribirse a eventos específicos y recibir actualizaciones en el momento en que ocurren.

## Cómo Funciona

Las suscripciones GraphQL utilizan una conexión WebSocket persistente para enviar eventos desde el servidor a su cliente.

**Endpoint:** `ws://admin.localhost:4455/graphql` (desarrollo) o `wss://<su-dominio>/graphql` (producción)

**Protocolo:** GraphQL sobre WebSocket (`graphql-transport-ws`)

**Autenticación:** La conexión WebSocket requiere la misma autenticación JWT que las consultas GraphQL regulares. Pase el token de autorización como parámetro de conexión al iniciar el handshake de WebSocket.

### Ciclo de vida de la conexión

1. Abra una conexión WebSocket al endpoint de GraphQL
2. Envíe el mensaje `connection_init` con su token de autenticación
3. Envíe un mensaje `subscribe` con su consulta de suscripción
4. Reciba eventos a medida que ocurren
5. Envíe `complete` para cancelar la suscripción, o cierre la conexión

## Suscripciones Persistentes

Las suscripciones persistentes entregan eventos de manera confiable a través del patrón outbox. Los eventos sobreviven a los reinicios del servidor y se garantiza su entrega. Utilícelas para eventos críticos del negocio.

### KYC de Cliente Actualizado

Se dispara cuando el estado de verificación KYC de un cliente cambia (por ejemplo, de `PENDING_VERIFICATION` a `VERIFIED` o `REJECTED`).

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `kycVerification` | `KycVerification!` | Nuevo estado de verificación: `PENDING_VERIFICATION`, `VERIFIED` o `REJECTED` |
| `customer` | `Customer!` | El objeto de cliente completo con datos actualizados |

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

### Colateralización de Línea de Crédito Pendiente Actualizada

Se dispara cuando el nivel de colateralización de una línea de crédito pendiente cambia debido a movimientos de precios o depósitos de garantía.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `state` | `PendingCreditFacilityCollateralizationState!` | `FULLY_COLLATERALIZED` o `UNDER_COLLATERALIZED` |
| `collateral` | `Satoshis!` | Cantidad actual de garantía en satoshis |
| `price` | `UsdCents!` | Precio BTC/USD al momento de la actualización |
| `recordedAt` | `Timestamp!` | Cuándo se registró el evento |
| `effective` | `Date!` | Fecha efectiva del cambio de colateralización |
| `pendingCreditFacility` | `PendingCreditFacility!` | El objeto completo de línea de crédito pendiente |

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
      creditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Línea de Crédito Pendiente Completada

Se activa cuando una línea de crédito pendiente transiciona a un estado terminal (aprobada y activada, o denegada).

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `status` | `PendingCreditFacilityStatus!` | `PENDING_COLLATERALIZATION` o `COMPLETED` |
| `recordedAt` | `Timestamp!` | Cuándo se registró la finalización |
| `pendingCreditFacility` | `PendingCreditFacility!` | El objeto completo de la línea de crédito pendiente |

```graphql
subscription PendingFacilityCompleted($id: UUID!) {
  pendingCreditFacilityCompleted(pendingCreditFacilityId: $id) {
    status
    recordedAt
    pendingCreditFacility {
      pendingCreditFacilityId
      creditFacilityId
      status
      facilityAmount
    }
  }
}
```

### Propuesta de Línea de Crédito Concluida

Se activa cuando un proceso de aprobación para una propuesta de línea de crédito alcanza una decisión final (aprobada o denegada).

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

### Colateralización de Línea de Crédito Actualizada

Se activa cuando el nivel de colateralización de una línea de crédito activa cambia debido a movimientos de precios, cambios en el colateral o cambios en el saldo pendiente.

**Campos del payload:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `state` | `CollateralizationState!` | `FULLY_COLLATERALIZED`, `UNDER_MARGIN_CALL_THRESHOLD`, `UNDER_LIQUIDATION_THRESHOLD`, `NO_COLLATERAL` o `NO_EXPOSURE` |
| `collateral` | `Satoshis!` | Cantidad actual de colateral en satoshis |
| `outstandingInterest` | `UsdCents!` | Interés acumulado pendiente |
| `outstandingDisbursal` | `UsdCents!` | Principal desembolsado pendiente |
| `recordedAt` | `Timestamp!` | Cuándo se registró el evento |
| `effective` | `Date!` | Fecha efectiva del cambio |
| `price` | `UsdCents!` | Precio BTC/USD al momento de la actualización |
| `creditFacility` | `CreditFacility!` | El objeto completo de la línea de crédito |

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

Las suscripciones efímeras entregan eventos transitorios únicamente mientras un cliente está activamente suscrito. Los eventos que ocurren mientras está desconectado no se reproducen. Utilízalas para actualizaciones de interfaz de usuario y notificaciones no críticas.

### Exportación CSV de Cuenta de Libro Mayor Cargada

Se activa cuando una exportación CSV de transacciones de cuenta de libro mayor termina de cargarse y está lista para descargar.

**Campos de carga útil:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `documentId` | `UUID!` | ID del documento CSV generado, utilizado para generar un enlace de descarga |

```graphql
subscription CsvExportReady($accountId: UUID!) {
  ledgerAccountCsvExportUploaded(ledgerAccountId: $accountId) {
    documentId
  }
}
```

### Precio en Tiempo Real Actualizado

Se activa cada vez que cambia el tipo de cambio BTC/USD. No se requieren argumentos.

**Campos de carga útil:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `usdCentsPerBtc` | `UsdCents!` | Precio actual de 1 BTC en centavos de dólar |

```graphql
subscription PriceUpdates {
  realtimePriceUpdated {
    usdCentsPerBtc
  }
}
```

### Ejecución de Informe Actualizada

Se activa cuando se crea una ejecución de informe o cambia su estado (por ejemplo, de `QUEUED` a `RUNNING` a `SUCCESS` o `FAILED`). No se requieren argumentos — entrega actualizaciones para todas las ejecuciones de informes.

**Campos de carga útil:**

| Campo | Tipo | Descripción |
|-------|------|-------------|
| `reportRunId` | `UUID!` | ID de la ejecución de informe que fue actualizada |

```graphql
subscription ReportUpdates {
  reportRunUpdated {
    reportRunId
  }
}
```

## Mejores Prácticas

- **Manejo de reconexión**: Las conexiones WebSocket pueden caerse. Implementa reconexión automática con retroceso exponencial en tu cliente.
- **Procesamiento idempotente**: Las suscripciones persistentes pueden volver a entregar eventos en casos excepcionales. Diseña tus manejadores para procesar de forma segura el mismo evento más de una vez.
- **Usa suscripciones persistentes para flujos críticos**: Los cambios de KYC de clientes, transiciones de estado de facilidades crediticias y actualizaciones de garantías se entregan de manera confiable. Confía en estas para integraciones críticas del negocio.
- **Usa suscripciones efímeras para la interfaz de usuario**: Las actualizaciones de precios y notificaciones de exportación CSV son más adecuadas para retroalimentación de interfaz de usuario en tiempo real, no para procesamiento duradero.
- **Suscríbete a entidades específicas**: La mayoría de las suscripciones aceptan un ID de entidad para filtrar eventos. Suscríbete solo a las entidades que necesitas en lugar de procesar todos los eventos del lado del cliente.
