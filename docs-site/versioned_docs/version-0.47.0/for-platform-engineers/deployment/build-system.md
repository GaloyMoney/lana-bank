---
id: build-system
title: Build System
sidebar_position: 2
---

# Build System

Lana uses [Nix Flakes](https://nix.dev/concepts/flakes) for reproducible builds and [Cachix](https://www.cachix.org/) for binary caching. If you're not familiar with Nix, the short version is: it's a build system that guarantees the same inputs always produce the same output, no matter what machine you're building on. This eliminates the "works on my machine" problem for both development and CI.

This page covers how builds work locally, how CI uses the Nix cache, and how Docker images are produced for releases.

## Nix Flake Structure

Everything starts with the `flake.nix` file at the repository root. It defines all the build targets, development environments, and CI entry points:

```
flake.nix
├── packages
│   ├── lana-cli              # The main server/CLI binary (release build, optimized)
│   ├── lana-cli-debug        # A debug build (faster to compile, useful in development)
│   └── lana-deps             # Just the Rust dependency tree, pre-compiled (used for caching)
├── devShells
│   └── default               # The development environment with all tools
├── checks
│   └── flake-check           # Validates the flake itself
└── apps
    ├── nextest               # Runs the Rust test suite via cargo nextest
    ├── bats                  # Runs the BATS end-to-end tests
    └── simulation            # Runs facility scenario simulations
```

The `packages` section is what CI builds. The `apps` section provides convenient entry points for running tests — CI calls `nix run .#nextest` instead of wrangling `cargo nextest` manually, because the Nix app ensures all the right environment variables and dependencies are set up.

The `lana-deps` package is worth calling out: it pre-compiles just the Rust dependency tree (all the crates in `Cargo.lock`) without any of lana's own code. This is a caching optimization — since dependencies change far less often than application code, building them separately means they can be cached and reused across many builds.

## Development Shell

When you run `nix develop`, Nix sets up a shell with every tool you need for development:

```bash
nix develop
```

This gives you: the Rust stable toolchain, Node 20, pnpm 10, Python 3.13, a PostgreSQL client, sqlx-cli, and all the other utilities the project uses. You don't need to install any of these globally on your machine — Nix manages them for you and they don't interfere with other projects.

If you've configured Cachix (see below), this command is nearly instant because the pre-built shell environment is downloaded from the cache instead of being compiled on your machine.

## Building the Release Binary

There are two ways to build the `lana-cli` binary:

```bash
# Release build — optimized, used in CI for Docker images
nix build --impure .#lana-cli-release

# Debug build — faster to compile, has debug symbols
nix build .#lana-cli-debug
```

The release build uses the `--impure` flag because it reads environment variables (`VERSION`, `COMMITHASH`, `BUILDTIME`) that get baked into the binary. These are set by the CI pipeline so the running application knows what version it is and when it was built. In local development you'd typically use the debug build, which skips this and compiles much faster.

## Docker Images

Docker images are built during the Concourse release pipeline (see [CI/CD & Release Engineering](ci-cd) for the full picture). The key thing to understand is that there are two different Dockerfiles depending on whether we're building a release candidate or a final release:

- **`Dockerfile.rc`** is used for release candidates. The Nix build step compiles the binary, and this Dockerfile simply copies it into a minimal base image. This is fast because the binary is already compiled.

- **`Dockerfile.release`** is used for the final release. Instead of copying a local binary, it downloads the released binary from the GitHub Release. This makes the image build fully reproducible — anyone can rebuild the exact same image from the GitHub Release artifacts.

Both Dockerfiles use a **distroless base image**, which contains only the bare minimum needed to run a binary (no shell, no package manager, no utilities). This minimizes the attack surface and keeps the image small.

### The four images

Each release produces four Docker images, pushed to Google Artifact Registry:

| Image | What it contains | Registry |
|-------|-----------------|----------|
| `lana-bank` | The main lana-cli server binary | `gcr.io/galoyorg/lana-bank` |
| `lana-bank-admin-panel` | The admin panel Next.js app | `gcr.io/galoyorg/lana-bank-admin-panel` |
| `lana-bank-customer-portal` | The customer portal Next.js app | `gcr.io/galoyorg/lana-bank-customer-portal` |
| `dagster-code-location-lana-dw` | The Dagster data pipeline code | `us.gcr.io/galoyorg/dagster-code-location-lana-dw` |

### Build metadata

Every image build injects three pieces of information via a `.env` file so the running application can report what version it is:

- `VERSION` — the semantic version (e.g., `0.42.0`)
- `COMMITHASH` — the short git SHA it was built from
- `BUILDTIME` — a UTC timestamp of when the build happened

---

## Cachix Binary Caching

Here's the problem Cachix solves: Nix builds are perfectly reproducible, but they're slow when you're building from scratch. Compiling the Rust toolchain, all the dependencies, and the application binary can take a long time. If every CI run and every developer had to do this from scratch, it would be painful.

[Cachix](https://www.cachix.org/) is a binary cache for Nix. When someone builds a Nix derivation and pushes it to Cachix, everyone else who needs the same derivation can download the pre-built result instead of compiling it themselves. Since Nix derivations are content-addressed (the output is determined entirely by the inputs), this is safe — you'll always get the exact same result whether you build locally or download from the cache.

### Cache details

| | |
|---|---|
| **Cache name** | `galoymoney` |
| **URL** | `https://galoymoney.cachix.org` |
| **Who writes to it** | The Concourse nix-cache pipeline (with a write token) |
| **Who reads from it** | GitHub Actions workflows, Concourse jobs, and any developer who runs `cachix use galoymoney` |

### The two-system caching design

There's an intentional split between who builds for the cache and who reads from it:

- **Concourse** is the builder. A dedicated Concourse pipeline (`ci/nix-cache/pipeline.yml`) watches for new PRs and pushes to `main`, builds the relevant Nix derivations, and pushes them to Cachix. Concourse workers have persistent storage and are well-suited for long-running builds.

- **GitHub Actions** is the consumer. Every workflow configures Cachix with `skipPush: true`, meaning it will download pre-built binaries from the cache but never upload anything. GitHub Actions runners are ephemeral, and having many parallel runners push to the cache would create redundant uploads and potential race conditions.

This design keeps the cache clean and ensures builds are fast across both CI systems.

### How the cache pipeline works

The Concourse nix-cache pipeline has four jobs:

**`populate-nix-cache-pr`** is the main workhorse. When a PR is opened or updated, it builds derivations in a carefully ordered sequence:

1. First, it checks that the PR is still the latest commit (no point building a stale revision).
2. It builds `lana-deps` — the pre-compiled Rust dependency tree. This is the biggest and most valuable thing to cache.
3. Once dependencies are cached, it builds several things in parallel: the `nextest` runner, the `simulation` tests, `lana-cli-debug`, and the `bats` test environment.
4. Finally, it runs `nix flake check`, builds the `next-version` script, and builds the full release binary.

Each derivation is pushed to Cachix immediately when it completes (using `cachix watch-exec`), so subsequent GitHub Actions runs can pick them up even while the pipeline is still working on other derivations.

**`cache-dev-profile`** caches the `nix develop` shell and CI utility scripts. This makes `nix develop` fast for developers who use Cachix.

**`build-release-main`** triggers on every push to `main` and builds the release binary. This keeps the cache warm for the most common build path — when the release pipeline runs after a merge, the derivations it needs are already cached.

**`rebuild-nix-cache`** is a manual job that loops through all open PRs and re-triggers their cache builds. This is handy when a dependency update has invalidated the cache and you want to rebuild everything proactively.

### Using Cachix as a developer

You can speed up your local `nix develop` and `nix build` commands by configuring Cachix:

```bash
# One-time setup — adds galoymoney as a binary cache
cachix use galoymoney

# Now this downloads pre-built tools instead of compiling them
nix develop
```

After this, Nix will check the `galoymoney` cache before building anything. If the derivation you need is already there, it's downloaded in seconds instead of compiled in minutes.

### What happens when the cache is cold

If the cache doesn't have what you need (a "cache miss"), Nix simply builds it locally. This is slower but always works — the cache is a performance optimization, not a requirement. The `wait-cachix-paths` utility script is available in CI for cases where a job needs to wait for the cache to be populated by a parallel pipeline before proceeding.

---

## SQLx Offline Mode

Lana uses [SQLx](https://github.com/launchbadge/sqlx) for database queries, which provides compile-time SQL verification — the Rust compiler checks your SQL queries against the actual database schema at build time. This is great for catching bugs, but it means you need a running database to compile the code.

That's a problem in CI, where there's no database available during the build step. The solution is **SQLx offline mode**: you save the query metadata to a `.sqlx/` directory (checked into git), and CI builds use that cached metadata instead of connecting to a real database.

```bash
# When you have a database running locally, regenerate the metadata
make sqlx-prepare

# In CI or when building without a database
SQLX_OFFLINE=true cargo build
```

If you change a SQL query and forget to run `make sqlx-prepare`, the CI build will fail because the offline metadata won't match the actual query. This is by design — it keeps the metadata in sync with the code.

## Common Makefile Targets

| Target | What it does |
|--------|-------------|
| `make check-code-rust` | Compiles all Rust code with `SQLX_OFFLINE=true` to verify it builds |
| `make check-code-apps` | Lints, type-checks, and builds the Next.js frontend apps |
| `make sqlx-prepare` | Regenerates the `.sqlx` offline query metadata (requires a running database) |
| `make sdl` | Regenerates the GraphQL schema files from the Rust code |
| `make start-deps` | Starts local development dependencies (PostgreSQL, Keycloak, etc.) |
| `make reset-deps` | Stops dependencies, wipes the databases, and restarts everything clean |
