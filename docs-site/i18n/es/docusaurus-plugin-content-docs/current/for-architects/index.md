---
id: index
title: Guía de Arquitectura
sidebar_position: 1
---

# Guía de Arquitectura

Bienvenido a la documentación de arquitectura de Lana. Esta sección está dirigida a equipos técnicos que evalúan o despliegan la plataforma.

## Resumen del Sistema

Lana es un core bancario moderno construido sobre:

- **Arquitectura Hexagonal** - Separación limpia entre lógica de dominio, servicios de aplicación e infraestructura
- **Event Sourcing** - Rastro de auditoría completo de todos los cambios de estado
- **Diseño Dirigido por Dominio** - Lógica de negocio organizada alrededor de conceptos del dominio bancario
- **APIs GraphQL** - Capa de API flexible y fuertemente tipada

## Documentación

### Arquitectura Funcional

Arquitectura técnica integral que cubre:

- Arquitectura de aplicación y diseño de módulos
- Patrones de comunicación (event sourcing, outbox, webhooks)
- Integraciones externas (KYC/KYB, pasarelas de pago, sistemas regulatorios)
- Arquitectura de seguridad (autenticación, autorización, cifrado)
- Requisitos de infraestructura

[Ver Arquitectura Funcional](functional-architecture)

### Modelos de Datos

Diagramas de entidad-relación para las bases de datos principales:

- [Resumen de ERDs](erds/) - Documentación de esquemas de base de datos
- [Esquema del Libro Mayor Cala](erds/cala) - Base de datos del libro mayor subyacente
- [Esquema Core de Lana](erds/lana) - Base de datos principal de la aplicación

### Despliegue

Requisitos de infraestructura y despliegue.

*[Documentación de despliegue próximamente - se añadirá del manual técnico]*

### Integraciones

Patrones y requisitos de integración con sistemas externos.

*[Documentación de integraciones próximamente - se añadirá del manual técnico]*

## Stack Tecnológico

| Componente | Tecnología |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Libro Mayor | Cala (contabilidad de doble entrada) |
| Base de Datos | PostgreSQL |
| Eventos | Event sourcing con patrón outbox |
| Autenticación | OAuth 2.0 / OpenID Connect |
