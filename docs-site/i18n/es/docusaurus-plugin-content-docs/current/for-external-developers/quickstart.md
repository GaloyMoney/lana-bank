---
id: quickstart
title: Inicio Rápido
sidebar_position: 2
---

# Inicio Rápido para Desarrolladores

Comienza a usar las APIs de Lana en minutos.

## Requisitos Previos

- Credenciales de API proporcionadas por tu administrador de Lana
- Un cliente GraphQL (curl, Postman o una biblioteca cliente específica del lenguaje)
- La URL de tu instancia de Lana (por ejemplo, `https://admin.your-instance.com`)

## Paso 1: Obtener un Token de Autenticación

Lana utiliza OAuth 2.0 / OpenID Connect para la autenticación. Solicita un token del proveedor de identidad:

```bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

Extrae el `access_token` de la respuesta JSON.

## Paso 2: Realiza tu Primera Consulta

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id email } }"}' \
  https://admin.your-instance.com/graphql
```

## Paso 3: Explora el Esquema

Utiliza la introspección de GraphQL o navega por las referencias de la API para descubrir las operaciones disponibles:

- [Referencia de la API de Administración](../apis/admin-api/api-reference.mdx) — Operaciones administrativas completas
- [Referencia de la API de Cliente](../apis/customer-api/api-reference.mdx) — Operaciones de cara al cliente

## Próximos Pasos

- [Autenticación](authentication) — Gestión de tokens y flujos de actualización
- [Integración con GraphQL](graphql-integration) — Configuración de bibliotecas cliente y patrones comunes
- [Suscripciones en Tiempo Real](realtime-subscriptions) — Suscríbete a notificaciones de eventos en tiempo real
- [Eventos de Dominio](../apis/events/events.md) — Catálogo de eventos
