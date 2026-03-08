---
id: graphql-integration
title: Integración de GraphQL
sidebar_position: 4
---

# Integración de GraphQL

Esta guía cubre la integración de aplicaciones cliente con las APIs GraphQL de Lana.

## Endpoints de la API

Lana expone dos APIs GraphQL:

| API | Propósito | URL típica |
|-----|---------|-------------|
| **API de administración** | Operaciones administrativas: clientes, crédito, contabilidad | `https://admin.your-instance.com/graphql` |
| **API de cliente** | Operaciones de cara al cliente: información de cuenta, estado de facilidades | `https://app.your-instance.com/graphql` |

## Realizar solicitudes

### Con curl

```bash
curl -X POST \
  -H "Authorization: Bearer YOUR_ACCESS_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"query": "{ customers(first: 10) { edges { node { id email } } } }"}' \
  https://admin.your-instance.com/graphql
```

### Con JavaScript (Apollo Client)

```bash
npm install @apollo/client graphql
```

```typescript
import { ApolloClient, InMemoryCache, createHttpLink } from '@apollo/client';
import { setContext } from '@apollo/client/link/context';

const httpLink = createHttpLink({
  uri: 'https://admin.your-instance.com/graphql',
});

const authLink = setContext((_, { headers }) => ({
  headers: {
    ...headers,
    authorization: `Bearer ${getAccessToken()}`,
  },
}));

const client = new ApolloClient({
  link: authLink.concat(httpLink),
  cache: new InMemoryCache(),
});
```

### Con Python

```python
import requests

url = "https://admin.your-instance.com/graphql"
headers = {
    "Authorization": f"Bearer {access_token}",
    "Content-Type": "application/json",
}

query = """
query {
  customers(first: 10) {
    edges {
      node {
        id
        email
        status
      }
    }
  }
}
"""

response = requests.post(url, json={"query": query}, headers=headers)
data = response.json()
```

## Paginación

Las APIs de Lana utilizan paginación basada en cursores siguiendo la especificación Relay:

```graphql
query GetCustomers($first: Int!, $after: String) {
  customers(first: $first, after: $after) {
    edges {
      cursor
      node {
        id
        email
        status
      }
    }
    pageInfo {
      hasNextPage
      hasPreviousPage
      startCursor
      endCursor
    }
  }
}
```

Para obtener la siguiente página, pasa el valor de `endCursor` como parámetro `after`.

## Manejo de errores

Los errores de GraphQL se devuelven en el array `errors` de la respuesta:

```json
{
  "data": null,
  "errors": [
    {
      "message": "Not authorized",
      "path": ["customerCreate"],
      "extensions": {
        "code": "FORBIDDEN"
      }
    }
  ]
}
```

| Tipo de error | Descripción | Acción |
|------------|-------------|--------|
| `FORBIDDEN` | Permisos insuficientes | Verifica las credenciales y el rol de la API |
| `UNAUTHENTICATED` | Token inválido o expirado | Actualiza el token de acceso |
| `BAD_USER_INPUT` | Datos de entrada inválidos | Verifica los parámetros de la solicitud |
| `INTERNAL_SERVER_ERROR` | Error del servidor | Intenta de nuevo con retroceso exponencial |

## Encabezados requeridos

```
Authorization: Bearer <access-token>
Content-Type: application/json
```

## Referencias de la API

- [Referencia de la API de administración](../apis/admin-api/api-reference.mdx) — Operaciones y tipos administrativos completos
- [Referencia de la API de cliente](../apis/customer-api/api-reference.mdx) — Operaciones completas de cliente
