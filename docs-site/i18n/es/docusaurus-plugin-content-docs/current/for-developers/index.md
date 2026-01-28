---
id: index
title: Guía del Desarrollador
sidebar_position: 1
---

# Guía del Desarrollador

Bienvenido a la documentación para desarrolladores de Lana. Esta sección cubre todo lo que necesitas para integrar con las APIs de Lana.

## Resumen de APIs

Lana proporciona dos APIs GraphQL:

| API | Propósito | Audiencia |
|-----|-----------|-----------|
| **[API de Administración](admin-api/)** | Gestión completa del sistema - clientes, crédito, contabilidad, configuración | Sistemas internos, aplicaciones de back-office |
| **[API de Cliente](customer-api/)** | Operaciones orientadas al cliente - información de cuenta, estado de facilidades | Portales de clientes, aplicaciones móviles |

## Conceptos Clave

### GraphQL

Ambas APIs utilizan GraphQL, proporcionando:
- Esquemas fuertemente tipados
- Consultas flexibles - solicita exactamente los datos que necesitas
- Suscripciones en tiempo real para actualizaciones en vivo

### Autenticación

Todas las solicitudes de API requieren autenticación. Consulta [Autenticación](authentication) para detalles de configuración.

### Eventos

Lana utiliza event sourcing. Puedes suscribirte a [Eventos de Dominio](events/) para notificaciones en tiempo real de cambios en el sistema.

## Enlaces Rápidos

- [Referencia de API de Administración](admin-api/) - Operaciones y tipos de administración completos
- [Referencia de API de Cliente](customer-api/) - Operaciones orientadas al cliente
- [Eventos de Dominio](events/) - Catálogo de eventos e integración de webhooks
