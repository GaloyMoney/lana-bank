---
sidebar_position: 1
title: Referencia de API
description: Documentación de la API GraphQL para Lana Bank
---

# Referencia de API

Lana Bank expone dos APIs GraphQL para diferentes casos de uso.

## API de Administración

La **API de Administración** está diseñada para operadores y administradores del banco. Proporciona acceso completo a todas las capacidades del sistema, incluyendo:

- Gestión de clientes e incorporación
- Creación, aprobación y gestión de líneas de crédito
- Operaciones de depósito y retiro
- Contabilidad e informes financieros
- Flujos de trabajo de gobernanza y aprobación
- Gestión de custodia y colateral
- Gestión de usuarios y permisos
- Acceso a pistas de auditoría

**Endpoint**: `/admin/graphql`

[Ver Documentación de API de Administración →](/api/admin)

## API de Cliente

La **API de Cliente** está diseñada para aplicaciones orientadas al cliente, como el Portal del Cliente. Proporciona a los clientes acceso a sus propios datos:

- Información de cuenta y saldos
- Estado e historial de líneas de crédito
- Operaciones de cuenta de depósito
- Estado de verificación KYC
- Historial de transacciones

**Endpoint**: `/customer/graphql`

[Ver Documentación de API de Cliente →](/api/customer)

## Eventos de Dominio

El sistema publica **eventos de dominio** mediante el patrón de outbox transaccional para integración con sistemas externos. Estos eventos cubren:

- Gestión de acceso y usuarios
- Ciclo de vida de líneas de crédito (propuestas, activación, pagos, liquidaciones)
- Operaciones de custodia y billeteras
- Incorporación de clientes y KYC
- Operaciones de depósito y retiro
- Actualizaciones de precios
- Gobernanza y aprobaciones

[Ver Documentación de Eventos de Dominio →](/api/events)

---

## Autenticación

Ambas APIs requieren autenticación mediante tokens JWT obtenidos de Keycloak. El token debe incluirse en el encabezado `Authorization`:

```
Authorization: Bearer <token>
```
