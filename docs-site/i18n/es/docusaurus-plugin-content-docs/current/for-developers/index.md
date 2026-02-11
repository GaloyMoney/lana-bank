---
id: index
title: Guía del Desarrollador
sidebar_position: 1
---

# Guía del Desarrollador

Bienvenido a la documentación para desarrolladores de Lana. Esta sección cubre todo lo que necesitas para integrar con las APIs de Lana y desarrollar aplicaciones frontend.

## Resumen de APIs

Lana proporciona dos APIs GraphQL:

| API | Propósito | Audiencia |
|-----|-----------|-----------|
| **[API de Administración](../apis/admin-api)** | Gestión completa del sistema - clientes, crédito, contabilidad, configuración | Sistemas internos, aplicaciones de back-office |
| **[API de Cliente](../apis/customer-api)** | Operaciones orientadas al cliente - información de cuenta, estado de facilidades | Portales de clientes, aplicaciones móviles |

## Conceptos Clave

### GraphQL

Ambas APIs utilizan GraphQL, proporcionando:
- Esquemas fuertemente tipados
- Consultas flexibles - solicita exactamente los datos que necesitas
- Suscripciones en tiempo real para actualizaciones en vivo

### Autenticación

Todas las solicitudes de API requieren autenticación. Consulta [Autenticación](authentication) para detalles de configuración.

### Eventos

Lana utiliza event sourcing. Puedes suscribirte a [Eventos de Dominio](../apis/events) para notificaciones en tiempo real de cambios en el sistema.

## Integración de Clientes

- [Integración GraphQL](graphql-integration) - Configuración de Apollo Client, autenticación y patrones de uso

## Desarrollo Frontend

Las aplicaciones frontend de Lana están construidas con Next.js y React:

- [Aplicaciones Frontend](frontend/) - Visión general del stack frontend
- [Panel de Administración](frontend/admin-panel) - Arquitectura del panel admin
- [Portal del Cliente](frontend/customer-portal) - Arquitectura del portal de clientes
- [Componentes Compartidos](frontend/shared-components) - Biblioteca de UI y utilidades
- [Interfaz de Crédito](frontend/credit-ui) - Gestión de facilidades de crédito

## Referencias de API

- [Referencia de API de Administración](../apis/admin-api) - Operaciones y tipos de administración completos
- [Referencia de API de Cliente](../apis/customer-api) - Operaciones orientadas al cliente
- [Eventos de Dominio](../apis/events) - Catálogo de eventos e integración de webhooks

## Desarrollo Local

### URLs de Desarrollo

| Servicio | URL |
|----------|-----|
| Panel de Administración | http://admin.localhost:4455 |
| Portal del Cliente | http://app.localhost:4455 |
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |
| Keycloak | http://localhost:8081 |

### Comandos Útiles

```bash
# Iniciar dependencias
make start-deps

# Desarrollo de aplicaciones frontend
cd apps/admin-panel && pnpm dev
cd apps/customer-portal && pnpm dev

# Generar tipos GraphQL
pnpm codegen
```

