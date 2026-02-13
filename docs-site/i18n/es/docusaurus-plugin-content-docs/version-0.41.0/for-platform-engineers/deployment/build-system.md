---
id: build-system
title: Sistema de Build
sidebar_position: 2
---

# Sistema de Build y Despliegue

Este documento describe el sistema de build de Lana Bank, incluyendo la arquitectura basada en Nix, compilación cruzada para binarios estáticos, y construcción de imágenes Docker.

## Arquitectura del Sistema de Build

```
┌─────────────────────────────────────────────────────────────────┐
│                         flake.nix                               │
│                  (Definición del proyecto)                      │
└─────────────────────────────────────────────────────────────────┘
                              │
        ┌─────────────────────┼─────────────────────┐
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  devShells    │   │   packages    │   │    apps       │
│ (Desarrollo)  │   │  (Artefactos) │   │  (Ejecutables)│
└───────────────┘   └───────────────┘   └───────────────┘
        │                     │                     │
        ▼                     ▼                     ▼
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│ nix develop   │   │ nix build     │   │  nix run      │
└───────────────┘   └───────────────┘   └───────────────┘
```

## Perfiles de Build y Artefactos

### Artefactos Principales

| Artefacto | Descripción | Comando |
|-----------|-------------|---------|
| lana-cli | Binario principal del servidor | `nix build .#lana-cli` |
| admin-panel | Frontend de administración | `nix build .#admin-panel` |
| customer-portal | Frontend de clientes | `nix build .#customer-portal` |
| meltano-image | Imagen de pipelines de datos | `nix build .#meltano-image` |

### Características de Build

```nix
# flake.nix
{
  packages = {
    lana-cli = craneLib.buildPackage {
      src = ./.;
      pname = "lana-cli";
      cargoExtraArgs = "--package lana-cli";

      buildInputs = [
        openssl
        pkg-config
      ];

      # Features de Cargo
      cargoFeatures = [
        "production"
        "mock-custodian"  # Solo en desarrollo/pruebas
      ];
    };
  };
}
```

## Compilación Cruzada para Binarios Estáticos

### Configuración de Compilación Cruzada

Para producir binarios completamente estáticos que funcionen en cualquier distribución Linux:

```nix
# Configuración para x86_64-unknown-linux-musl
{
  packages.lana-cli-static = craneLib.buildPackage {
    src = ./.;
    pname = "lana-cli";

    CARGO_BUILD_TARGET = "x86_64-unknown-linux-musl";
    CARGO_BUILD_RUSTFLAGS = "-C target-feature=+crt-static";

    buildInputs = [
      pkgsCross.musl64.stdenv.cc
      pkgsStatic.openssl
    ];
  };
}
```

### Proceso de Build Estático en CI

```yaml
# .github/workflows/release.yml
jobs:
  build-static:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - uses: cachix/install-nix-action@v24
        with:
          github_access_token: ${{ secrets.GITHUB_TOKEN }}

      - uses: cachix/cachix-action@v13
        with:
          name: lana
          authToken: ${{ secrets.CACHIX_AUTH_TOKEN }}

      - name: Build static binary
        run: nix build .#lana-cli-static

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: lana-cli-linux-x86_64
          path: result/bin/lana-cli
```

## Construcción de Imágenes Docker

### Estrategias de Build de Imágenes

```nix
# Imagen minimalista basada en scratch
{
  packages.lana-image = dockerTools.buildImage {
    name = "lana";
    tag = "latest";

    copyToRoot = buildEnv {
      name = "image-root";
      paths = [ self.packages.lana-cli-static ];
    };

    config = {
      Cmd = [ "/bin/lana-cli" ];
      ExposedPorts = {
        "5253/tcp" = {};
        "5254/tcp" = {};
      };
      Env = [
        "RUST_LOG=info"
      ];
    };
  };
}
```

### Variantes de Imágenes

| Imagen | Base | Tamaño | Uso |
|--------|------|--------|-----|
| lana:latest | scratch | ~50MB | Producción |
| lana:debug | debian:slim | ~150MB | Depuración |
| meltano:latest | python:3.11 | ~500MB | Pipelines de datos |

### Proceso de Build de la Imagen de Meltano

```nix
{
  packages.meltano-image = dockerTools.buildImage {
    name = "meltano";
    tag = "latest";

    fromImage = dockerTools.pullImage {
      imageName = "python";
      imageDigest = "sha256:...";
      sha256 = "...";
    };

    copyToRoot = buildEnv {
      name = "meltano-root";
      paths = [
        meltano
        tap-postgres
        tap-bitfinexapi
        tap-sumsubapi
        target-bigquery
        dbt
      ];
    };

    config = {
      WorkingDir = "/app";
      Cmd = [ "meltano" ];
    };
  };
}
```

## Publicación de Artefactos y Registros

### Configuración de Registros

| Registro | Propósito | URL |
|----------|-----------|-----|
| GitHub Container Registry | Imágenes públicas | ghcr.io/galoymoney/lana |
| Cachix | Caché binaria de Nix | lana.cachix.org |
| GitHub Releases | Binarios estáticos | github.com/GaloyMoney/lana-bank/releases |

### Estrategia de Etiquetado de Imágenes

```yaml
# Tags automáticos
- latest        # Última versión de main
- v1.2.3        # Release con tag
- sha-abc1234   # Commit específico
- pr-123        # Pull request
```

### Releases de GitHub

```yaml
# .github/workflows/release.yml
- name: Create Release
  uses: softprops/action-gh-release@v1
  with:
    files: |
      lana-cli-linux-x86_64
      lana-cli-linux-aarch64
      lana-cli-darwin-x86_64
      lana-cli-darwin-aarch64
    generate_release_notes: true
```

### Caché Binaria de Cachix

```yaml
# Push a Cachix después de build exitoso
- name: Push to Cachix
  run: |
    nix build .#lana-cli --json | jq -r '.[].outputs.out' | cachix push lana
```

## Flujo de Trabajo de Despliegue

### Etapas del Pipeline

```
┌─────────────┐   ┌─────────────┐   ┌─────────────┐   ┌─────────────┐
│   Build     │──▶│    Test     │──▶│   Publish   │──▶│   Deploy    │
│             │   │             │   │             │   │             │
│ - Compile   │   │ - Unit      │   │ - Images    │   │ - Staging   │
│ - Lint      │   │ - E2E       │   │ - Binaries  │   │ - Prod      │
│ - Check     │   │ - BATS      │   │ - Cachix    │   │             │
└─────────────┘   └─────────────┘   └─────────────┘   └─────────────┘
```

### Pipeline de Release

```yaml
# Triggered on tag push (v*)
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-musl
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - name: Build
        run: nix build .#lana-cli-${{ matrix.target }}

  publish-images:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - name: Build and push Docker image
        run: |
          nix build .#lana-image
          docker load < result
          docker push ghcr.io/galoymoney/lana:${{ github.ref_name }}
```

## Comandos del Sistema de Build

### Comandos Basados en Nix (Recomendados para CI)

```bash
# Construir binario
nix build .#lana-cli

# Construir imagen Docker
nix build .#lana-image

# Construir todos los paquetes
nix build .#all

# Ejecutar pruebas
nix build .#test-archive
nix run .#run-tests

# Verificar formato
nix build .#check-fmt
```

### Targets del Makefile

```bash
# Desarrollo
make check-code-rust    # Verificar compilación
make check-code-apps    # Verificar frontend

# Pruebas
make test               # Ejecutar tests de Rust
make e2e                # Ejecutar tests BATS
make cypress            # Ejecutar tests Cypress

# Calidad
make fmt                # Formatear código
make lint               # Ejecutar linters
make sqlx-prepare       # Actualizar caché SQLx
```

### Shell de Desarrollo

```bash
# Entrar al shell con todas las herramientas
nix develop

# Shell con herramientas mínimas
nix develop .#minimal

# Verificar herramientas disponibles
which cargo rustc node pnpm
```

## Filtrado de Fuentes para Builds Reproducibles

```nix
# Filtrar archivos no necesarios para el build
{
  src = lib.cleanSourceWith {
    src = ./.;
    filter = path: type:
      let
        name = baseNameOf path;
      in
        # Incluir solo archivos necesarios
        (type == "regular" && (
          lib.hasSuffix ".rs" name ||
          lib.hasSuffix ".toml" name ||
          name == "Cargo.lock"
        )) ||
        # Incluir directorios
        (type == "directory" && !(
          name == "target" ||
          name == "node_modules" ||
          name == ".git"
        ));
  };
}
```
