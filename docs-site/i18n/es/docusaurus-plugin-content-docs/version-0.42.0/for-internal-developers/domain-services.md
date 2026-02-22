---
id: domain-services
title: Servicios de Dominio
sidebar_position: 3
---

# Servicios de Dominio

Este documento proporciona una visión general de la capa de servicios de dominio en el sistema Lana Bank. Los servicios de dominio se implementan como crates `core-*` que encapsulan dominios de negocio específicos siguiendo los principios de Domain-Driven Design (DDD).

## Enfoque de Domain-Driven Design

El sistema Lana Bank sigue un enfoque de diseño guiado por el dominio en el que la lógica de negocio se organiza en módulos discretos y cohesivos llamados servicios de dominio. Cada servicio de dominio:

- **Encapsula un contexto delimitado**: Contiene toda la lógica, modelos y datos relacionados con un dominio de negocio específico
- **Mantiene la pureza del dominio**: La lógica de negocio está separada de las preocupaciones de infraestructura
- **Usa event sourcing**: Las entidades de dominio publican eventos que impulsan cambios de estado y comunicación entre servicios
- **Hace cumplir invariantes**: Las reglas de negocio se hacen cumplir dentro de los límites del dominio
- **Expone una API limpia**: Otros servicios interactúan a través de interfaces bien definidas

Los servicios de dominio se ubican en la estructura de directorios `core/*`, siendo cada servicio un crate independiente de Rust.

## Arquitectura de Servicios de Dominio

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        Capa de Aplicación                               │
│                           (lana-app)                                    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Servicios de Dominio (core-*)                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │ core-credit │  │core-deposit │  │core-customer│  │core-custody │    │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐    │
│  │core-account │  │ governance  │  │ core-access │  │core-applicant│   │
│  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────┘    │
└─────────────────────────────────────────────────────────────────────────┘
                                   │
                                   ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                    Servicios de Infraestructura                         │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────────┐   │
│  │  audit  │  │  authz  │  │  outbox │  │   job   │  │tracing-utils│   │
│  └─────────┘  └─────────┘  └─────────┘  └─────────┘  └─────────────┘   │
└─────────────────────────────────────────────────────────────────────────┘
```

## Módulos Principales del Dominio

### Servicios de Dominio Primarios

| Módulo | Crate | Propósito | Dependencias Clave |
|--------|-------|-----------|-------------------|
| Money | core-money | Define tipos de moneda, importes monetarios y utilidades de conversión | rust_decimal, async-graphql |
| Customer | core-customer | Gestiona entidades de clientes, perfiles y relaciones | es-entity, governance, document-storage |
| Credit | core-credit | Maneja facilidades de crédito, desembolsos, obligaciones, devengo de intereses y pagos | cala-ledger, core-accounting, core-custody, governance |
| Deposit | core-deposit | Gestiona cuentas de depósito, depósitos y retiros | cala-ledger, core-accounting, core-customer, governance |
| Accounting | core-accounting | Proporciona plan de cuentas, balance de comprobación y transacciones manuales | cala-ledger, core-money, cloud-storage |
| Custody | core-custody | Gestiona billeteras e integra con custodios externos (BitGo, Komainu) | bitgo, komainu, core-money |
| Applicant | core-applicant | Maneja flujos KYC/AML e integración con Sumsub | sumsub, core-customer, es-entity |
| Access | core-access | Gestiona usuarios, roles y control de acceso | es-entity, governance, authz |

### Servicios de Dominio de Soporte

| Módulo | Crate | Propósito |
|--------|-------|-----------|
| Governance | governance | Implementa procesos de aprobación, votación de comités y aplicación de políticas |
| Document Storage | document-storage | Maneja carga, almacenamiento y recuperación de documentos |
| Public ID | public-id | Genera y gestiona identificadores públicos legibles por humanos |
| Price | core-price | Proporciona fuentes de precios de criptomonedas y tasas de conversión |
| Report | core-report | Gestiona la generación de reportes e integración con Airflow |

## Patrones Arquitectónicos Comunes

Todos los servicios de dominio siguen patrones arquitectónicos consistentes para garantizar mantenibilidad y consistencia.

### Patrón de Event Sourcing

Las entidades de dominio usan el framework `es-entity`, que proporciona:

- **Definiciones de entidades**: Rasgos base para agregados y entidades
- **Event sourcing**: Persistencia y reproducción automática de eventos
- **Contexto de eventos**: Seguimiento de metadatos para auditoría y trazabilidad
- **Integración GraphQL**: Generación automática de esquema cuando la característica `graphql` está habilitada

```
┌──────────────┐    ┌──────────────────┐    ┌──────────────────┐
│   Command    │───▶│  Domain Service  │───▶│  Domain Events   │
└──────────────┘    └──────────────────┘    └──────────────────┘
                           │                        │
                           ▼                        ▼
                    ┌──────────────┐         ┌──────────────┐
                    │  Repository  │         │   Outbox     │
                    └──────────────┘         │  Publisher   │
                           │                 └──────────────┘
                           ▼                        │
                    ┌──────────────┐                ▼
                    │  PostgreSQL  │         ┌──────────────┐
                    │ Event Store  │         │outbox.events │
                    └──────────────┘         └──────────────┘
```

### Patrón de Autorización y Auditoría

Cada operación de dominio sigue un patrón consistente para autorización y auditoría:

1. **Verificación de autorización**: La librería `authz` valida que el sujeto tenga los permisos requeridos
2. **Ejecución de la lógica de negocio**: La lógica de dominio se ejecuta y emite eventos
3. **Registro de auditoría**: La librería `audit` registra la acción con sujeto, recurso y resultado
4. **Publicación de eventos**: Los eventos se publican en outbox para procesamiento asíncrono

```rust
// Ejemplo de patrón en un servicio de dominio
pub async fn execute_action(
    &self,
    subject: &Subject,
    input: ActionInput,
) -> Result<ActionOutput, Error> {
    // 1. Verificar autorización
    self.authz.enforce(subject, Object::Action, Permission::Execute).await?;

    // 2. Ejecutar lógica de negocio
    let result = self.perform_action(input).await?;

    // 3. Registrar auditoría
    self.audit.record(subject, Action::Execute, &result).await;

    // 4. Publicar eventos (automático via es-entity)
    Ok(result)
}
```

### Patrón de Integración con el Libro Mayor

Los servicios de dominio que manejan transacciones financieras se integran con el libro mayor Cala:

```
┌─────────────────────┐
│  Servicio Dominio   │
│  (core-credit)      │
└─────────────────────┘
          │
          ▼
┌─────────────────────┐
│  core-accounting    │
│  (Adaptador)        │
└─────────────────────┘
          │
          ▼
┌─────────────────────┐
│   cala-ledger       │
│  (Libro Mayor)      │
└─────────────────────┘
```

## Comunicación Impulsada por Eventos

Los servicios de dominio se comunican mediante eventos:

### Tipos de Eventos

| Tipo | Propósito | Ejemplo |
|------|-----------|---------|
| Eventos de Entidad | Cambios de estado internos | `CreditFacilityCreated` |
| Eventos de Dominio | Comunicación entre servicios | `CollateralUpdated` |
| Eventos Públicos | Integración externa | `PaymentReceived` |

### Flujo de Eventos

```
┌────────────┐    ┌────────────┐    ┌────────────┐    ┌────────────┐
│  Comando   │───▶│  Servicio  │───▶│   Evento   │───▶│   Outbox   │
└────────────┘    │  Dominio   │    │            │    │  Publisher │
                  └────────────┘    └────────────┘    └────────────┘
                                                            │
                  ┌────────────┐    ┌────────────┐          │
                  │   Job      │◀───│   Evento   │◀─────────┘
                  │  Handler   │    │  Consumer  │
                  └────────────┘    └────────────┘
```

## Estructura de Dependencias

### Reglas de Dependencia

1. Los servicios de dominio no dependen unos de otros directamente
2. La coordinación se realiza a través de eventos o la capa de aplicación
3. Los servicios de infraestructura son compartidos
4. La comunicación con el libro mayor se realiza a través de `core-accounting`

### Feature Flags

Los servicios de dominio usan feature flags para controlar dependencias opcionales:

```toml
[features]
default = []
graphql = ["async-graphql"]
import = ["cloud-storage"]
```

## Integración con la Capa de Aplicación

El crate `lana-app` agrega todos los servicios de dominio:

```rust
pub struct LanaApp {
    pub customers: Customers,
    pub credit_facilities: CreditFacilities,
    pub deposits: Deposits,
    pub accounting: Accounting,
    pub governance: Governance,
    // ... otros servicios
}
```

Esta agregación permite:
- Inicialización coordinada de servicios
- Inyección de dependencias compartidas
- Acceso unificado desde los resolvers GraphQL
