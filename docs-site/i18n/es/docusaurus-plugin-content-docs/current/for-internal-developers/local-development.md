---
id: local-development
title: Configuración de Desarrollo Local
sidebar_position: 2
---

# Configuración del Entorno de Desarrollo Local

Esta guía te acompaña en la configuración de un entorno de desarrollo local para lana-bank.

## Requisitos Previos

- [Nix](https://nixos.org/download.html) con flakes habilitados
- Docker y Docker Compose

## Inicio Rápido

### 1. Ingresar al Shell de Nix

```bash
nix develop
```

Esto proporciona un shell reproducible con todas las herramientas requeridas: cadena de herramientas Rust estable, Node.js 20, pnpm 10, Python 3.13, herramientas de cliente PostgreSQL, `sqlx-cli` y Tilt.

### 2. Iniciar Dependencias

```bash
make start-deps
```

Esto inicia los siguientes servicios de Docker:

| Servicio | Puerto | Propósito |
|---------|------|---------|
| `core-pg` (PostgreSQL) | 5433 | Base de datos principal de la aplicación |
| `keycloak` | 8081 | Proveedor de identidad (OIDC) |
| `keycloak-pg` | 5437 | Base de datos de Keycloak |
| `oathkeeper` | 4455 | Puerta de enlace API (validación JWT) |
| `otel-agent` | 4317, 4318 | Recolector OpenTelemetry |

Para incluir Dagster (pipelines de datos):

```bash
DAGSTER=true make start-deps
```

### 3. Ejecutar el Backend

```bash
make setup-db run-server
```

Esto ejecuta las migraciones de base de datos e inicia el servidor de aplicación Rust.

## Tiempo Simulado

Cuando un entorno está configurado con tiempo artificial manual, Lana mantiene el reloj de la aplicación en tiempo simulado. Esto es útil para probar flujos de trabajo dependientes del tiempo, como el procesamiento de fin de día y la acumulación de intereses.

- Los servidores web de administración y de clientes se inician inmediatamente, incluso si `sim-bootstrap` todavía está cargando datos semilla en segundo plano.
- Los operadores pueden inspeccionar el reloj actual del entorno desde la página `Información del Sistema` del Panel de Administración o mediante la consulta GraphQL de administración `time`.
- Los operadores pueden avanzar el entorno al siguiente límite de fin de día configurado desde la misma página o llamando a la mutación GraphQL de administración `timeAdvanceToNextEndOfDay`.

### 4. Ejecutar Aplicaciones Frontend

En terminales separadas:

```bash

# Panel de Administración

cd apps/admin-panel && pnpm dev

# Portal del Cliente

cd apps/customer-portal && pnpm dev
```

## URLs de Desarrollo

| Servicio | URL |
|---------|-----|
| Panel de Administración | http://admin.localhost:4455 |
| Portal del Cliente | http://app.localhost:4455 |
| API GraphQL de Administración | http://admin.localhost:4455/graphql |
| API GraphQL del Cliente | http://app.localhost:4455/graphql |
| Consola de Administración de Keycloak | http://localhost:8081 |

:::info
Las APIs GraphQL deben accederse a través de Oathkeeper (puerto 4455) que maneja la validación de JWT. Los puertos directos (5253/5254) carecen de contexto de autenticación y no funcionarán correctamente.
:::

:::tip
Si `app.localhost` no se resuelve, añade `127.0.0.1 app.localhost` y `::1 app.localhost` a tu archivo `/etc/hosts`.
:::

## Desarrollo Interactivo con Tilt

Para recarga en caliente de todos los servicios:

```bash
make dev-up
```

Tilt orquesta servicios Docker + procesos de aplicación locales con recarga en vivo. Detén con:

```bash
make dev-down
```

## Comandos Comunes

| Comando | Propósito |
|---------|---------|
| `make start-deps` | Iniciar dependencias de Docker |
| `make stop-deps` | Detener dependencias de Docker |
| `make reset-deps` | Limpiar y reiniciar bases de datos |
| `make check-code-rust` | Verificar que el código Rust compile |
| `make check-code-apps` | Lint, verificación de tipos y compilación de frontends |
| `cargo nextest run` | Ejecutar todas las pruebas de Rust |
| `cargo nextest run -p <crate>` | Ejecutar pruebas para un único crate |
| `make e2e` | Ejecutar pruebas end-to-end de BATS |
| `make sdl` | Regenerar esquemas GraphQL |
| `make sqlx-prepare` | Actualizar caché de consultas offline de SQLx |

:::warning
Anteponer `SQLX_OFFLINE=true` a los comandos directos de `cargo` para usar la caché de consultas offline en lugar de requerir una base de datos en ejecución.
:::

## Acceso a la Base de Datos

Conectarse a la base de datos principal de PostgreSQL:

```bash
psql postgres://user:password@localhost:5433/pg
```

Ejecutar migraciones manualmente:

```bash
cargo sqlx migrate run
```

Las migraciones están ubicadas en `lana/app/migrations/`.

## Variables de Entorno

El shell de Nix establece automáticamente las variables de entorno clave:

| Variable | Valor | Propósito |
|----------|-------|---------|
| `PG_CON` | `postgres://user:password@localhost:5433/pg` | Conexión a la base de datos |
| `ENCRYPTION_KEY` | (clave de desarrollo) | Clave de cifrado para secretos |
| `KC_URL` | `http://localhost:8081` | URL de Keycloak |
| `REALM` | (configurado por realm) | Realm de Keycloak |
