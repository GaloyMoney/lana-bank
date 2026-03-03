---
id: development-environment
title: Entorno de Desarrollo
sidebar_position: 3
---

# Entorno de Desarrollo

Este documento describe como configurar y utilizar el entorno de desarrollo local.

## Requisitos Previos

- Nix con flakes habilitado
- Docker y Docker Compose
- Git

## Configuracion

### 1. Entrar al Shell de Nix

```bash
cd lana-bank
nix develop
```

### 2. Iniciar Dependencias

```bash
make start-deps
```

Esto inicia:
- PostgreSQL (puerto 5433)
- Keycloak (puerto 8081)
- Oathkeeper (puerto 4455)

### 3. Ejecutar Migraciones

```bash
cargo sqlx migrate run
```

### 4. Iniciar la Aplicacion

```bash
# Ejecutar todos los servidores
cargo run

# O usar Tilt para desarrollo interactivo
make dev-up
```

## URLs de Servicios

| Servicio | URL |
|----------|-----|
| Admin Panel | http://admin.localhost:4455 |
| Customer Portal | http://app.localhost:4455 |
| Admin GraphQL | http://admin.localhost:4455/graphql |
| Customer GraphQL | http://app.localhost:4455/graphql |
| Keycloak | http://localhost:8081 |

## Desarrollo con Tilt

Desarrollo interactivo con recompilacion automatica:

```bash
# Iniciar Tilt
make dev-up

# Abrir la interfaz de Tilt
# http://localhost:10350

# Detener Tilt
make dev-down
```

## Acceso a la Base de Datos

```bash
# Conectar a PostgreSQL
psql -h localhost -p 5433 -U lana -d lana

# Reiniciar la base de datos
make reset-deps
```

## Desarrollo Frontend

```bash
# Admin Panel
cd apps/admin-panel
pnpm install
pnpm dev

# Customer Portal
cd apps/customer-portal
pnpm install
pnpm dev
```

## Variables de Entorno

```bash
# .env.local
DATABASE_URL=postgres://lana:lana@localhost:5433/lana
KEYCLOAK_URL=http://localhost:8081
OATHKEEPER_URL=http://localhost:4455
```

## Credenciales de Keycloak

| Realm | Usuario | Contrasena |
|-------|---------|------------|
| admin | admin | admin |
| customer | test@test.com | test |

## Problemas Comunes

### Conflictos de Puerto

```bash
# Verificar que esta usando un puerto
lsof -i :5433

# Terminar el proceso
kill -9 <PID>
```

### Reiniciar la Base de Datos

```bash
make reset-deps
cargo sqlx migrate run
```

### Problemas de Cache

```bash
# Limpiar cache de Rust
cargo clean

# Limpiar cache de pnpm
pnpm store prune
```
