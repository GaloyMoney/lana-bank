---
id: index
title: Despliegue
sidebar_position: 1
---

# Guía de Despliegue

Esta sección cubre los aspectos de construcción, despliegue y operación de Lana Bank.

## Documentación Disponible

### [Sistema de Build](build-system)
Arquitectura del sistema de construcción, compilación cruzada, y generación de artefactos.

### [Pipeline CI/CD](ci-cd)
Flujos de trabajo de GitHub Actions, Concourse y estrategia de releases.

## Stack de Herramientas

| Herramienta | Propósito |
|-------------|-----------|
| Nix | Builds reproducibles y gestión de dependencias |
| Cargo | Compilación de proyectos Rust |
| Docker | Contenedorización de servicios |
| Tilt | Orquestación de desarrollo local |
| GitHub Actions | CI/CD para PRs y releases |
| Concourse | Pipeline de caché de Nix |
| Cachix | Caché binaria de Nix |
| Helm | Despliegue en Kubernetes |

## Arquitectura de Despliegue

```
┌─────────────────────────────────────────────────────────────────┐
│                      Desarrollo Local                           │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                       Tilt                               │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │   │
│  │  │ lana-cli│  │ admin   │  │customer │  │ Keycloak│    │   │
│  │  │         │  │ panel   │  │ portal  │  │         │    │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │   │
│  └─────────────────────────────────────────────────────────┘   │
│                             │                                   │
│                     Docker Compose                              │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  PostgreSQL │ Oathkeeper │ OTEL Collector │ Keycloak DB │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                        Producción                               │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Kubernetes                            │   │
│  │  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐    │   │
│  │  │ lana-cli│  │ admin   │  │customer │  │ Ingress │    │   │
│  │  │ (Pod)   │  │ (Pod)   │  │ (Pod)   │  │         │    │   │
│  │  └─────────┘  └─────────┘  └─────────┘  └─────────┘    │   │
│  │                                                         │   │
│  │  ┌─────────────────────────────────────────────────┐   │   │
│  │  │  PostgreSQL (Cloud SQL) │ Keycloak │ Monitoring │   │   │
│  │  └─────────────────────────────────────────────────┘   │   │
│  └─────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Flujo de Trabajo Típico

### Desarrollo

```bash
# 1. Entrar al shell de desarrollo
nix develop

# 2. Iniciar dependencias
make start-deps

# 3. Ejecutar servicios con Tilt
make dev-up

# 4. Acceder a los servicios
# - Admin Panel: http://admin.localhost:4455
# - Customer Portal: http://app.localhost:4455
# - Tilt UI: http://localhost:10350
```

### CI/CD

1. **Pull Request**: Se ejecutan pruebas y verificaciones
2. **Merge a main**: Se construyen y publican artefactos
3. **Tag de release**: Se despliega a producción

### Producción

```bash
# Desplegar con Helm
helm upgrade --install lana ./helm/lana \
  --namespace lana \
  --values values.prod.yaml
```

## Requisitos de Infraestructura

### Mínimos

| Recurso | Especificación |
|---------|----------------|
| CPU | 4 cores |
| RAM | 8 GB |
| Disco | 50 GB SSD |
| PostgreSQL | 14+ |

### Recomendados para Producción

| Recurso | Especificación |
|---------|----------------|
| CPU | 8+ cores |
| RAM | 16+ GB |
| Disco | 100+ GB SSD |
| PostgreSQL | Instancia administrada (Cloud SQL, RDS) |
| Kubernetes | GKE, EKS, o AKS |
