---
id: index
title: Gestión de Clientes
sidebar_position: 1
---

# Gestión de Clientes

El sistema de Gestión de Clientes abarca el ciclo de vida completo del cliente, desde el registro inicial y la verificación KYC hasta el estado de cuenta activa.

```mermaid
graph TD
    subgraph Frontend["Estructura del Portal de Clientes"]
        ROOT_PAGE["app/page.tsx<br/>Página Principal"]
        LAYOUT["app/layout.tsx<br/>Layout Principal"]
    end

    subgraph SharedComponents["Componentes Compartidos"]
        STORYBOOK["@storybook<br/>Biblioteca UI"]
        THEME["Theme Provider<br/>next/themes"]
        CSS["Tailwind CSS<br/>(Definición CSS)"]
    end

    subgraph GQL["Integración GraphQL"]
        GQL_COMP["components/*<br/>Componentes UI"]
        GQL_API["GraphQL API<br/>customer-server"]
    end

    subgraph Auth["Flujo de Autenticación"]
        AUTH_CFG["Auth.ts<br/>Config NextAuth"]
        AUTH_MIDDLEWARE["appAuthProvider.tsx<br/>Ruta API Auth"]
        OIDC["OIDC Provider<br/>KeycloakProvider"]
        JWT["JWT Session<br/>JwtLibrary"]
    end

    STORYBOOK --> CSS
    THEME --> CSS
    ROOT_PAGE --> GQL_COMP
    GQL_COMP --> GQL_API
    AUTH_CFG --> OIDC
    AUTH_MIDDLEWARE --> AUTH_CFG
    OIDC --> JWT
```

## Componentes del Sistema

| Componente | Módulo | Propósito |
|------------|--------|-----------|
| Gestión de Clientes | core-customer | Persistencia, perfiles y documentos |
| Procesamiento KYC | core-applicant | Integración con Sumsub |
| Onboarding de Usuarios | user-onboarding | Aprovisionamiento en Keycloak |

## Ciclo de Vida del Cliente

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│   Registro  │───▶│ Verificación│───▶│   Cuenta    │───▶│   Cliente   │
│   Inicial   │    │     KYC     │    │  de Depósito│    │   Activo    │
└─────────────┘    └─────────────┘    └─────────────┘    └─────────────┘
```

El sistema establece la capa de identidad fundamental requerida antes de que los clientes puedan acceder a productos financieros:

1. **Registro inicial**: Captura de datos básicos del cliente
2. **Verificación KYC**: Validación de identidad mediante Sumsub
3. **Cuenta de depósito**: Creación automática tras aprobación KYC
4. **Acceso a productos**: Habilitación de facilidades de crédito

## Tipos de Cliente

El sistema soporta múltiples tipos de cliente para clasificación regulatoria:

| Tipo | Descripción | Tratamiento Contable |
|------|-------------|---------------------|
| INDIVIDUAL | Persona natural | Cuentas individuales |
| GOVERNMENT_ENTITY | Organización gubernamental | Cuentas gubernamentales |
| PRIVATE_COMPANY | Corporación privada | Cuentas empresariales |
| BANK | Institución bancaria | Cuentas interbancarias |
| FINANCIAL_INSTITUTION | Empresa de servicios financieros | Cuentas institucionales |
| FOREIGN_AGENCY_OR_SUBSIDIARY | Agencia/sucursal extranjera | Cuentas foráneas |
| NON_DOMICILED_COMPANY | Corporación no domiciliada | Cuentas no residentes |

## Estados del Cliente

| Estado | Descripción |
|--------|-------------|
| ACTIVE | El cliente puede realizar operaciones |
| INACTIVE | La cuenta está inactiva |
| SUSPENDED | La cuenta está suspendida |

## Documentación Relacionada

- [Proceso de Onboarding](onboarding) - Flujo completo de incorporación
- [Gestión de Documentos](documents) - Manejo de documentos del cliente
