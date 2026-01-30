---
id: authentication-architecture
title: Arquitectura de Autenticación
sidebar_position: 5
---

# Autenticación y Autorización

Este documento describe la infraestructura de autenticación y autorización en el sistema Lana Bank. Cubre la configuración del proveedor de identidad (Keycloak), la autenticación en el gateway de API (Oathkeeper), la validación de tokens en los servicios backend y el modelo de control de acceso basado en roles (RBAC).

![Flujo de Autenticación](/img/architecture/authentication-flow-1.png)

## Arquitectura de Autenticación

El sistema implementa una arquitectura de autenticación por capas con gestión de identidad separada para usuarios administrativos y clientes.

```
┌─────────────────────────────────────────────────────────────────────┐
│                      Aplicaciones Frontend                          │
│  ┌─────────────────────┐    ┌─────────────────────┐                │
│  │   admin-panel       │    │   customer-portal   │                │
│  │  (Next.js + OIDC)   │    │ (Next.js + NextAuth)│                │
│  │   Puerto 3001       │    │   Puerto 3002       │                │
│  └─────────────────────┘    └─────────────────────┘                │
└─────────────────────────────────────────────────────────────────────┘
            │                            │
            │  Authorization: Bearer     │
            ▼                            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Gateway de API                                   │
│              ┌─────────────────────────┐                           │
│              │      Oathkeeper         │                           │
│              │     Puerto 4455         │                           │
│              │  (Validación JWT)       │                           │
│              └─────────────────────────┘                           │
│                          │                                         │
│                          ▼                                         │
│              ┌─────────────────────────┐                           │
│              │       Keycloak          │                           │
│              │     Puerto 8081         │                           │
│              │  (Proveedor OIDC)       │                           │
│              └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
            │
            ▼
┌─────────────────────────────────────────────────────────────────────┐
│                    Servicios Backend                                │
│  ┌─────────────────────┐    ┌─────────────────────┐                │
│  │   admin-server      │    │   customer-server   │                │
│  │   Puerto 5253       │    │   Puerto 5254       │                │
│  └─────────────────────┘    └─────────────────────┘                │
│                          │                                         │
│                          ▼                                         │
│              ┌─────────────────────────┐                           │
│              │     authz service       │                           │
│              │   (Motor RBAC)          │                           │
│              └─────────────────────────┘                           │
└─────────────────────────────────────────────────────────────────────┘
```

## Flujo de Autenticación

1. Los usuarios se autentican con Keycloak a través de sus respectivas aplicaciones frontend
2. Keycloak devuelve un token JWT firmado que contiene la identidad del usuario e información del realm
3. Las aplicaciones frontend incluyen el JWT en todas las solicitudes a la API mediante el header `Authorization`
4. Oathkeeper valida la firma del JWT y extrae la información del usuario
5. Oathkeeper reenvía las solicitudes autenticadas a los servidores GraphQL backend
6. Los servicios backend realizan verificaciones de autorización mediante el servicio `authz`

## Proveedor de Identidad Keycloak

Keycloak actúa como el proveedor de identidad centralizado, gestionando la autenticación de usuarios, las sesiones y la emisión de tokens.

### Configuración de Doble Realm

El sistema utiliza dos realms de Keycloak separados para mantener límites de seguridad:

| Realm | Propósito | Usuarios | Aplicación Cliente |
|-------|-----------|----------|-------------------|
| internal | Operaciones administrativas | Empleados del banco, administradores | admin-panel |
| customer | Operaciones de clientes | Clientes finales, prestatarios | customer-portal |

### Configuración de Keycloak

```yaml
# docker-compose.yml
keycloak:
  environment:
    KC_DB: postgres
    KC_HOSTNAME: localhost
    KC_HOSTNAME_PORT: 8081
    KC_HTTP_ENABLED: "true"
    KC_TRACING_ENABLED: "true"
    KC_TRACING_ENDPOINT: http://otel-agent:4317
```

### Endpoints de Keycloak

| Endpoint | URL | Propósito |
|----------|-----|-----------|
| Realm Admin | `http://localhost:8081/admin/internal/console` | Consola de administración |
| Realm Customer | `http://localhost:8081/admin/customer/console` | Consola de clientes |
| JWKS (Admin) | `http://localhost:8081/realms/internal/protocol/openid-connect/certs` | Claves públicas |
| JWKS (Customer) | `http://localhost:8081/realms/customer/protocol/openid-connect/certs` | Claves públicas |

## Gateway de API Oathkeeper

Oathkeeper actúa como proxy inverso que valida tokens JWT antes de reenviar solicitudes.

### Configuración de Reglas

```yaml
# oathkeeper-rules.yaml
- id: admin-graphql
  match:
    url: http://admin.localhost:4455/graphql
    methods: ["POST", "GET"]
  authenticators:
    - handler: jwt
      config:
        jwks_urls:
          - http://keycloak:8080/realms/internal/protocol/openid-connect/certs
  authorizer:
    handler: allow
  mutators:
    - handler: header
      config:
        headers:
          X-User-Id: "{{ print .Subject }}"

- id: customer-graphql
  match:
    url: http://app.localhost:4455/graphql
    methods: ["POST", "GET"]
  authenticators:
    - handler: jwt
      config:
        jwks_urls:
          - http://keycloak:8080/realms/customer/protocol/openid-connect/certs
```

### Endpoints del Gateway

| Endpoint | Destino | Autenticación |
|----------|---------|---------------|
| `admin.localhost:4455/graphql` | admin-server:5253 | Realm internal |
| `app.localhost:4455/graphql` | customer-server:5254 | Realm customer |

## Autenticación en el Panel de Administración

El panel de administración usa Keycloak JS para autenticación directa.

### Integración con Keycloak JS

```typescript
// apps/admin-panel/lib/auth.ts
import Keycloak from 'keycloak-js';

const keycloak = new Keycloak({
  url: process.env.NEXT_PUBLIC_KEYCLOAK_URL,
  realm: 'internal',
  clientId: 'admin-panel',
});

export const initKeycloak = () => {
  return keycloak.init({
    onLoad: 'login-required',
    checkLoginIframe: false,
  });
};
```

### Flujo de Autenticación Admin

1. Usuario accede al panel de administración
2. Keycloak JS detecta falta de sesión
3. Redirige a página de login de Keycloak (realm internal)
4. Usuario ingresa credenciales
5. Keycloak emite JWT y redirige de vuelta
6. Token se almacena y usa en solicitudes GraphQL

## Autenticación en el Portal de Clientes

El portal de clientes usa NextAuth.js con proveedor Keycloak.

### Integración con NextAuth

```typescript
// apps/customer-portal/app/api/auth/[...nextauth]/route.ts
import NextAuth from 'next-auth';
import KeycloakProvider from 'next-auth/providers/keycloak';

export const authOptions = {
  providers: [
    KeycloakProvider({
      clientId: process.env.KEYCLOAK_CLIENT_ID!,
      clientSecret: process.env.KEYCLOAK_CLIENT_SECRET!,
      issuer: `${process.env.KEYCLOAK_URL}/realms/customer`,
    }),
  ],
  callbacks: {
    async jwt({ token, account }) {
      if (account) {
        token.accessToken = account.access_token;
      }
      return token;
    },
  },
};
```

## Validación de Tokens en el Backend

Los servidores backend validan tokens JWT usando `RemoteJwksDecoder`.

### Extracción del Contexto de Usuario

```rust
// lana/admin-server/src/lib.rs
async fn graphql_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    req: GraphQLRequest,
) -> impl IntoResponse {
    let token = extract_bearer_token(&headers)?;
    let claims = state.jwks_decoder.decode(token).await?;

    let subject = Subject::from_claims(&claims)?;
    let auth_context = AdminAuthContext::new(subject);

    let schema = state.schema.execute(req.into_inner().data(auth_context)).await;
    GraphQLResponse::from(schema)
}
```

## Modelo de Autorización RBAC

El sistema usa Control de Acceso Basado en Roles implementado con Casbin.

### Arquitectura RBAC

```
┌──────────────┐    ┌──────────────┐    ┌──────────────┐
│   Subject    │───▶│    authz     │───▶│   Casbin     │
│  (Usuario)   │    │   Service    │    │   Engine     │
└──────────────┘    └──────────────┘    └──────────────┘
                                               │
                                               ▼
                                        ┌──────────────┐
                                        │  PostgreSQL  │
                                        │  (Políticas) │
                                        └──────────────┘
```

### Definición de Políticas

```csv
# Modelo RBAC
p, admin, credit_facility, create
p, admin, credit_facility, approve
p, operator, credit_facility, read
p, operator, deposit_account, create
p, customer, own_account, read
```

### Verificación de Permisos

```rust
// Ejemplo de verificación de autorización
pub async fn create_facility(
    &self,
    subject: &Subject,
    input: CreateFacilityInput,
) -> Result<CreditFacility, Error> {
    self.authz
        .enforce(subject, Object::CreditFacility, Action::Create)
        .await?;

    // Lógica de negocio...
}
```

## Consideraciones de Seguridad

### Seguridad de Tokens JWT

- Tokens firmados con RS256 (RSA + SHA-256)
- Tiempo de expiración configurado (15 minutos típico)
- Refresh tokens para sesiones prolongadas
- Validación de audiencia y emisor

### HTTPS en Producción

- Todo el tráfico debe usar HTTPS
- Certificados TLS válidos requeridos
- HSTS habilitado

### Separación de Realms

- Realms completamente aislados
- Diferentes claves de firma por realm
- Políticas de contraseña independientes
