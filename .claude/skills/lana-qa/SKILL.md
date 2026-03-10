---
name: lana-qa
description: Run deterministic staging/QA testing with a prebuilt lana-admin, using the CLI's schema-derived workflow dependency graph for ordered stateful flows.
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

## Schema-Derived Workflow DAG

Use the CLI as the source of truth for dependency sequencing:

```bash
"${CLI[@]}" workflow list
"${CLI[@]}" workflow list --json
"${CLI[@]}" workflow list --yaml > workflow.yml
"${CLI[@]}" workflow deps --step credit_facility_disbursal_initiate
"${CLI[@]}" workflow deps --step credit_facility_disbursal_initiate --all
"${CLI[@]}" workflow deps --step deposit_record
```

## Known Environment Notes

- Supports `staging` (default) and `qa`.
- Staging may return transient HTTP `500` during active deployment windows. Wait `10-20 minutes` and retry.
- For GraphQL `Date` inputs such as `credit collateral update --effective`, prefer `YYYY-MM-DD`.

## Execution Guidance

- Treat `workflow deps` as the execution plan for any deep stateful action. Do not invent the order manually.
- For a target step, run `workflow deps --step <STEP> --all` first, then execute the returned steps in order.
- Existing customer fast path: if you already have a valid `customerId`, you can skip `prospect_create` and `prospect_convert` and start from the first step that requires `customerId` such as `deposit_account_create`, `credit_facility_proposal_create`, or `loan_agreement_generate`.
- Only use the fast path when the reused customer is in the correct state for the downstream flow and you already have the required IDs for any earlier prerequisites you are skipping.
- If you reuse an existing entity instead of creating a fresh one, verify that it already satisfies the missing prerequisite step before skipping it.
- `workflow deps` omits read-only bridge steps by default. Use `--all` when you need the full path, including query steps such as `pending_credit_facility`.
- Stop and inspect output if a required ID is missing or `null`. Do not guess the next step from memory.
- Run mutating commands with `--json` so IDs and statuses are easy to inspect.
- Use `--preview-graphql` when you want to inspect the request shape without sending it.
- Use credit wait helpers instead of ad-hoc polling when available:
  - `credit facility proposal-conclude --wait-for pending-ready`
  - `credit facility pending-get --wait-for completed`
  - `credit facility disbursal-initiate --wait-for confirmed`
- Keep deposit and credit flows dependency-ordered. Do not close/freeze entities before later steps that still need them.
