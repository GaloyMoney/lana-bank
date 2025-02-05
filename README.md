# Digital Asset-Backed Lending for Financial Institutions

**Lana** is a Bitcoin-backed lending platform that enables financial institutions of all sizes to offer fiat loans secured by Bitcoin collateral. Designed for easy integration, Lana streamlines the complex operational workflows associated with loan origination, collateral management, and liquidation.

## Key Features

- **Rapid Deployment** – Reduce time to market from months to weeks with Lana’s modular architecture.
- **Loan Origination & Management** – Automate loan creation, fee collection, and margin call management.
- **Seamless Banking Integration** – Works with existing core banking systems, custodians, and regulatory frameworks.
- **Security-First Design** – Adheres to industry security standards and best practices.
- **Source Code Auditable** – Under Fair Source License.

For enterprise inquiries, contact **[biz@galoy.io](mailto:biz@galoy.io)**.


# Lana bank

**to run entire stack**
```bash
make dev-up # Bring the stack up
make dev-down # Bring the stack down
```

**to run (unit) tests:**

```bash
make reset-deps next-watch
```

**to run e2e tests:**

```bash
make e2e
```

**to run the server:**

```
make run-server
```

### To fetch latest `cala-enterprise` build

1. Auth with `$ gcloud auth login`

1. Configure docker `$ gcloud auth configure-docker`

1. Run `$ make bump-cala-docker-image` to test that image can be fetched

### To update cala-enterprise schema

1. Create a github "fine-grained token" with **Content** read-only permission

1. Add token to `.env` file as `export GITHUB_TOKEN=<token-here>`

1. Run `$ direnv allow` to source token

1. Run `$ make bump-cala-schema` to update schema

# access the frontends:

the access through the frontends needs to be proxied to oathkeeper to receive the correct Header

admin panel: http://localhost:4455/admin-panel

use email admin@galoy.io
connect to http://0.0.0.0:8025/

app: http://localhost:4455/

- if you see a cookie error, delete the cookie and reload the page (for now)

# To setup BQ 

ensure you have the TF_VAR_sa_creds env variable in .env 

run 

```
gcloud auth application-default login
```

you can verify you already have access by running 
```
gcloud auth application-default print-access-token
```

commands to re-run when adding new BQ tables:

```
git checkout pre-merged-commit
# this is important to have the previous state before pulling
make reset-deps
git pull
make init-bq
```

If you are doing work that requires adding a new big query table you need to add it to `./tf/cala-setup/bq.tf`
