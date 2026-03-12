---
id: index
title: Guía de Ingeniería de Plataformas
sidebar_position: 1
---

# Guía de Ingeniería de Plataforma

Bienvenido a la documentación de ingeniería de la plataforma Lana. Esta sección está dirigida a equipos técnicos que evalúan o implementan la plataforma.

## Descripción General del Sistema

Lana es un núcleo bancario moderno construido sobre:

- **Arquitectura Hexagonal** - Separación clara entre lógica de dominio, servicios de aplicación e infraestructura
- **Event Sourcing** - Registro de auditoría completo de todos los cambios de estado
- **Diseño Orientado al Dominio** - Lógica de negocio organizada en torno a conceptos del dominio bancario
- **APIs GraphQL** - Capa de API flexible y fuertemente tipada

## Documentación

### Arquitectura del Sistema

- [Arquitectura del Sistema](system-architecture) - Capas del sistema y descripción general de componentes
- [Arquitectura Funcional](functional-architecture) - Arquitectura técnica completa
- [Arquitectura de Autenticación](authentication-architecture) - Keycloak, OAuth 2.0, diseño de gateway

### Pipelines de Datos

- [Pipelines de Datos](data-pipelines) - Meltano, dbt, BigQuery

### Modelos de Datos

- [Descripción General ERD](erds/) - Documentación del esquema de base de datos
- [Esquema del Libro Mayor Cala](erds/cala) - Base de datos del libro mayor subyacente
- [Esquema del Núcleo Lana](erds/lana) - Base de datos principal de la aplicación

### Ingeniería de Lanzamientos

- [Descripción General de Ingeniería de Lanzamientos](deployment/) - Flujo de lanzamiento de extremo a extremo
- [Sistema de Compilación](deployment/build-system) - Nix, almacenamiento en caché Cachix, imágenes Docker
- [CI/CD e Ingeniería de Lanzamientos](deployment/ci-cd) - GitHub Actions, pipelines Concourse, gráficos Helm, control de entornos Cepler

:::tip
¿Buscas información sobre componentes internos del dominio, event sourcing, trabajos en segundo plano u observabilidad? Consulta la [Guía para Desarrolladores Internos](../for-internal-developers/) — esos temas se han trasladado allí.
:::

## Stack Tecnológico

| Componente | Tecnología |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Libro Mayor | Cala (contabilidad por partida doble) |
| Base de Datos | PostgreSQL |
| Eventos | Event sourcing con patrón outbox |
| Autenticación | OAuth 2.0 / OpenID Connect |
