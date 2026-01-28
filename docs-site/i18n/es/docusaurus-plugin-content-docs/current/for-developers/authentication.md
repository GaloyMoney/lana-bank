---
id: authentication
title: Autenticación
sidebar_position: 3
---

# Autenticación

Todas las solicitudes a la API de Lana requieren autenticación.

## Resumen

Lana utiliza autenticación estándar de la industria:

- **OAuth 2.0 / OpenID Connect** para autenticación de usuarios
- **Tokens de API** para comunicación entre servicios

## Métodos de Autenticación

### Autenticación de Usuario

Para aplicaciones donde los usuarios inician sesión:

1. Redirigir al proveedor de identidad
2. Recibir código de autorización
3. Intercambiar por token de acceso
4. Incluir token en las solicitudes de API

### Autenticación de Servicio

Para integraciones de backend:

1. Obtener credenciales de servicio
2. Solicitar token de acceso
3. Incluir token en las solicitudes de API

## Realizando Solicitudes Autenticadas

Incluye el token de acceso en el encabezado Authorization:

```bash
curl -X POST \
  -H "Authorization: Bearer TU_TOKEN_DE_ACCESO" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ me { id } }"}' \
  https://tu-instancia-lana/graphql
```

## Renovación de Token

Los tokens de acceso expiran. Implementa la renovación de token para mantener las sesiones.

*[Documentación detallada de autenticación próximamente - se añadirá del manual técnico]*
