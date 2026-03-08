---
id: index
title: Guía de Ingeniería de Plataforma
sidebar_position: 1
---

# Guía de Ingeniería de Plataforma

Bienvenido a la documentación de ingeniería de plataforma de Lana. Esta sección está dirigida a equipos técnicos que evalúan o despliegan la plataforma.

## Resumen del Sistema

Lana es un core bancario moderno construido sobre:

- **Arquitectura Hexagonal** - Separación limpia entre lógica de dominio, servicios de aplicación e infraestructura
- **Event Sourcing** - Rastro de auditoría completo de todos los cambios de estado
- **Diseño Dirigido por Dominio** - Lógica de negocio organizada alrededor de conceptos del dominio bancario
- **APIs GraphQL** - Capa de API flexible y fuertemente tipada

## Documentación

### Arquitectura del Sistema

- [Arquitectura del Sistema](system-architecture) - Capas del sistema y visión general de componentes
- [Arquitectura Funcional](functional-architecture) - Arquitectura técnica integral
- [Arquitectura de Autenticación](authentication-architecture) - Keycloak, OAuth 2.0, diseño de gateway

### Canalizaciones de Datos

- [Canalizaciones de Datos](data-pipelines) - Meltano, dbt, BigQuery

### Modelos de Datos

- [Resumen de ERDs](erds/) - Documentación de esquemas de base de datos
- [Esquema del Libro Mayor Cala](erds/cala) - Base de datos del libro mayor subyacente
- [Esquema Core de Lana](erds/lana) - Base de datos principal de la aplicación

### Ingeniería de Releases

- [Resumen de Ingeniería de Releases](deployment/) - Flujo de release de extremo a extremo
- [Sistema de Build](deployment/build-system) - Nix, caché Cachix, imágenes Docker
- [CI/CD e Ingeniería de Releases](deployment/ci-cd) - GitHub Actions, pipelines Concourse, Helm charts, control de entornos con Cepler

:::tip
¿Buscas información sobre los internos del dominio, event sourcing, trabajos en segundo plano u observabilidad? Consulta la [Guía del Desarrollador Interno](../for-internal-developers/) — esos temas se han trasladado allí.
:::

## Stack Tecnológico

| Componente | Tecnología |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Libro Mayor | Cala (contabilidad de doble entrada) |
| Base de Datos | PostgreSQL |
| Eventos | Event sourcing con patrón outbox |
| Autenticación | OAuth 2.0 / OpenID Connect |
