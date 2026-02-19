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

- [Arquitectura del Sistema](system-architecture) - Visión general de capas y componentes
- [Arquitectura Funcional](functional-architecture) - Arquitectura técnica integral

### Infraestructura Técnica

- [Autenticación y Autorización](authentication-architecture) - Keycloak, OAuth 2.0, RBAC

### Integraciones

- [Canalización de Datos](data-pipelines) - Meltano, dbt, BigQuery

### Modelos de Datos

- [Resumen de ERDs](erds/) - Documentación de esquemas de base de datos
- [Esquema del Libro Mayor Cala](erds/cala) - Base de datos del libro mayor subyacente
- [Esquema Core de Lana](erds/lana) - Base de datos principal de la aplicación

### Despliegue y Operaciones

- [Guía de Despliegue](deployment/) - Resumen de despliegue
- [Sistema de Build](deployment/build-system) - Nix, compilación cruzada, Docker
- [Entorno de Desarrollo](deployment/development-environment) - Configuración local
- [Estrategia de Pruebas](deployment/testing-strategy) - Capas y herramientas de testing
- [Pipeline CI/CD](deployment/ci-cd) - GitHub Actions, Concourse, releases

## Stack Tecnológico

| Componente | Tecnología |
|-----------|------------|
| Backend | Rust |
| APIs | GraphQL (async-graphql) |
| Libro Mayor | Cala (contabilidad de doble entrada) |
| Base de Datos | PostgreSQL |
| Eventos | Event sourcing con patrón outbox |
| Autenticación | OAuth 2.0 / OpenID Connect |
