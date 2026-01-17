---
sidebar_position: 1
title: Referencia de API
description: Documentación de la API GraphQL para Lana Bank
---

import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

export const ApolloSandboxLink = ({endpoint, label}) => {
  const sandboxUrl = `https://studio.apollographql.com/sandbox/explorer?endpoint=${encodeURIComponent(endpoint)}`;
  return (
    <a href={sandboxUrl} target="_blank" rel="noopener noreferrer" style={{
      display: 'inline-flex',
      alignItems: 'center',
      gap: '0.5rem',
      padding: '0.5rem 1rem',
      backgroundColor: 'var(--ifm-color-primary)',
      color: 'white',
      borderRadius: '4px',
      textDecoration: 'none',
      fontWeight: 500,
      fontSize: '0.9rem',
      marginTop: '0.5rem',
    }}>
      {label} ↗
    </a>
  );
};

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

<ApolloSandboxLink endpoint="http://admin.localhost:4455/graphql" label="Abrir en Apollo Sandbox" />

## API de Cliente

La **API de Cliente** está diseñada para aplicaciones orientadas al cliente, como el Portal del Cliente. Proporciona a los clientes acceso a sus propios datos:

- Información de cuenta y saldos
- Estado e historial de líneas de crédito
- Operaciones de cuenta de depósito
- Estado de verificación KYC
- Historial de transacciones

**Endpoint**: `/customer/graphql`

[Ver Documentación de API de Cliente →](/api/customer)

<ApolloSandboxLink endpoint="http://app.localhost:4455/graphql" label="Abrir en Apollo Sandbox" />

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

### Uso de Apollo Sandbox

Para usar el explorador interactivo Apollo Sandbox con nuestras APIs:

1. **Obtener un token JWT** de Keycloak:
   - Para **API de Administración**: Inicia sesión en el Panel de Administración y extrae el token desde las DevTools del navegador (pestaña Network → cualquier solicitud GraphQL → Request Headers → `Authorization`)
   - Para **API de Cliente**: Inicia sesión en el Portal del Cliente y extrae el token de manera similar

2. **Abrir Apollo Sandbox** usando uno de los enlaces anteriores

3. **Agregar el encabezado Authorization** en Apollo Sandbox:
   - Haz clic en la pestaña **Headers** en la parte inferior del panel de Operación
   - Agrega un nuevo encabezado:
     - **Key**: `Authorization`
     - **Value**: `Bearer <tu-token-jwt>`

4. **Comenzar a explorar**: Ahora puedes ejecutar consultas y mutaciones contra la API

:::tip Expiración del Token
Los tokens JWT expiran después de un período de tiempo. Si recibes errores de autenticación, obtén un token nuevo iniciando sesión nuevamente en la aplicación.
:::
