---
id: ci-cd
title: CI/CD Pipeline
sidebar_position: 5
---

# CI/CD Pipeline

This document describes the continuous integration and deployment pipeline.

![CI/CD Pipeline](/img/architecture/ci-cd-1.png)

## Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                    CI/CD PIPELINE                               │
│                                                                  │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │   Pull Request  │───▶│  GitHub Actions │                    │
│  │                 │    │   (CI Checks)   │                    │
│  └─────────────────┘    └─────────────────┘                    │
│                                │                                │
│                                ▼                                │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │                    Test Suite                            │   │
│  │  ┌────────┐ ┌────────┐ ┌────────┐ ┌────────┐           │   │
│  │  │ Lint   │ │ Unit   │ │ E2E    │ │Security│           │   │
│  │  │        │ │ Tests  │ │ Tests  │ │ Scan   │           │   │
│  │  └────────┘ └────────┘ └────────┘ └────────┘           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                │                                │
│                                ▼ (on merge)                     │
│  ┌─────────────────┐    ┌─────────────────┐                    │
│  │    Concourse    │───▶│     Deploy      │                    │
│  │   (CD Pipeline) │    │   to Staging    │                    │
│  └─────────────────┘    └─────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
```

## GitHub Actions

### PR Checks

```yaml
# .github/workflows/ci.yml
name: CI

on:
  pull_request:
    branches: [main]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - uses: cachix/cachix-action@v14
        with:
          name: lana-bank
      - run: nix develop -c make check-code-rust

  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - run: nix develop -c cargo nextest run

  e2e:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: cachix/install-nix-action@v24
      - run: nix develop -c make e2e
```

### Security Scanning

```yaml
security:
  runs-on: ubuntu-latest
  steps:
    - uses: actions/checkout@v4
    - name: Cargo audit
      run: cargo audit
    - name: Cargo deny
      run: cargo deny check
    - name: pnpm audit
      run: pnpm audit
```

## Concourse Pipeline

### Pipeline Definition

```yaml
# ci/pipeline.yml
resources:
  - name: lana-repo
    type: git
    source:
      uri: https://github.com/lana-bank/lana
      branch: main

jobs:
  - name: build
    plan:
      - get: lana-repo
        trigger: true
      - task: build
        file: lana-repo/ci/tasks/build.yml

  - name: deploy-staging
    plan:
      - get: lana-repo
        passed: [build]
      - task: deploy
        file: lana-repo/ci/tasks/deploy.yml
        params:
          ENVIRONMENT: staging
```

## Deployment Stages

### Staging

Automatic deployment on merge to main:

1. Build Docker images
2. Run database migrations
3. Deploy to GKE cluster
4. Run smoke tests

### Production

Manual promotion with approval:

1. Review staging metrics
2. Approve deployment
3. Blue-green deployment
4. Gradual traffic shift

## Cachix Caching

Binary caching for Nix builds:

```yaml
- uses: cachix/cachix-action@v14
  with:
    name: lana-bank
    authToken: '${{ secrets.CACHIX_AUTH_TOKEN }}'
    pushFilter: '(-source$|\.tar\.gz$)'
```

## Secrets Management

| Secret | Usage |
|--------|-------|
| `CACHIX_AUTH_TOKEN` | Nix binary cache |
| `GCP_SA_KEY` | GCloud deployment |
| `DOCKER_TOKEN` | Container registry |

## Monitoring

### Deployment Metrics

- Deployment frequency
- Lead time for changes
- Change failure rate
- Time to recovery

### Alerts

- Build failures
- Deployment failures
- Test flakiness
- Security vulnerabilities

## Rollback Procedure

```bash
# View deployment history
kubectl rollout history deployment/lana-api

# Rollback to previous version
kubectl rollout undo deployment/lana-api

# Rollback to specific revision
kubectl rollout undo deployment/lana-api --to-revision=2
```

