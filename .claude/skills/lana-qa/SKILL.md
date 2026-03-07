---
name: lana-qa
description: Run deterministic staging/QA testing with a prebuilt lana-admin, using the CLI's built-in workflow dependency graph for ordered stateful flows.
---

# Lana Admin QA/Staging

Use this skill to run deterministic admin checks against `staging` or `qa` with the prebuilt `skills/lana-qa/lana-admin`.

## Setup

```bash
export LANA_USERNAME="galoysuperuser@mailinator.com"
export LANA_ENV="${LANA_ENV:-staging}"
export LANA_KEYCLOAK_CLIENT_ID="${LANA_KEYCLOAK_CLIENT_ID:-admin-panel}"

case "${LANA_ENV}" in
  staging)
    export LANA_ADMIN_URL="https://admin.staging.galoy.io/graphql"
    export LANA_KEYCLOAK_URL="https://auth.staging.galoy.io"
    ;;
  qa)
    export LANA_ADMIN_URL="https://admin.qa.galoy.io/graphql"
    export LANA_KEYCLOAK_URL="https://auth.qa.galoy.io"
    ;;
  *)
    echo "Unsupported LANA_ENV='${LANA_ENV}'. Expected 'staging' or 'qa'." >&2
    exit 1
    ;;
esac

CLI_BIN="./skills/lana-qa/lana-admin"
[[ -x "${CLI_BIN}" ]] || {
  echo "Missing lana-admin at ${CLI_BIN}" >&2
  exit 1
}

CLI=("${CLI_BIN}")
```

Default target is `staging`. To target QA, set `export LANA_ENV=qa`.

If staging returns HTTP `500`, assume deployment is in progress. Wait `10-20 minutes`, then retry.

## Login Once

```bash
"${CLI[@]}" auth login --username "${LANA_USERNAME}"
"${CLI[@]}" auth info --json
```

The `auth login` command reads `LANA_ADMIN_URL`, `LANA_KEYCLOAK_URL`, `LANA_KEYCLOAK_CLIENT_ID`, and `LANA_USERNAME` from the environment, saves the session locally, and all subsequent commands should omit auth flags.

## Built-In Workflow DAG

Use the CLI as the source of truth for dependency sequencing:

```bash
"${CLI[@]}" workflow deps --step credit_facility_partial_payment_record
"${CLI[@]}" workflow deps --step deposit_close --all
```

The dependency graph is embedded in the binary. Do not rely on local YAML copies or helper scripts.

## Known Environment Notes

- Supports `staging` (default) and `qa`.
- Staging may return transient HTTP `500` during active deployment windows. Wait `10-20 minutes` and retry.
- For GraphQL `Date` inputs such as `credit collateral update --effective`, prefer `YYYY-MM-DD`.

## Execution Guidance

- Use `workflow deps` before running a deep stateful step.
- Run mutating commands with `--json` so IDs and statuses are easy to inspect.
- Use `--preview-graphql` when you want to inspect the request shape without sending it.
- Keep deposit and credit flows dependency-ordered. Do not close/freeze entities before later steps that still need them.
