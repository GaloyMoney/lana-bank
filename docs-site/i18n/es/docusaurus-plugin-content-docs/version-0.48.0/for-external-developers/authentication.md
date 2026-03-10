---
id: authentication
title: Autenticación
sidebar_position: 3
---

# Autenticación

Todas las solicitudes a la API de Lana requieren autenticación mediante tokens OAuth 2.0 / OpenID Connect.

## Métodos de autenticación

### Autenticación de usuario (flujo de código de autorización)

Para aplicaciones donde los usuarios finales inician sesión:

1. Redirige al usuario al endpoint de autorización del proveedor de identidad
2. Recibe un código de autorización mediante callback
3. Intercambia el código por tokens de acceso y actualización
4. Incluye el token de acceso en las solicitudes a la API

### Autenticación de servicio (credenciales de cliente)

Para integraciones de servicio a servicio en backend:

bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=client_credentials" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

## Realizar solicitudes autenticadas

Incluye el token de acceso en el encabezado `Authorization`:

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id } }"}' \
  https://admin.your-instance.com/graphql
```

## Actualización de tokens

Los tokens de acceso caducan y deben actualizarse para mantener las sesiones.

### Duración de los tokens

| Tipo de token | Duración predeterminada |
|------------|------------------|
| Token de acceso | 5 minutos |
| Token de actualización | 30 minutos |
| Sesión | 8 horas |

### Actualizar tokens

```bash
curl -X POST \
  -d "client_id=YOUR_CLIENT_ID" \
  -d "client_secret=YOUR_CLIENT_SECRET" \
  -d "grant_type=refresh_token" \
  -d "refresh_token=YOUR_REFRESH_TOKEN" \
  https://auth.your-instance.com/realms/admin/protocol/openid-connect/token
```

## Mejores prácticas de seguridad

- **Almacenamiento de tokens**: almacena los tokens en memoria cuando sea posible, no en localStorage
- **Tokens de actualización**: usa cookies httpOnly para tokens de actualización en aplicaciones web
- **Cierre de sesión**: borra todos los tokens al cerrar sesión
- **Transporte**: usa siempre HTTPS para las solicitudes a la API
- **Rotación**: implementa la actualización automática de tokens antes de que caduquen
