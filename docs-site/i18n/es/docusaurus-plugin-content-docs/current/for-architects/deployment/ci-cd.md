---
id: ci-cd
title: Pipeline CI/CD
sidebar_position: 5
---

# Pipeline de CI/CD

Este documento describe los flujos de trabajo de integración continua y despliegue continuo de Lana Bank, incluyendo GitHub Actions, Concourse y la estrategia de releases.

![Pipeline de CI/CD](/img/architecture/ci-cd-1.png)

## Visión General de la Arquitectura de CI/CD

```
┌─────────────────────────────────────────────────────────────────┐
│                    GitHub Actions                               │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   PR Check  │  │   Release   │  │   Security  │             │
│  │   Workflow  │  │   Workflow  │  │    Scan     │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                      Concourse CI                               │
│              ┌─────────────────────────┐                       │
│              │    Nix Cache Pipeline   │                       │
│              │    (Cachix population)  │                       │
│              └─────────────────────────┘                       │
└─────────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Artefactos y Registros                       │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │   Cachix    │  │    GHCR     │  │   GitHub    │             │
│  │  (Binarios) │  │  (Imágenes) │  │  Releases   │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
└─────────────────────────────────────────────────────────────────┘
```

## Separación de Responsabilidades

| Sistema | Responsabilidad |
|---------|-----------------|
| GitHub Actions | Pruebas, verificaciones, releases |
| Concourse | Caché de Nix, builds largos |
| Cachix | Almacenamiento de binarios Nix |
| GHCR | Registro de imágenes Docker |

## Workflows de GitHub Actions

### Inventario de Workflows

| Workflow | Trigger | Propósito |
|----------|---------|-----------|
| `check.yml` | PR, push to main | Verificaciones de código |
| `test.yml` | PR, push to main | Ejecución de pruebas |
| `release.yml` | Tag v* | Publicación de releases |
| `security.yml` | Schedule, PR | Escaneo de seguridad |
| `cypress.yml` | PR | Tests E2E |

### Workflow de Verificación (check.yml)

```yaml
name: Check

on:
  pull_request:
  push:
    branches: [main]

jobs:
  check:
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

      - name: Check Rust
        run: make check-code-rust

      - name: Check Apps
        run: make check-code-apps

      - name: Check Format
        run: cargo fmt --check

      - name: Clippy
        run: cargo clippy -- -D warnings

      - name: Check SQLx
        run: |
          cargo sqlx prepare --check
```

### Workflow de Tests (test.yml)

```yaml
name: Test

on:
  pull_request:
  push:
    branches: [main]

jobs:
  unit-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - name: Run tests
        run: nix build .#test-archive && nix run .#run-tests

  e2e-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_USER: lana
          POSTGRES_PASSWORD: lana
          POSTGRES_DB: lana
        ports:
          - 5433:5432

    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - name: Run E2E tests
        run: make e2e
```

### Integración con Cachix

```yaml
# Configuración común para todos los workflows
- uses: cachix/cachix-action@v13
  with:
    name: lana
    authToken: ${{ secrets.CACHIX_AUTH_TOKEN }}
    # Push solo en main
    pushFilter: ${{ github.ref == 'refs/heads/main' && '.*' || '-' }}
```

## Pipeline de Release

### Estructura del Pipeline

```yaml
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
            artifact: lana-cli-linux-x86_64
          - os: macos-latest
            target: x86_64-apple-darwin
            artifact: lana-cli-darwin-x86_64
          - os: macos-latest
            target: aarch64-apple-darwin
            artifact: lana-cli-darwin-aarch64

    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24

      - name: Build
        run: nix build .#lana-cli-${{ matrix.target }}

      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: ${{ matrix.artifact }}
          path: result/bin/lana-cli

  docker:
    needs: build
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build Docker image
        run: nix build .#lana-image

      - name: Push to GHCR
        run: |
          docker load < result
          docker tag lana:latest ghcr.io/galoymoney/lana:${{ github.ref_name }}
          docker push ghcr.io/galoymoney/lana:${{ github.ref_name }}

  release:
    needs: [build, docker]
    runs-on: ubuntu-latest
    steps:
      - uses: actions/download-artifact@v4

      - name: Create Release
        uses: softprops/action-gh-release@v1
        with:
          files: |
            lana-cli-linux-x86_64/lana-cli
            lana-cli-darwin-x86_64/lana-cli
            lana-cli-darwin-aarch64/lana-cli
          generate_release_notes: true
```

### Mecanismo de Bloqueo de Releases

Para evitar releases accidentales:

```yaml
# Requiere aprobación manual para producción
environment:
  name: production
  url: https://lana.example.com
```

## Pipeline de Caché Nix en Concourse

### Trabajos del Pipeline

```yaml
# ci/pipeline.yml
jobs:
  - name: cache-dev-profile
    plan:
      - get: lana-bank
        trigger: true
      - task: build-and-cache
        config:
          platform: linux
          image_resource:
            type: registry-image
            source: { repository: nixos/nix }
          run:
            path: sh
            args:
              - -c
              - |
                nix build .#devShell --json | \
                  jq -r '.[].outputs.out' | \
                  cachix push lana

  - name: populate-nix-cache-pr
    plan:
      - get: lana-bank-pr
        trigger: true
      - task: build-pr
        config:
          # Similar al anterior, pero para PRs
```

### Detección de Obsolescencia de PR

```bash
# ci/scripts/check-pr-stale.sh
#!/bin/bash
PR_NUMBER=$1
LATEST_SHA=$(gh pr view $PR_NUMBER --json headRefOid -q .headRefOid)
CACHED_SHA=$(cat .pr-cache-sha 2>/dev/null || echo "")

if [ "$LATEST_SHA" != "$CACHED_SHA" ]; then
    echo "PR has new commits, rebuilding cache"
    exit 0
else
    echo "PR cache is up to date"
    exit 1
fi
```

## Caché Binaria con Cachix

### Arquitectura de Caché

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Developer     │───▶│     Cachix      │◀───│      CI         │
│   nix build     │    │   lana.cachix   │    │   push builds   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
```

### Estrategia de Población

1. **Main branch**: Siempre push
2. **PRs**: Push condicional (solo si cambios significativos)
3. **Tags**: Push de release builds

### Configuración del Caché

```nix
# flake.nix
{
  nixConfig = {
    extra-substituters = [
      "https://lana.cachix.org"
    ];
    extra-trusted-public-keys = [
      "lana.cachix.org-1:XXXXXXXXX"
    ];
  };
}
```

## Distribución de Artefactos de Release

### Artefactos Binarios

| Plataforma | Arquitectura | Nombre |
|------------|--------------|--------|
| Linux | x86_64 | lana-cli-linux-x86_64 |
| Linux | aarch64 | lana-cli-linux-aarch64 |
| macOS | x86_64 | lana-cli-darwin-x86_64 |
| macOS | aarch64 | lana-cli-darwin-aarch64 |

### Imágenes de Contenedor

| Imagen | Registry | Tags |
|--------|----------|------|
| lana | ghcr.io/galoymoney/lana | latest, vX.Y.Z, sha-XXXXXX |
| meltano | ghcr.io/galoymoney/lana-meltano | latest, vX.Y.Z |

### Actualizaciones de Helm Chart

```yaml
# En release, actualizar Chart.yaml
- name: Update Helm Chart
  run: |
    yq -i '.appVersion = "${{ github.ref_name }}"' helm/lana/Chart.yaml
    yq -i '.version = "${{ github.ref_name }}"' helm/lana/Chart.yaml
```

## Scripts y Utilidades de CI

### Scripts Disponibles

| Script | Propósito |
|--------|-----------|
| `check-latest-commit.sh` | Verificar si PR está actualizado |
| `wait-cachix-paths.sh` | Esperar paths en Cachix |
| `bump-version.sh` | Incrementar versión |

### Implementación de check-latest-commit

```bash
#!/bin/bash
# ci/scripts/check-latest-commit.sh
set -e

BRANCH=$1
REPO=${2:-"GaloyMoney/lana-bank"}

MAIN_SHA=$(gh api repos/$REPO/commits/main --jq .sha)
BRANCH_BASE=$(gh api repos/$REPO/compare/main...$BRANCH --jq .merge_base_commit.sha)

if [ "$MAIN_SHA" = "$BRANCH_BASE" ]; then
    echo "Branch is up to date with main"
    exit 0
else
    echo "Branch needs rebase"
    exit 1
fi
```

## Gestión de Configuración del Pipeline

### Despliegue de Pipelines en Concourse

```bash
# Configurar pipeline
fly -t lana set-pipeline \
    -p lana-cache \
    -c ci/pipeline.yml \
    -l ci/values.yml
```

### Valores de Configuración

```yaml
# ci/values.yml
github_repo: GaloyMoney/lana-bank
cachix_cache: lana
slack_webhook: ((slack-webhook))
```

### Definiciones de Recursos

```yaml
# ci/pipeline.yml
resources:
  - name: lana-bank
    type: git
    source:
      uri: https://github.com/GaloyMoney/lana-bank.git
      branch: main

  - name: lana-bank-pr
    type: pull-request
    source:
      repository: GaloyMoney/lana-bank
      access_token: ((github-token))
```

## Resumen

El pipeline de CI/CD de Lana Bank combina:

1. **GitHub Actions**: Para verificaciones rápidas y releases
2. **Concourse**: Para builds largos y población de caché
3. **Cachix**: Para compartir binarios de Nix entre desarrolladores y CI
4. **GHCR**: Para distribución de imágenes Docker

Esta arquitectura permite:
- Feedback rápido en PRs
- Builds reproducibles
- Releases automatizados
- Caché eficiente de dependencias
