---
id: quickstart
title: Inicio Rápido
sidebar_position: 2
---

# Inicio Rápido para Desarrolladores

Comienza con las APIs de Lana en minutos.

## Prerrequisitos

- Credenciales de API (contacta a tu administrador de Lana)
- Un cliente GraphQL (curl, Postman, o cliente específico del lenguaje)

## Tu Primera Llamada a la API

### 1. Obtener Token de Autenticación

Lana usa OAuth 2.0 / OpenID Connect para autenticación. Solicita un token de tu servidor Keycloak:

```bash
curl -X POST \
  -d "client_id=api-client" \
  -d "client_secret=TU_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://tu-servidor-keycloak/realms/admin/protocol/openid-connect/token
```

Extrae el `access_token` de la respuesta para usarlo en las solicitudes de API.

### 2. Consultar la API de Administración

```graphql
query {
  me {
    id
    email
  }
}
```

### 3. Explorar el Esquema

Usa la introspección de GraphQL o navega la [Referencia de API de Administración](../apis/admin-api/api-reference) para descubrir las operaciones disponibles.

## Siguientes Pasos

- [Referencia de API de Administración](../apis/admin-api/api-reference) - Documentación completa de la API
- [Referencia de API de Cliente](../apis/customer-api/api-reference) - Operaciones orientadas al cliente
- [Eventos de Dominio](../apis/events/events) - Suscríbete a eventos del sistema
- [Autenticación](authentication) - Configuración detallada de autenticación
