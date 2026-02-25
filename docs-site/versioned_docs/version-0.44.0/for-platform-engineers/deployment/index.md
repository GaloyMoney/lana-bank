---
id: index
title: Deployment & Release Engineering
sidebar_position: 1
---

# Deployment & Release Engineering

Getting code from a developer's laptop into production involves several systems working together. This section explains every step of that journey — from how we build the software, to how it gets tested, packaged into Docker images, bundled into Helm charts, and finally deployed across multiple environments.

If you're new to the project, start with the [CI/CD & Release Engineering](ci-cd) page. It walks through the entire pipeline end-to-end in the order things actually happen.

## The Big Picture

The diagram below shows the full path a code change takes. Don't worry if it looks like a lot — each step is explained in detail in the linked pages.

```mermaid
graph TD
    DEV["Developer pushes code"] --> PR["Pull Request on GitHub"]
    PR --> GHA["GitHub Actions<br/>(CI Checks)"]
    GHA -->|"All checks pass"| MERGE["Merge to main"]
    MERGE --> CONC["Concourse Release Pipeline<br/>(lana-bank repo)"]
    CONC --> RC["Build Release Candidate<br/>(Docker images + version tag)"]
    RC --> PROMOTE_PR["Open promote-rc PR<br/>(CHANGELOG + docs update)"]
    PROMOTE_PR -->|"Engineer merges PR"| REL["Final Release<br/>(GitHub Release + images)"]
    REL --> BUMP["Bump image digests<br/>in galoy-private-charts"]
    BUMP --> CHARTS_PR["PR to galoy-private-charts"]
    CHARTS_PR -->|"Auto-merged"| TF["Testflight<br/>(Helm deploy + smoketest)"]
    TF -->|"Tests pass"| DEPLOY_BUMP["Bump chart ref<br/>in galoy-deployments"]
    DEPLOY_BUMP --> CEPLER["Cepler environment gating"]
    CEPLER --> STAGING["Staging"]
    CEPLER --> QA["QA"]
    CEPLER --> PROD["Production"]
```

In short: code goes through **three repositories** before it reaches production. Each repository has its own CI pipeline, and each one adds a layer of validation.

## Three Repositories, Three Pipelines

| Repository | What lives here | What its CI does |
|------------|----------------|-----------------|
| **lana-bank** | Application source code | Runs tests on PRs (GitHub Actions), builds Docker images and creates releases (Concourse) |
| **galoy-private-charts** | Helm chart that bundles the app with all its dependencies | Deploys the chart to a throwaway namespace to verify it works ("testflight"), then pushes the chart reference forward |
| **galoy-deployments** | Per-environment Terraform configs and Cepler gating rules | Deploys to staging, QA, and production — in that order, with safety gates between each |

## The Technology Stack

```mermaid
graph TD
    subgraph Build["Build & CI"]
        NIX["Nix Flakes<br/>(Reproducible builds)"]
        CACHIX["Cachix<br/>(Binary cache)"]
        GHA2["GitHub Actions<br/>(PR checks)"]
        CONC2["Concourse<br/>(Release + deploy pipelines)"]
    end
    subgraph Packaging["Packaging & Promotion"]
        DOCKER["Docker Images<br/>(Google Artifact Registry)"]
        HELM["Helm Charts<br/>(galoy-private-charts)"]
        CEPLER2["Cepler<br/>(Environment gating)"]
    end
    subgraph Runtime["Runtime Services"]
        K8S["Kubernetes / GKE"]
        PG["PostgreSQL"]
        KC["Keycloak<br/>(Identity & Auth)"]
        OAT["Oathkeeper<br/>(API Gateway)"]
        DAG["Dagster<br/>(Data Pipelines)"]
        OTEL["OpenTelemetry<br/>(Observability)"]
    end
    Build --> Packaging --> Runtime
```

## Where to Go Next

- **[Build System](build-system)** — How Nix builds work, how the Cachix binary cache keeps things fast, and how Docker images are produced.
- **[CI/CD & Release Engineering](ci-cd)** — The main guide. Walks through every step from a PR all the way to production, including GitHub Actions, Concourse pipelines, Helm chart testing, Cepler environment gating, and production promotion.
