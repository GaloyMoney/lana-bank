---
id: build-system
title: Build System
sidebar_position: 2
---

# Build System

This document describes the Nix-based build system used for Lana.

## Overview

Lana uses Nix for:

- Reproducible builds
- Development environments
- Cross-compilation
- Docker image creation

## Nix Flake Structure

```
flake.nix
├── packages
│   ├── lana-cli          # Main CLI binary
│   ├── admin-server      # Admin GraphQL server
│   ├── customer-server   # Customer GraphQL server
│   └── docker-image      # Docker container
├── devShells
│   └── default           # Development environment
└── checks
    └── tests             # CI test suite
```

## Development Shell

Enter the development environment:

```bash
nix develop
```

Provides:
- Rust toolchain
- Node.js and pnpm
- Python 3.13
- PostgreSQL client
- Development tools

## Building Packages

### Build CLI

```bash
nix build .#lana-cli
```

### Build All

```bash
nix build
```

### Cross-Compilation

```bash
# Build for Linux x86_64
nix build .#packages.x86_64-linux.lana-cli

# Build for Linux ARM64
nix build .#packages.aarch64-linux.lana-cli
```

## Docker Images

### Build Image

```bash
nix build .#docker-image
```

### Load Image

```bash
docker load < result
```

### Image Contents

```
/
├── bin/
│   ├── lana-cli
│   ├── admin-server
│   └── customer-server
└── etc/
    └── ssl/
```

## Cachix Integration

Binary caching for faster builds:

```bash
# Configure Cachix
cachix use lana-bank

# Push to cache (CI)
nix build | cachix push lana-bank
```

## Makefile Targets

| Target | Description |
|--------|-------------|
| `make build` | Build all Rust packages |
| `make check-code-rust` | Verify Rust compilation |
| `make check-code-apps` | Check frontend apps |
| `make docker` | Build Docker images |

## SQLx Offline Mode

For builds without database access:

```bash
# Update offline cache (with DB running)
make sqlx-prepare

# Build with offline mode
SQLX_OFFLINE=true cargo build
```

## Release Process

1. Update version in `Cargo.toml`
2. Create git tag
3. CI builds and pushes artifacts
4. Deploy to staging
5. Promote to production

