---
id: system-architecture
title: Arquitectura del Sistema
sidebar_position: 2
---

# Arquitectura del Sistema

Este documento proporciona una visión arquitectónica de alto nivel del sistema de Lana Bank, describiendo la arquitectura de tres capas (cliente, gateway de API, backend), los componentes principales en cada capa y cómo interactúan.

![Capas del Sistema y Relaciones](/img/architecture/system-layers-1.png)

## Visión General de las Capas del Sistema

Lana Bank sigue una arquitectura en capas que separa responsabilidades entre presentación del cliente, seguridad del gateway de API, orquestación de la aplicación, lógica de dominio, servicios de infraestructura y persistencia de datos.

```
┌─────────────────────────────────────────────────────────────────┐
│                        Capa de Cliente                          │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │   admin-panel       │    │   customer-portal   │            │
│  │   (Next.js)         │    │   (Next.js)         │            │
│  │   Puerto 3001       │    │   Puerto 3002       │            │
│  └─────────────────────┘    └─────────────────────┘            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Capa de Gateway de API                       │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │   Oathkeeper        │    │   Keycloak          │            │
│  │   Puerto 4455       │    │   Puerto 8081       │            │
│  └─────────────────────┘    └─────────────────────┘            │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Capa de Aplicación (Rust)                    │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │   admin-server      │    │   customer-server   │            │
│  │   Puerto 5253       │    │   Puerto 5254       │            │
│  └─────────────────────┘    └─────────────────────┘            │
│                    ┌─────────────────────┐                     │
│                    │      lana-app       │                     │
│                    │  (Orquestador)      │                     │
│                    └─────────────────────┘                     │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Capa de Datos                              │
│  ┌─────────────────────┐    ┌─────────────────────┐            │
│  │   PostgreSQL        │    │   cala-ledger       │            │
│  │   (Base de datos)   │    │   (Libro mayor)     │            │
│  └─────────────────────┘    └─────────────────────┘            │
└─────────────────────────────────────────────────────────────────┘
```

## Capa de Cliente

La capa de cliente consiste en dos aplicaciones web separadas que atienden a diferentes perfiles de usuario:

| Aplicación | Tecnología | Puerto | Propósito |
|------------|------------|--------|-----------|
| admin-panel | Next.js 15 + Apollo Client | 3001 | Interfaz administrativa para operaciones bancarias |
| customer-portal | Next.js 15 + NextAuth | 3002 | Interfaz orientada al cliente para gestión de cuentas |

Ambas aplicaciones se comunican con el backend a través del gateway de API Oathkeeper, que maneja la validación de tokens de autenticación antes de enrutar las solicitudes al servidor GraphQL correspondiente.

## Capa de Gateway de API

La capa de gateway de API proporciona autenticación centralizada y enrutamiento.

### Oathkeeper (Puerto 4455)

Oathkeeper actúa como un proxy inverso que:
- Valida tokens JWT en las solicitudes entrantes
- Delega la autenticación a Keycloak vía endpoint JWKS
- Enruta solicitudes autenticadas a admin-server o customer-server
- Aplica políticas CORS

### Keycloak (Puerto 8081)

Keycloak funciona como proveedor de identidad:
- Emite tokens JWT tras autenticación exitosa
- Gestiona cuentas de usuario y credenciales
- Expone un endpoint JWKS para distribución de claves públicas
- Soporta tanto ámbitos internos (admin) como de clientes

## Capa de Aplicación

La capa de aplicación está implementada en Rust y consiste en el orquestador y los servidores GraphQL.

### lana-cli

El binario lana-cli sirve como punto de entrada principal y orquestador:
- Inicializa el pool de conexiones a la base de datos
- Crea la instancia LanaApp que agrega todos los servicios de dominio
- Lanza admin-server y customer-server en tareas separadas de tokio
- Maneja apagado elegante ante señales SIGTERM/SIGINT
- Gestiona recursos compartidos (pool de base de datos, estado de la aplicación)

### admin-server

Servidor de API GraphQL para operaciones administrativas:
- Expone consultas y mutaciones administrativas
- Valida tokens JWT mediante RemoteJwksDecoder
- Extrae el ID de usuario administrador de las claims del JWT
- Crea AdminAuthContext para verificaciones de autorización
- Sirve GraphQL Playground en `/admin/graphql`
- Maneja endpoints de webhook para integraciones externas

### customer-server

Servidor de API GraphQL para operaciones de clientes:
- Expone consultas para clientes
- Valida tokens JWT mediante RemoteJwksDecoder
- Extrae el ID de usuario cliente de las claims del JWT
- Crea CustomerAuthContext para verificaciones de autorización
- Sirve GraphQL Playground en `/customer/graphql`

### lana-app

El crate lana-app sirve como la fachada de la lógica de negocio central:
- Agrega todos los servicios de dominio (`core-credit`, `core-deposit`, `core-customer`, etc.)
- Inicializa infraestructura compartida (pool de base de datos, sistema de jobs, outbox)
- Proporciona una interfaz unificada para los resolvers de GraphQL
- Gestiona ciclos de vida y dependencias de servicios

## Capa de Dominio

Los servicios de dominio implementan la lógica de negocio siguiendo principios de Domain-Driven Design (DDD):

| Módulo | Crate | Propósito |
|--------|-------|-----------|
| Money | core-money | Tipos de moneda, importes monetarios y conversión |
| Customer | core-customer | Entidades de clientes, perfiles y relaciones |
| Credit | core-credit | Facilidades de crédito, desembolsos, obligaciones |
| Deposit | core-deposit | Cuentas de depósito, depósitos y retiros |
| Accounting | core-accounting | Plan de cuentas, balance de comprobación |
| Custody | core-custody | Billeteras e integración con custodios externos |
| Applicant | core-applicant | Flujos KYC/AML e integración con Sumsub |
| Access | core-access | Usuarios, roles y control de acceso |

## Capa de Infraestructura

Los servicios de infraestructura proporcionan capacidades transversales:

| Servicio | Crate | Propósito |
|----------|-------|-----------|
| Auditoría | audit | Registro inmutable de acciones para cumplimiento |
| Autorización | authz | Control de acceso basado en roles (RBAC) con Casbin |
| Outbox | outbox | Entrega confiable de eventos mediante patrón outbox |
| Jobs | job | Procesamiento de tareas en segundo plano |
| Trazabilidad | tracing-utils | Trazado distribuido con OpenTelemetry |

## Capa de Datos

### PostgreSQL

Base de datos relacional principal que almacena:
- Eventos de entidades (event sourcing)
- Estado proyectado de entidades
- Configuración del sistema
- Datos de auditoría

### cala-ledger

Libro mayor de contabilidad de partida doble:
- Cuentas y conjuntos de cuentas
- Transacciones y asientos
- Historial de saldos
- Plantillas de transacciones

### BigQuery (Analítica)

Almacén de datos para análisis:
- Datos extraídos mediante Meltano
- Modelos transformados con dbt
- Reportes e inteligencia de negocio

## Modelo de Despliegue

El sistema puede desplegarse como:
- **Desarrollo local**: Docker Compose con Tilt para orquestación
- **Producción**: Kubernetes con Helm charts

### Stack Tecnológico

| Capa | Tecnología |
|------|------------|
| Frontend | Next.js 15, React, Apollo Client |
| API | GraphQL (async-graphql) |
| Backend | Rust, Tokio (async runtime) |
| Base de datos | PostgreSQL |
| Libro mayor | Cala Ledger |
| Autenticación | Keycloak, OAuth 2.0/OIDC |
| Gateway | Oathkeeper |
| Observabilidad | OpenTelemetry |
| Build | Nix, Cargo |
| Orquestación | Kubernetes, Helm |
