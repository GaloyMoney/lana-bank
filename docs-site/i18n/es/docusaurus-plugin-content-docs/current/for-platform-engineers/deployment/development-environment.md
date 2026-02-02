---
id: development-environment
title: Entorno de Desarrollo
sidebar_position: 3
---

# Entorno de Desarrollo Local

Este documento describe la configuración del entorno de desarrollo local para Lana Bank, incluyendo el uso de Nix, Tilt y Docker Compose.

## Descripción General

El entorno de desarrollo local utiliza:
- **Nix**: Gestión reproducible de dependencias
- **Tilt**: Orquestación de servicios con hot-reload
- **Docker Compose**: Servicios de infraestructura

```
┌─────────────────────────────────────────────────────────────────┐
│                    Tilt (Orquestación)                          │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐ │
│  │   lana-cli      │  │  admin-panel    │  │ customer-portal │ │
│  │   (Rust)        │  │  (Next.js)      │  │ (Next.js)       │ │
│  │   Hot-reload    │  │  Hot-reload     │  │ Hot-reload      │ │
│  └─────────────────┘  └─────────────────┘  └─────────────────┘ │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│               Docker Compose (Infraestructura)                  │
│  ┌───────────┐  ┌───────────┐  ┌───────────┐  ┌───────────┐   │
│  │PostgreSQL │  │ Keycloak  │  │Oathkeeper │  │   OTEL    │   │
│  │           │  │           │  │           │  │ Collector │   │
│  └───────────┘  └───────────┘  └───────────┘  └───────────┘   │
└─────────────────────────────────────────────────────────────────┘
```

## Requisitos Previos

### Instalación de Nix

```bash
# Instalar Nix con flakes habilitados
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh
```

### Verificar Instalación

```bash
nix --version
# nix (Nix) 2.x.x
```

## Inicio Rápido

### Iniciar el Entorno de Desarrollo

```bash
# 1. Clonar el repositorio
git clone https://github.com/GaloyMoney/lana-bank.git
cd lana-bank

# 2. Entrar al shell de desarrollo
nix develop

# 3. Iniciar dependencias (PostgreSQL, Keycloak, etc.)
make start-deps

# 4. Iniciar todos los servicios con Tilt
make dev-up
```

### Acceso a los Servicios

| Servicio | URL | Credenciales |
|----------|-----|--------------|
| Admin Panel | http://admin.localhost:4455 | admin / admin |
| Customer Portal | http://app.localhost:4455 | customer@test.com / test123 |
| Tilt UI | http://localhost:10350 | - |
| Keycloak Admin | http://localhost:8081/admin | admin / admin |
| GraphQL Admin | http://admin.localhost:4455/graphql | Bearer token |
| GraphQL Customer | http://app.localhost:4455/graphql | Bearer token |

### Detener el Entorno

```bash
# Detener Tilt
make dev-down

# Detener dependencias
make stop-deps

# Limpiar y reiniciar dependencias
make reset-deps
```

## Selección del Motor de Contenedores

### Lógica de Detección Automática

El sistema detecta automáticamente Docker o Podman:

```bash
# El Makefile detecta automáticamente
# Prioridad: Docker > Podman

# Verificar motor detectado
make check-container-engine
```

### Anulación Manual

```bash
# Forzar uso de Podman
export CONTAINER_ENGINE=podman
make start-deps

# Forzar uso de Docker
export CONTAINER_ENGINE=docker
make start-deps
```

## Orquestación con Tilt

### Estructura del Tiltfile

```python
# dev/Tiltfile

# Servicios Rust
local_resource(
    'lana-cli',
    serve_cmd='cargo run --package lana-cli -- serve',
    deps=['./core', './lana', './lib'],
    resource_deps=['deps'],
)

# Servicios Frontend
local_resource(
    'admin-panel',
    serve_cmd='pnpm --filter admin-panel dev',
    deps=['./apps/admin-panel'],
    resource_deps=['lana-cli'],
)

local_resource(
    'customer-portal',
    serve_cmd='pnpm --filter customer-portal dev',
    deps=['./apps/customer-portal'],
    resource_deps=['lana-cli'],
)

# Dependencias Docker Compose
docker_compose('./docker-compose.yml')
```

### Recursos Locales

| Recurso | Comando | Dependencias |
|---------|---------|--------------|
| lana-cli | `cargo run` | deps (docker-compose) |
| admin-panel | `pnpm dev` | lana-cli |
| customer-portal | `pnpm dev` | lana-cli |

### Comportamiento de Recarga en Caliente

- **Rust**: Recompilación automática al cambiar archivos `.rs`
- **Frontend**: Hot Module Replacement (HMR) de Next.js
- **Dependencias**: Reinicio manual si cambia docker-compose.yml

## Servicios de Docker Compose

### Definiciones de Servicios

```yaml
# docker-compose.yml
services:
  postgres:
    image: postgres:14
    ports:
      - "5433:5432"
    environment:
      POSTGRES_USER: lana
      POSTGRES_PASSWORD: lana
      POSTGRES_DB: lana
    volumes:
      - postgres_data:/var/lib/postgresql/data

  keycloak:
    image: quay.io/keycloak/keycloak:23.0
    ports:
      - "8081:8080"
    environment:
      KEYCLOAK_ADMIN: admin
      KEYCLOAK_ADMIN_PASSWORD: admin
      KC_DB: postgres
      KC_DB_URL: jdbc:postgresql://postgres:5432/keycloak
    depends_on:
      - postgres

  oathkeeper:
    image: oryd/oathkeeper:v0.40
    ports:
      - "4455:4455"
    volumes:
      - ./dev/oathkeeper:/etc/config
    command: serve --config /etc/config/oathkeeper.yml

  otel-collector:
    image: otel/opentelemetry-collector:0.91.0
    ports:
      - "4317:4317"   # OTLP gRPC
      - "4318:4318"   # OTLP HTTP
    volumes:
      - ./dev/otel-agent-config.yaml:/etc/otel/config.yaml
```

### Configuración de Keycloak

Los realms se importan automáticamente al iniciar:

```bash
# Archivos de configuración
dev/keycloak/
├── internal-realm.json   # Realm para admin
└── customer-realm.json   # Realm para clientes
```

### Credenciales de Base de Datos

| Base de datos | Host | Puerto | Usuario | Contraseña |
|---------------|------|--------|---------|------------|
| Lana | localhost | 5433 | lana | lana |
| Keycloak | localhost | 5437 | keycloak | keycloak |

## Entorno del Shell de Desarrollo

### Variables de Entorno

```bash
# Cargadas automáticamente en nix develop
DATABASE_URL=postgres://lana:lana@localhost:5433/lana
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
KEYCLOAK_URL=http://localhost:8081
```

### Herramientas Disponibles

El shell de desarrollo incluye:

| Categoría | Herramientas |
|-----------|--------------|
| Rust | cargo, rustc, rustfmt, clippy |
| Node | node (v20), pnpm |
| Base de datos | sqlx-cli, psql |
| Contenedores | docker-compose |
| Utilidades | jq, yq, gh |

## Objetivos Make para Desarrollo

### Objetivos Esenciales

```bash
make start-deps     # Iniciar PostgreSQL, Keycloak, etc.
make stop-deps      # Detener dependencias
make reset-deps     # Limpiar y reiniciar

make dev-up         # Iniciar Tilt
make dev-down       # Detener Tilt
```

### Ejecutar el Servidor Directamente

```bash
# Sin Tilt (útil para depuración)
cargo run --package lana-cli -- serve

# Con logs detallados
RUST_LOG=debug cargo run --package lana-cli -- serve
```

### Objetivos para Desarrollo Frontend

```bash
make admin-dev      # Solo admin-panel
make customer-dev   # Solo customer-portal
make codegen        # Regenerar tipos GraphQL
```

### Objetivos de Pruebas

```bash
make test           # Tests unitarios Rust
make e2e            # Tests BATS
make cypress        # Tests Cypress
```

### Objetivos de Calidad de Código

```bash
make fmt            # Formatear código
make lint           # Ejecutar linters
make check          # Verificar compilación
```

### Objetivos de Base de Datos

```bash
make migrate        # Ejecutar migraciones
make sqlx-prepare   # Actualizar caché SQLx offline
```

## Flujo de Trabajo de Desarrollo

### Sesión Típica de Desarrollo

```bash
# 1. Entrar al shell
nix develop

# 2. Iniciar stack
make start-deps
make dev-up

# 3. Hacer cambios y ver hot-reload
# - Cambios en Rust: Tilt recompila
# - Cambios en Frontend: HMR actualiza

# 4. Ejecutar tests
make test

# 5. Verificar antes de commit
make fmt
make check
make sqlx-prepare  # Si cambió SQL
make sdl           # Si cambió GraphQL

# 6. Commit
git add .
git commit -m "feat: ..."
```

### Trabajando con Migraciones de Base de Datos

```bash
# Crear nueva migración
sqlx migrate add descripcion_de_migracion

# Ejecutar migraciones
sqlx migrate run --database-url $DATABASE_URL

# Actualizar caché offline
make sqlx-prepare
```

### Regenerar Esquemas GraphQL

```bash
# Después de cambiar resolvers
make sdl

# Regenerar tipos TypeScript
cd apps/admin-panel && pnpm codegen
cd apps/customer-portal && pnpm codegen
```

## Resolución de Problemas

### Problemas Comunes

**Puerto en uso:**
```bash
# Verificar qué usa el puerto
lsof -i :5433
# Matar proceso o cambiar puerto
```

**Migraciones fallidas:**
```bash
make reset-deps  # Reinicia desde cero
```

**Caché SQLx desactualizado:**
```bash
make sqlx-prepare
```

### Visualización de Logs

```bash
# Logs de Tilt
# Ver en http://localhost:10350

# Logs de Docker Compose
docker-compose logs -f postgres
docker-compose logs -f keycloak

# Logs de cargo
RUST_LOG=debug cargo run --package lana-cli -- serve
```

## Configuración Avanzada

### Habilitar el Pipeline de Datos

```bash
# Iniciar con Dagster
DAGSTER=true make start-deps

# Acceder a Dagster UI
# http://localhost:3000
```

### Variables de Entorno Personalizadas

```bash
# Crear archivo .env.local
cat > .env.local << EOF
DATABASE_URL=postgres://custom:pass@localhost:5432/lana
RUST_LOG=debug
EOF

# Cargar antes de iniciar
source .env.local
```

### Ejecutar sin Tilt

```bash
# Terminal 1: Backend
cargo run --package lana-cli -- serve

# Terminal 2: Admin Panel
cd apps/admin-panel && pnpm dev

# Terminal 3: Customer Portal
cd apps/customer-portal && pnpm dev
```
