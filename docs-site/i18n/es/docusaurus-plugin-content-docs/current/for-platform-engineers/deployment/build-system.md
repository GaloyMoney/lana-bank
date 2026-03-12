---
id: build-system
title: Sistema de Compilación
sidebar_position: 2
---

# Sistema de Compilación

Lana utiliza [Nix Flakes](https://nix.dev/concepts/flakes) para compilaciones reproducibles y [Cachix](https://www.cachix.org/) para almacenamiento en caché binario. Si no estás familiarizado con Nix, la versión corta es: es un sistema de compilación que garantiza que las mismas entradas siempre produzcan la misma salida, sin importar en qué máquina estés compilando. Esto elimina el problema de "funciona en mi máquina" tanto para desarrollo como para CI.

Esta página cubre cómo funcionan las compilaciones localmente, cómo CI utiliza la caché de Nix y cómo se producen las imágenes de Docker para los lanzamientos.

## Estructura del Nix Flake

Todo comienza con el archivo `flake.nix` en la raíz del repositorio. Define todos los objetivos de compilación, entornos de desarrollo y puntos de entrada de CI:

```
flake.nix
├── packages
│   ├── lana-cli              # El binario principal del servidor/CLI (compilación de lanzamiento, optimizada)
│   ├── lana-cli-debug        # Una compilación de depuración (más rápida de compilar, útil en desarrollo)
│   └── lana-deps             # Solo el árbol de dependencias de Rust, precompilado (usado para almacenamiento en caché)
├── devShells
│   └── default               # El entorno de desarrollo con todas las herramientas
├── checks
│   └── flake-check           # Valida el flake mismo
└── apps
    ├── nextest               # Ejecuta la suite de pruebas de Rust mediante cargo nextest
    ├── bats                  # Ejecuta las pruebas de extremo a extremo de BATS
    └── simulation            # Ejecuta simulaciones de escenarios de instalaciones
```

La sección `packages` es lo que compila CI. La sección `apps` proporciona puntos de entrada convenientes para ejecutar pruebas — CI llama a `nix run .#nextest` en lugar de manipular `cargo nextest` manualmente, porque la aplicación Nix asegura que todas las variables de entorno y dependencias correctas estén configuradas.

El paquete `lana-deps` merece mención especial: precompila solo el árbol de dependencias de Rust (todos los crates en `Cargo.lock`) sin ningún código propio de lana. Esta es una optimización de almacenamiento en caché — dado que las dependencias cambian con mucha menos frecuencia que el código de la aplicación, compilarlas por separado significa que pueden almacenarse en caché y reutilizarse en muchas compilaciones.

## Entorno de Desarrollo

Cuando ejecutas `nix develop`, Nix configura un entorno con todas las herramientas que necesitas para el desarrollo:

```bash
nix develop
```

Esto te proporciona: el toolchain estable de Rust, Node 20, pnpm 10, Python 3.13, un cliente de PostgreSQL, sqlx-cli y todas las demás utilidades que usa el proyecto. No necesitas instalar ninguna de estas herramientas globalmente en tu máquina — Nix las gestiona por ti y no interfieren con otros proyectos.

Si has configurado Cachix (ver más abajo), este comando es casi instantáneo porque el entorno precompilado se descarga desde la caché en lugar de compilarse en tu máquina.

## Compilar el Binario de Producción

Hay dos formas de compilar el binario `lana-cli`:

```bash

# Compilación de producción — optimizada, usada en CI para imágenes Docker

nix build --impure .#lana-cli-release

# Compilación de depuración — más rápida de compilar, incluye símbolos de depuración

nix build .#lana-cli-debug
```

La compilación de producción usa el flag `--impure` porque lee variables de entorno (`VERSION`, `COMMITHASH`, `BUILDTIME`) que se integran en el binario. Estas son establecidas por el pipeline de CI para que la aplicación en ejecución sepa qué versión es y cuándo fue compilada. En desarrollo local normalmente usarías la compilación de depuración, que omite esto y compila mucho más rápido.

## Imágenes Docker

Las imágenes Docker se compilan durante el pipeline de lanzamiento de Concourse (consulta [CI/CD e Ingeniería de Lanzamientos](ci-cd) para ver el panorama completo). Lo importante es entender que hay dos Dockerfiles diferentes dependiendo de si estamos compilando un candidato de lanzamiento o un lanzamiento final:

- **`Dockerfile.rc`** se usa para candidatos de lanzamiento. El paso de compilación de Nix compila el binario, y este Dockerfile simplemente lo copia en una imagen base mínima. Esto es rápido porque el binario ya está compilado.

- **`Dockerfile.release`** se usa para el lanzamiento final. En lugar de copiar un binario local, descarga el binario publicado desde el Release de GitHub. Esto hace que la compilación de la imagen sea completamente reproducible — cualquiera puede reconstruir exactamente la misma imagen a partir de los artefactos del Release de GitHub.

Ambos Dockerfiles usan una **imagen base distroless**, que contiene solo lo mínimo necesario para ejecutar un binario (sin shell, sin gestor de paquetes, sin utilidades). Esto minimiza la superficie de ataque y mantiene la imagen pequeña.

### Las cuatro imágenes

Cada lanzamiento produce cuatro imágenes Docker, enviadas a Google Artifact Registry:

| Imagen | Qué contiene | Registro |
|-------|-----------------|----------|
| `lana-bank` | El binario principal del servidor lana-cli | `gcr.io/galoyorg/lana-bank` |
| `lana-bank-admin-panel` | La aplicación Next.js del panel de administración | `gcr.io/galoyorg/lana-bank-admin-panel` |
| `lana-bank-customer-portal` | La aplicación Next.js del portal de clientes | `gcr.io/galoyorg/lana-bank-customer-portal` |
| `dagster-code-location-lana-dw` | El código del pipeline de datos Dagster | `us.gcr.io/galoyorg/dagster-code-location-lana-dw` |

### Metadatos de compilación

Cada compilación de imagen inyecta tres piezas de información a través de un archivo `.env` para que la aplicación en ejecución pueda reportar qué versión es:

- `VERSION` — la versión semántica (ej., `0.42.0`)
- `COMMITHASH` — el SHA corto de git desde el cual se compiló
- `BUILDTIME` — una marca de tiempo UTC de cuándo ocurrió la compilación

---

## Almacenamiento en Caché Binario de Cachix

Este es el problema que Cachix resuelve: Las compilaciones de Nix son perfectamente reproducibles, pero son lentas cuando estás compilando desde cero. Compilar la cadena de herramientas de Rust, todas las dependencias y el binario de la aplicación puede llevar mucho tiempo. Si cada ejecución de CI y cada desarrollador tuviera que hacer esto desde cero, sería doloroso.

[Cachix](https://www.cachix.org/) es una caché binaria para Nix. Cuando alguien compila una derivación de Nix y la envía a Cachix, todos los demás que necesiten la misma derivación pueden descargar el resultado precompilado en lugar de compilarlo ellos mismos. Dado que las derivaciones de Nix están direccionadas por contenido (la salida está determinada completamente por las entradas), esto es seguro — siempre obtendrás exactamente el mismo resultado ya sea que compiles localmente o descargues desde la caché.

### Detalles de la caché

| | |
|---|---|
| **Nombre del caché** | `galoymoney` |
| **URL** | `https://galoymoney.cachix.org` |
| **Quién escribe en él** | El pipeline nix-cache de Concourse (con un token de escritura) |
| **Quién lee de él** | Flujos de trabajo de GitHub Actions, tareas de Concourse y cualquier desarrollador que ejecute `cachix use galoymoney` |

### El diseño de almacenamiento en caché de dos sistemas

Existe una división intencional entre quién construye para el caché y quién lee de él:

- **Concourse** es el constructor. Un pipeline dedicado de Concourse (`ci/nix-cache/pipeline.yml`) monitorea nuevos PRs y pushes a `main`, construye las derivaciones Nix relevantes y las envía a Cachix. Los workers de Concourse tienen almacenamiento persistente y están bien preparados para compilaciones de larga duración.

- **GitHub Actions** es el consumidor. Cada flujo de trabajo configura Cachix con `skipPush: true`, lo que significa que descargará binarios preconstruidos del caché pero nunca subirá nada. Los runners de GitHub Actions son efímeros, y tener muchos runners paralelos enviando al caché crearía cargas redundantes y posibles condiciones de carrera.

Este diseño mantiene el caché limpio y garantiza que las compilaciones sean rápidas en ambos sistemas de CI.

### Cómo funciona el pipeline de caché

El pipeline nix-cache de Concourse tiene cuatro trabajos:

**`populate-nix-cache-pr`** es el caballo de batalla principal. Cuando se abre o actualiza un PR, construye derivaciones en una secuencia cuidadosamente ordenada:

1. Primero, verifica que el PR siga siendo el último commit (no tiene sentido construir una revisión obsoleta).
2. Construye `lana-deps` — el árbol de dependencias Rust precompilado. Esto es lo más grande y valioso para almacenar en caché.
3. Una vez que las dependencias están en caché, construye varias cosas en paralelo: el runner `nextest`, las pruebas `simulation`, `lana-cli-debug` y el entorno de pruebas `bats`.
4. Finalmente, ejecuta `nix flake check`, construye el script `next-version` y construye el binario de release completo.

Cada derivación se envía a Cachix inmediatamente cuando se completa (usando `cachix watch-exec`), por lo que las ejecuciones posteriores de GitHub Actions pueden recogerlas incluso mientras el pipeline todavía está trabajando en otras derivaciones.

**`cache-dev-profile`** almacena en caché el shell `nix develop` y los scripts de utilidad de CI. Esto hace que `nix develop` sea rápido para los desarrolladores que usan Cachix.

**`build-release-main`** se activa con cada push a `main` y construye el binario de release. Esto mantiene el caché activo para la ruta de compilación más común — cuando el pipeline de release se ejecuta después de un merge, las derivaciones que necesita ya están en caché.

**`rebuild-nix-cache`** es un trabajo manual que recorre todos los PRs abiertos y vuelve a activar sus compilaciones de caché. Esto es útil cuando una actualización de dependencias ha invalidado el caché y deseas reconstruir todo de forma proactiva.

### Usando Cachix como desarrollador

Puedes acelerar tus comandos locales `nix develop` y `nix build` configurando Cachix:

```bash

# Configuración única — añade galoymoney como caché binaria

cachix use galoymoney

# Ahora esto descarga herramientas precompiladas en lugar de compilarlas

nix develop
```

Después de esto, Nix verificará la caché `galoymoney` antes de compilar cualquier cosa. Si la derivación que necesitas ya está allí, se descarga en segundos en lugar de compilarse en minutos.

### Qué sucede cuando la caché está vacía

Si la caché no tiene lo que necesitas (un "fallo de caché"), Nix simplemente lo compila localmente. Esto es más lento pero siempre funciona — la caché es una optimización de rendimiento, no un requisito. El script de utilidad `wait-cachix-paths` está disponible en CI para casos donde un trabajo necesita esperar a que la caché sea poblada por un pipeline paralelo antes de continuar.

---

## Modo Offline de SQLx

Lana utiliza [SQLx](https://github.com/launchbadge/sqlx) para consultas de base de datos, que proporciona verificación SQL en tiempo de compilación — el compilador de Rust verifica tus consultas SQL contra el esquema real de la base de datos durante la compilación. Esto es excelente para detectar errores, pero significa que necesitas una base de datos en ejecución para compilar el código.

Eso es un problema en CI, donde no hay ninguna base de datos disponible durante el paso de compilación. La solución es el **modo offline de SQLx**: guardas los metadatos de las consultas en un directorio `.sqlx/` (incluido en git), y las compilaciones de CI usan esos metadatos en caché en lugar de conectarse a una base de datos real.

```bash

# Cuando tengas una base de datos ejecutándose localmente, regenera los metadatos

make sqlx-prepare

# En CI o al compilar sin una base de datos

SQLX_OFFLINE=true cargo build
```

Si cambias una consulta SQL y olvidas ejecutar `make sqlx-prepare`, la compilación en CI fallará porque los metadatos sin conexión no coincidirán con la consulta real. Esto es intencional: mantiene los metadatos sincronizados con el código.

## Objetivos Comunes del Makefile

| Objetivo | Qué hace |
|--------|-------------|
| `make check-code-rust` | Compila todo el código Rust con `SQLX_OFFLINE=true` para verificar que se compile |
| `make check-code-apps` | Analiza, verifica tipos y compila las aplicaciones frontend de Next.js |
| `make sqlx-prepare` | Regenera los metadatos de consultas sin conexión `.sqlx` (requiere una base de datos en ejecución) |
| `make sdl` | Regenera los archivos de esquema GraphQL a partir del código Rust |
| `make start-deps` | Inicia las dependencias de desarrollo locales (PostgreSQL, Keycloak, etc.) |
| `make reset-deps` | Detiene las dependencias, limpia las bases de datos y reinicia todo desde cero |
