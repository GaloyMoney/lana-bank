---
id: system-architecture
title: Arquitectura del Sistema
sidebar_position: 2
---

# Arquitectura del Sistema

Este documento describe la arquitectura del sistema de Lana, incluyendo capas, componentes y flujo de datos.

```mermaid
graph TD
    subgraph Client["Client Layer"]
        CP["Customer Portal<br/>(Next.js)"]
        AP["Admin Panel<br/>(Next.js)"]
    end

    subgraph Gateway["API Gateway Layer"]
        OAT["Oathkeeper<br/>(Port 4455)"]
        KC["Keycloak<br/>(Port 8081)"]
    end

    subgraph App["Application Layer (Rust)"]
        CS["customer-server<br/>(GraphQL API)<br/>Port 5254"]
        AS["admin-server<br/>(GraphQL API)<br/>Port 5253"]
        LA["lana-app<br/>(Business Logic Layer)"]
    end

    subgraph Domain["Domain Layer"]
        CC["core-credit"]
        CD["core-deposit"]
        CCU["core-customer"]
        CA["core-accounting"]
        GOV["governance"]
        CUS["core-custody"]
    end

    subgraph Infra["Infrastructure Layer"]
        PG["PostgreSQL"]
        CALA["cala-ledger"]
        EXT["External APIs<br/>(BitGo, Sumsub)"]
    end

    CP --> OAT
    AP --> OAT
    OAT --> CS
    OAT --> AS
    CS --> LA
    AS --> LA
    LA --> CC
    LA --> CD
    LA --> CCU
    LA --> CA
    LA --> GOV
    LA --> CUS
    CC --> PG
    CD --> PG
    CCU --> PG
    CA --> CALA
    CUS --> EXT
    CALA --> PG
```

## Resumen de Capas del Sistema

Lana sigue una arquitectura por capas que separa las responsabilidades y permite el mantenimiento:

```mermaid
graph TD
    subgraph ClientLayer["Client Layer"]
        AP["Admin Panel<br/>(Next.js)"]
        CPO["Customer Portal<br/>(Next.js)"]
        EAPI["External APIs"]
    end

    subgraph GatewayLayer["API Gateway Layer"]
        OAT["Oathkeeper<br/>(Port 4455)"]
        KC["Keycloak<br/>(Port 8081)"]
    end

    subgraph AppLayer["Application Layer"]
        ASRV["admin-server<br/>(GraphQL)"]
        CSRV["customer-server<br/>(GraphQL)"]
        LCLI["lana-cli"]
        LAPP["lana-app<br/>(Business Logic Orchestrator)"]
        ASRV --> LAPP
        CSRV --> LAPP
        LCLI --> LAPP
    end

    subgraph DomainLayer["Domain Layer"]
        CUST["Customer"]
        CRED["Credit"]
        DEP["Deposit"]
        GOV["Governance"]
        ACCT["Accounting"]
    end

    subgraph InfraLayer["Infrastructure Layer"]
        PG["PostgreSQL"]
        CALA["Cala Ledger"]
        EXT["External APIs<br/>(BitGo, Sumsub)"]
    end

    ClientLayer --> GatewayLayer
    GatewayLayer --> AppLayer
    LAPP --> DomainLayer
    DomainLayer --> InfraLayer
```

## Capa de Cliente

### Panel de Administración

Aplicación web para operaciones bancarias:
- Gestión de clientes
- Administración de créditos
- Informes financieros
- Configuración

### Portal del Cliente

Interfaz de cara al cliente:
- Vista de cuenta
- Solicitudes de crédito
- Historial de transacciones
- Documentos

## Capa de API Gateway

### Oathkeeper (Puerto 4455)

Gestiona la validación de JWT y el enrutamiento de solicitudes:
- Valida los tokens emitidos por Keycloak
- Enruta las solicitudes a los servidores apropiados
- Aplica políticas de autenticación

### Keycloak (Puerto 8081)

Gestión de identidad y acceso:
- Dos reinos: `admin` y `customer`
- OAuth 2.0 / OpenID Connect
- Autenticación de usuarios y gestión de sesiones

## Capa de Aplicación

```mermaid
graph TD
    subgraph CLI["lana-cli Process"]
        MAIN["main()<br/>lana/cli/src/lib.rs:64-105"]
        RUNCMD["run_cmd()<br/>lana/cli/src/lib.rs:154-254"]
        MAIN --> RUNCMD
    end

    RUNCMD -->|"tokio::spawn"| ASRUN
    RUNCMD -->|"tokio::spawn"| CSRUN

    subgraph AdminServer["admin-server (Port 5253)"]
        ASRUN["run()<br/>lana/admin-server/src/lib.rs:28-70"]
        ASGQL["graphql_handler()<br/>lana/admin-server/src/lib.rs:79-136"]
        AS_SCHEMA["Schema"]
        ASRUN --> ASGQL --> AS_SCHEMA
    end

    subgraph CustServer["customer-server (Port 5254)"]
        CSRUN["run()<br/>lana/customer-server/src/lib.rs:26-66"]
        CSGQL["graphql_handler()<br/>lana/customer-server/src/lib.rs:75-132"]
        CS_SCHEMA["Schema"]
        CSRUN --> CSGQL --> CS_SCHEMA
    end

    subgraph LanaApp["lana-app"]
        INIT["LanaApp::init()<br/>lana/app/Cargo.toml:1-77"]
        AGG["Aggregates domain services"]
        INIT --> AGG
    end

    AS_SCHEMA --> INIT
    CS_SCHEMA --> INIT
```

### admin-server

API GraphQL para operaciones administrativas:
- Acceso completo al sistema
- Autorización basada en RBAC
- Se conecta al reino admin de Keycloak

### customer-server

API GraphQL para operaciones de clientes:
- Alcance limitado a los datos propios del cliente
- Interfaz simplificada
- Se conecta al reino customer de Keycloak

### lana-cli

Herramienta de línea de comandos para:
- Iniciar servidores
- Ejecutar migraciones
- Tareas administrativas
- Operaciones por lotes

### lana-app

Orquestador central de la lógica de negocio:
- Inicializa todos los servicios de dominio
- Coordina operaciones entre dominios
- Gestiona el ciclo de vida de la aplicación

## Capa de Dominio

Implementa la lógica de negocio principal usando Diseño Orientado al Dominio:

| Dominio | Propósito |
|--------|---------|
| Customer | Ciclo de vida del cliente y KYC |
| Credit | Facilidades de crédito y desembolsos |
| Deposit | Cuentas de depósito y retiros |
| Governance | Flujos de aprobación multipartita |
| Accounting | Gestión de períodos financieros |

## Capa de Infraestructura

### PostgreSQL

Almacén de datos principal:
- Almacenamiento de eventos
- Estado de entidades
- Registros de auditoría

### Cala Ledger

Sistema de contabilidad por partida doble:
- Jerarquía de cuentas
- Registro de transacciones
- Cálculo de saldos

### Integraciones Externas

- **BitGo/Komainu**: Custodia de criptomonedas
- **Sumsub**: Verificación KYC
- **SMTP**: Notificaciones por correo electrónico

## Flujo de Datos

### Procesamiento de Solicitudes

```mermaid
graph LR
    REQ["Client Request"] --> OAT["Oathkeeper"] --> JWT["JWT Validation"] --> GQL["GraphQL Server"] --> DOM["Domain Services"] --> CALA["Cala Ledger"] --> RESP["Response"]
```

### Flujo de Eventos

```mermaid
graph LR
    EVT["Domain Event"] --> OUT["Outbox Table"] --> PROC["Event Processor"] --> DEP["Dependent Domains"] --> NOTIF["External Notifications"]
```

## Decisiones Arquitectónicas Clave

### Event Sourcing

Todos los cambios de estado se capturan como eventos:
- Registro de auditoría completo
- Consultas temporales
- Capacidad de reproducción de eventos

### Arquitectura Hexagonal

Separación clara de responsabilidades:
- Lógica de dominio aislada de la infraestructura
- Patrón adaptador para servicios externos
- Lógica de negocio testeable

### Patrón CQRS

Segregación de responsabilidad de comandos y consultas:
- Rutas de lectura optimizadas
- Operaciones de escritura separadas
- Consistencia eventual cuando sea apropiado
