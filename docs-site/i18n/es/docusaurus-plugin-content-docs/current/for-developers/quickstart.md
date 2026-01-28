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

*[Detalles de configuración de autenticación próximamente]*

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

Usa la introspección de GraphQL o navega la [Referencia de API de Administración](admin-api/) para descubrir las operaciones disponibles.

## Siguientes Pasos

- [Referencia de API de Administración](admin-api/) - Documentación completa de la API
- [Referencia de API de Cliente](customer-api/) - Operaciones orientadas al cliente
- [Eventos de Dominio](events/) - Suscríbete a eventos del sistema
- [Autenticación](authentication) - Configuración detallada de autenticación

*Guía de inicio rápido completa próximamente.*
