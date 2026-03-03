---
id: build-system
title: Sistema de Build
sidebar_position: 2
---

# Sistema de Build

Lana utiliza [Nix Flakes](https://nix.dev/concepts/flakes) para builds reproducibles y [Cachix](https://www.cachix.org/) para caché binaria. Si no estás familiarizado con Nix, la versión corta es: es un sistema de build que garantiza que las mismas entradas siempre producen la misma salida, sin importar en qué máquina estés compilando. Esto elimina el problema de "funciona en mi máquina" tanto para desarrollo como para CI.

Esta página cubre cómo funcionan los builds localmente, cómo CI utiliza la caché de Nix, y cómo se producen las imágenes Docker para los releases.

## Estructura del Nix Flake

Todo comienza con el archivo `flake.nix` en la raíz del repositorio. Define todos los objetivos de build, entornos de desarrollo y puntos de entrada de CI:

```
flake.nix
├── packages
│   ├── lana-cli              # El binario principal del servidor/CLI (build release, optimizado)
│   ├── lana-cli-debug        # Un build de depuración (más rápido de compilar, útil en desarrollo)
│   └── lana-deps             # Solo el árbol de dependencias de Rust, precompilado (usado para caché)
├── devShells
│   └── default               # El entorno de desarrollo con todas las herramientas
├── checks
│   └── flake-check           # Valida el propio flake
└── apps
    ├── nextest               # Ejecuta la suite de tests de Rust vía cargo nextest
    ├── bats                  # Ejecuta los tests end-to-end de BATS
    └── simulation            # Ejecuta simulaciones de escenarios de facilidades
```

La sección `packages` es lo que CI compila. La sección `apps` proporciona puntos de entrada convenientes para ejecutar tests — CI ejecuta `nix run .#nextest` en lugar de manejar `cargo nextest` manualmente, porque la app de Nix garantiza que todas las variables de entorno y dependencias correctas estén configuradas.

Vale la pena destacar el paquete `lana-deps`: precompila solo el árbol de dependencias de Rust (todos los crates en `Cargo.lock`) sin incluir ningún código propio de lana. Esta es una optimización de caché — dado que las dependencias cambian con mucha menos frecuencia que el código de la aplicación, compilarlas por separado significa que pueden almacenarse en caché y reutilizarse en muchos builds.

## Shell de Desarrollo

Cuando ejecutas `nix develop`, Nix configura un shell con todas las herramientas que necesitas para desarrollo:

```bash
nix develop
```

Esto te proporciona: el toolchain estable de Rust, Node 20, pnpm 10, Python 3.13, un cliente PostgreSQL, sqlx-cli, y todas las demás utilidades que usa el proyecto. No necesitas instalar ninguna de estas globalmente en tu máquina — Nix las gestiona por ti y no interfieren con otros proyectos.

Si has configurado Cachix (ver más abajo), este comando es casi instantáneo porque el entorno de shell precompilado se descarga de la caché en lugar de compilarse en tu máquina.

## Compilación del Binario de Release

Hay dos formas de compilar el binario `lana-cli`:

```bash
# Build release — optimizado, usado en CI para imágenes Docker
nix build --impure .#lana-cli-release

# Build de depuración — más rápido de compilar, tiene símbolos de depuración
nix build .#lana-cli-debug
```

El build release usa la bandera `--impure` porque lee variables de entorno (`VERSION`, `COMMITHASH`, `BUILDTIME`) que se incorporan en el binario. Estas son establecidas por el pipeline de CI para que la aplicación en ejecución sepa qué versión es y cuándo fue compilada. En desarrollo local normalmente usarías el build de depuración, que omite esto y compila mucho más rápido.

## Imágenes Docker

Las imágenes Docker se construyen durante el pipeline de release de Concourse (ver [CI/CD e Ingeniería de Releases](ci-cd) para el panorama completo). Lo clave a entender es que hay dos Dockerfiles diferentes dependiendo de si estamos construyendo un release candidate o un release final:

- **`Dockerfile.rc`** se usa para release candidates. El paso de build de Nix compila el binario, y este Dockerfile simplemente lo copia en una imagen base mínima. Esto es rápido porque el binario ya está compilado.

- **`Dockerfile.release`** se usa para el release final. En lugar de copiar un binario local, descarga el binario publicado desde el GitHub Release. Esto hace que la construcción de la imagen sea completamente reproducible — cualquiera puede reconstruir exactamente la misma imagen a partir de los artefactos del GitHub Release.

Ambos Dockerfiles usan una **imagen base distroless**, que contiene solo lo mínimo necesario para ejecutar un binario (sin shell, sin gestor de paquetes, sin utilidades). Esto minimiza la superficie de ataque y mantiene la imagen pequeña.

### Las cuatro imágenes

Cada release produce cuatro imágenes Docker, publicadas en Google Artifact Registry:

| Imagen | Qué contiene | Registro |
|--------|-------------|----------|
| `lana-bank` | El binario principal del servidor lana-cli | `gcr.io/galoyorg/lana-bank` |
| `lana-bank-admin-panel` | La aplicación Next.js del panel de administración | `gcr.io/galoyorg/lana-bank-admin-panel` |
| `lana-bank-customer-portal` | La aplicación Next.js del portal de clientes | `gcr.io/galoyorg/lana-bank-customer-portal` |
| `dagster-code-location-lana-dw` | El código del pipeline de datos de Dagster | `us.gcr.io/galoyorg/dagster-code-location-lana-dw` |

### Metadatos de build

Cada build de imagen inyecta tres datos mediante un archivo `.env` para que la aplicación en ejecución pueda reportar qué versión es:

- `VERSION` — la versión semántica (por ejemplo, `0.42.0`)
- `COMMITHASH` — el SHA corto de git desde el cual fue compilado
- `BUILDTIME` — una marca de tiempo UTC de cuándo ocurrió el build

---

## Caché Binaria de Cachix

Este es el problema que Cachix resuelve: los builds de Nix son perfectamente reproducibles, pero son lentos cuando compilas desde cero. Compilar el toolchain de Rust, todas las dependencias y el binario de la aplicación puede tomar mucho tiempo. Si cada ejecución de CI y cada desarrollador tuviera que hacer esto desde cero, sería muy tedioso.

[Cachix](https://www.cachix.org/) es una caché binaria para Nix. Cuando alguien compila una derivación de Nix y la sube a Cachix, todos los demás que necesiten la misma derivación pueden descargar el resultado precompilado en lugar de compilarlo ellos mismos. Dado que las derivaciones de Nix están direccionadas por contenido (la salida está determinada enteramente por las entradas), esto es seguro — siempre obtendrás exactamente el mismo resultado ya sea que compiles localmente o descargues de la caché.

### Detalles de la caché

| | |
|---|---|
| **Nombre de la caché** | `galoymoney` |
| **URL** | `https://galoymoney.cachix.org` |
| **Quién escribe en ella** | El pipeline nix-cache de Concourse (con un token de escritura) |
| **Quién lee de ella** | Los workflows de GitHub Actions, los jobs de Concourse, y cualquier desarrollador que ejecute `cachix use galoymoney` |

### El diseño de caché con dos sistemas

Hay una separación intencional entre quién compila para la caché y quién lee de ella:

- **Concourse** es el constructor. Un pipeline dedicado de Concourse (`ci/nix-cache/pipeline.yml`) vigila nuevos PRs y pushes a `main`, compila las derivaciones de Nix relevantes, y las sube a Cachix. Los workers de Concourse tienen almacenamiento persistente y son ideales para builds de larga duración.

- **GitHub Actions** es el consumidor. Cada workflow configura Cachix con `skipPush: true`, lo que significa que descargará binarios precompilados de la caché pero nunca subirá nada. Los runners de GitHub Actions son efímeros, y tener muchos runners paralelos subiendo a la caché crearía uploads redundantes y posibles condiciones de carrera.

Este diseño mantiene la caché limpia y asegura que los builds sean rápidos en ambos sistemas de CI.

### Cómo funciona el pipeline de caché

El pipeline nix-cache de Concourse tiene cuatro jobs:

**`populate-nix-cache-pr`** es el caballo de batalla principal. Cuando se abre o actualiza un PR, compila derivaciones en una secuencia cuidadosamente ordenada:

1. Primero, verifica que el PR siga siendo el último commit (no tiene sentido compilar una revisión obsoleta).
2. Compila `lana-deps` — el árbol precompilado de dependencias de Rust. Este es el elemento más grande y valioso para almacenar en caché.
3. Una vez que las dependencias están en caché, compila varias cosas en paralelo: el ejecutor de `nextest`, los tests de `simulation`, `lana-cli-debug`, y el entorno de tests `bats`.
4. Finalmente, ejecuta `nix flake check`, compila el script `next-version`, y compila el binario de release completo.

Cada derivación se sube a Cachix inmediatamente cuando se completa (usando `cachix watch-exec`), así que las ejecuciones posteriores de GitHub Actions pueden utilizarlas incluso mientras el pipeline aún está trabajando en otras derivaciones.

**`cache-dev-profile`** almacena en caché el shell de `nix develop` y los scripts de utilidad de CI. Esto hace que `nix develop` sea rápido para los desarrolladores que usan Cachix.

**`build-release-main`** se activa con cada push a `main` y compila el binario de release. Esto mantiene la caché caliente para la ruta de build más común — cuando el pipeline de release se ejecuta después de un merge, las derivaciones que necesita ya están en caché.

**`rebuild-nix-cache`** es un job manual que recorre todos los PRs abiertos y re-dispara sus builds de caché. Esto es útil cuando una actualización de dependencias ha invalidado la caché y quieres reconstruir todo proactivamente.

### Uso de Cachix como desarrollador

Puedes acelerar tus comandos locales de `nix develop` y `nix build` configurando Cachix:

```bash
# Configuración única — agrega galoymoney como caché binaria
cachix use galoymoney

# Ahora esto descarga herramientas precompiladas en lugar de compilarlas
nix develop
```

Después de esto, Nix verificará la caché `galoymoney` antes de compilar cualquier cosa. Si la derivación que necesitas ya está allí, se descarga en segundos en lugar de compilarse en minutos.

### Qué sucede cuando la caché está fría

Si la caché no tiene lo que necesitas (un "cache miss"), Nix simplemente lo compila localmente. Esto es más lento pero siempre funciona — la caché es una optimización de rendimiento, no un requisito. El script de utilidad `wait-cachix-paths` está disponible en CI para casos donde un job necesita esperar a que la caché sea poblada por un pipeline paralelo antes de continuar.

---

## Modo Offline de SQLx

Lana usa [SQLx](https://github.com/launchbadge/sqlx) para consultas de base de datos, que proporciona verificación de SQL en tiempo de compilación — el compilador de Rust verifica tus consultas SQL contra el esquema real de la base de datos en tiempo de build. Esto es excelente para detectar errores, pero significa que necesitas una base de datos en ejecución para compilar el código.

Eso es un problema en CI, donde no hay base de datos disponible durante el paso de build. La solución es el **modo offline de SQLx**: guardas los metadatos de las consultas en un directorio `.sqlx/` (registrado en git), y los builds de CI usan esos metadatos almacenados en caché en lugar de conectarse a una base de datos real.

```bash
# Cuando tienes una base de datos ejecutándose localmente, regenera los metadatos
make sqlx-prepare

# En CI o cuando compilas sin base de datos
SQLX_OFFLINE=true cargo build
```

Si cambias una consulta SQL y olvidas ejecutar `make sqlx-prepare`, el build de CI fallará porque los metadatos offline no coincidirán con la consulta real. Esto es por diseño — mantiene los metadatos sincronizados con el código.

## Targets Comunes del Makefile

| Target | Qué hace |
|--------|----------|
| `make check-code-rust` | Compila todo el código Rust con `SQLX_OFFLINE=true` para verificar que compila |
| `make check-code-apps` | Ejecuta lint, verificación de tipos y build de las aplicaciones frontend Next.js |
| `make sqlx-prepare` | Regenera los metadatos de consultas offline de `.sqlx` (requiere una base de datos en ejecución) |
| `make sdl` | Regenera los archivos de esquema GraphQL a partir del código Rust |
| `make start-deps` | Inicia las dependencias de desarrollo local (PostgreSQL, Keycloak, etc.) |
| `make reset-deps` | Detiene las dependencias, borra las bases de datos, y reinicia todo limpio |
