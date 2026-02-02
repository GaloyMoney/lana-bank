# Lana: Digital Asset-Backed Lending for Financial Institutions

**Lana** is a Bitcoin-backed lending platform that enables financial institutions of all sizes to offer fiat loans secured by Bitcoin collateral. Designed for easy integration, Lana streamlines the complex operational workflows associated with loan origination, collateral management, and liquidation.

## Key Features

- **Rapid Deployment** – Reduce time to market from months to weeks with Lana's modular architecture
- **Loan Origination & Management** – Automate loan creation, fee collection, and margin call management
- **Seamless Banking Integration** – Works with existing core banking systems, custodians, and regulatory frameworks
- **Security-First Design** – Adheres to industry security standards and best practices
- **Source Code Auditable** – Under Business Source License 1.1

For enterprise inquiries, contact **[biz@galoy.io](mailto:biz@galoy.io)**

---

## Setup & Development

### Dependencies

#### Nix package manager

- Recommended install method using https://github.com/DeterminateSystems/nix-installer
  ```
  curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
  ```

#### direnv >= 2.30.0

- Recommended install method from https://direnv.net/docs/installation.html:
  ```
  curl -sfL https://direnv.net/install.sh | bash
  echo "eval \"\$(direnv hook bash)\"" >> ~/.bashrc
  source ~/.bashrc
  ```

#### Docker

- Choose the install method for your system https://docs.docker.com/desktop/

### Environment Variables

Set them in your `.env` file

#### Optional

- `DAGSTER=true`: Enables the local dagster deployment.
- `TF_VAR_sa_creds`: Service account credentials into GCP (BigQuery & Documents access)
- `SUMSUB_KEY`: SumSub API key for identity verification
- `SUMSUB_SECRET`: SumSub API secret for identity verification
- `INGEST_HONEYCOMB_API_KEY`: Honeycomb Ingest API key for tracing
- `HONEYCOMB_API_KEY`: Honeycomb Configuration API key for dashboards 

### Start & Stop the stack

```bash
make dev-up   # Start the development stack
make dev-down # Stop the development stack
```

### MailCrab - Email Testing

- **SMTP Server**: Available at `localhost:1025`
- **Web Interface**: Available at [http://localhost:1080](http://localhost:1080)

### Access the Frontends

After bringing the development stack up, you can access the following services:

| Service         | URL                                                        | Notes                                 |
| --------------- | ---------------------------------------------------------- | ------------------------------------- |
| Admin Panel     | [http://admin.localhost:4455](http://admin.localhost:4455) | Admin panel for managing the platform |
| Customer Portal | [http://app.localhost:4455](http://app.localhost:4455)     | App for customers to see their data   |

#### Steps to access Admin Panel

1. Open [Admin Panel](http://admin.localhost:4455) in your browser
1. Use email `admin@galoy.io` to log in
1. You're in!

#### Steps to access Customer Portal

1. Create customer from Admin Panel
1. Open [Customer Portal](http://app.localhost:4455) in a separate browser (or incognito mode)
1. Use the customer email to try and login
1. Get the login code by running `make get-customer-login-code EMAIL=your-customer-email@example.com`
1. You're in!

> If you see a cookie error, delete the cookie and reload the page (but this should not happen if you're using separate browsers)

You might need to add these entries in your `/etc/hosts` file for authentication to work correctly on the customer portal:

```
127.0.0.1 app.localhost
::1 app.localhost
```

#### Steps to access Dagster web UI

- Simply visit `http://localhost:3000`.

### Running Tests

#### Unit Tests

```bash
make reset-deps next-watch
```

#### End-to-End Tests

```bash
make e2e
```

#### Cypress Tests

```bash
make dev-up # launch the full stack

# In a different terminal with tilt running:
cd apps/admin-panel && pnpm run cypress:run-ui # or run-headless
```

## BigQuery Setup

We use BigQuery for analytics and reporting. To set up the BigQuery tables, you need to have the `TF_VAR_sa_creds` environment variable set to the service account credentials.

Authenticate with Google Cloud SDK

```
gcloud auth application-default login
```

Verify access

```
gcloud auth application-default print-access-token
```

## Honeycomb Dashboards

We use Honeycomb for observability and tracing. To set up dashboards locally:

### Prerequisites

Set your Honeycomb API key TF_VAR_honeycomb_api_key in your .env.
Note: this needs to be a `Configuration API Keys`, not an `Ingest API Keys`

### Create Dashboards

```bash
make honeycomb-init   # Initialize OpenTofu
make honeycomb-plan   # Preview changes
make honeycomb-apply  # Create dashboards
```

---

## Release Process

Lana Bank uses an automated Release Candidate (RC) and release workflow managed through Concourse CI.

### Overview

The release workflow follows these stages:

1. **Continuous Testing** → Runs on every commit to `main`
2. **RC Build** → Creates release candidate after tests pass
3. **RC Promotion** → Automated PR creation for promoting RC to release
4. **Final Release** → Publishes release artifacts and updates deployment

### CI Pipeline Jobs

All of the jobs listed below are defined in `ci/release/pipeline.yml`.

#### 1. Test Jobs

- **`test-integration`**: Runs Rust integration tests using `cargo nextest`
- **`test-bats`**: Runs end-to-end BATS tests
- **`flake-check`**: Validates Nix flake configuration
- **`build-admin-panel-edge-image`**: Builds Admin Panel Docker image
- **`build-customer-portal-edge-image`**: Builds Customer Portal Docker image
- **`build-dagster-image`**: Builds Dagster data pipeline Docker image

The image building jobs ensure that the `build-rc` job only processes commits that have had images built on them  successfully.

All test jobs use Cachix for Nix dependency caching to speed up builds.

#### 2. Build RC and promote RC PR

**Trigger**: Automatically runs after all test jobs pass

The `build-rc` job:
- Generates next RC version (e.g., `0.39.0-rc.1`)
- Builds Rust binaries using Nix
- Builds Docker images for:
  - Admin Panel
  - Customer Portal
  - Dagster (data pipeline)
  - Backend services
- Tags all images with RC version
- Pushes images to Google Artifact Registry (GAR)
- Creates Git tag for the RC version

**Trigger**: Automatically runs after `build-rc` completes

The `open-promote-rc-pr` job:
- Generates the next final version (e.g., `0.39.0`)
- Updates `CHANGELOG.md` using [git-cliff](https://git-cliff.org/)
- Updates API documentation in `docs-site/`
- Creates a **draft PR** with:
  - Title: `ci: promote RC to {version}`
  - Labels: `promote-rc`, `galoybot`
  - Only allows changes to `CHANGELOG.md` and `docs-site/**`

**Important**: A GitHub workflow (`promote-rc-file-check.yml`) enforces that promote-rc PRs can **only** modify `CHANGELOG.md` and files under `docs-site/`. Any other file changes will cause the PR to fail.

#### 3. Release Job

**Trigger**: Runs when the promote-rc PR is **merged**

The `release` job:
- Rebuilds all binaries and Docker images with the final version tag
- Publishes Docker images to GAR with both `latest` and version tags
- Creates a GitHub Release with:
  - Release notes generated from commits
  - `lana-cli` binary as downloadable artifact
- Updates the version resource for future releases

#### 4. Chart Update Jobs

After a release, automated jobs update the Helm charts:

- **`bump-image-in-chart`**: Updates chart with released image digests
- **`bump-image-in-chart-rc`**: Updates chart with RC image digests

These jobs create PRs in the charts repository to update image references.

### Version Management

Versions are managed using [Concourse Semver Resource](https://github.com/concourse/semver-resource) and cocogitto:

- **RC versions**: `{major}.{minor}.{patch}-rc.{number}` (e.g., `0.39.0-rc.1`)
- **Final versions**: `{major}.{minor}.{patch}` (e.g., `0.39.0`)
- The `next-version` Nix app (in `ci/flake.nix`) calculates version bumps
