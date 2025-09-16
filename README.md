# Lana: Digital Asset-Backed Lending for Financial Institutions

**Lana** is a Bitcoin-backed lending platform that enables financial institutions of all sizes to offer fiat loans secured by Bitcoin collateral. Designed for easy integration, Lana streamlines the complex operational workflows associated with loan origination, collateral management, and liquidation.

## Key Features

- **Rapid Deployment** – Reduce time to market from months to weeks with Lana's modular architecture
- **Loan Origination & Management** – Automate loan creation, fee collection, and margin call management
- **Seamless Banking Integration** – Works with existing core banking systems, custodians, and regulatory frameworks
- **Security-First Design** – Adheres to industry security standards and best practices
- **Source Code Auditable** – Under Fair Source License

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

- `DATA_PIPELINE=true`: Enable data pipeline features (Meltano, Airflow)
- `TF_VAR_sa_creds`: Service account credentials into GCP (BigQuery & Documents access)
- `SUMSUB_KEY`: SumSub API key for identity verification
- `SUMSUB_SECRET`: SumSub API secret for identity verification

- `BROWSERSTACK_USERNAME`: BrowserStack username for e2e testing via Cypress
- `BROWSERSTACK_ACCESS_KEY`: BrowserStack access key for e2e testing via Cypress
- `HONEYCOMB_KEY`: Honeycomb API key for tracing
- `HONEYCOMB_DATASET`: Honeycomb dataset for tracing

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
1. Get the login code by running `make get-superadmin-login-code` or `make get-admin-login-code EMAIL=admin@galoy.io`
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
cd apps/admin-panel && pnpm run cypress:run ui # or headless
# or if you want to run the tests via browserstack - needs BROWSERSTACK_USERNAME and BROWSERSTACK_ACCESS_KEY in env
cd apps/admin-panel && pnpm run cypress:run browserstack
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

### Adding new BigQuery tables

Commands to re-run when adding new BQ tables

```
git checkout pre-merged-commit
# this is important to have the previous state before pulling
make reset-deps
git pull
make init-bq
```

If you are doing work that requires adding a new big query table you need to add it to `./tf/bq-setup/bq.tf`
