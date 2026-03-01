---
name: lana-qa
description: Run broad exploratory testing on QA with a prebuilt lana-admin-cli by enumerating all available commands and executing coverage plus randomized mixed actions across customers, loans, and related domains.
---

# Lana Admin CLI QA Exploration

Goal: exercise the QA admin API broadly through `lana-admin-cli` only.  
Do not rely on external Python harnesses; use CLI help + shell commands.

## Setup

```bash
export LANA_USERNAME="galoysuperuser@mailinator.com"
export LANA_ADMIN_URL="https://admin.qa.lana.galoy.io/graphql"
export LANA_KEYCLOAK_URL="https://auth.qa.lana.galoy.io"
export LANA_KEYCLOAK_CLIENT_ID="${LANA_KEYCLOAK_CLIENT_ID:-admin-panel}"

CLI_BIN="./lana-admin-cli"
[[ -x "${CLI_BIN}" ]] || { echo "Missing ./lana-admin-cli" >&2; exit 1; }

CLI=(
  "${CLI_BIN}"
  --admin-url "${LANA_ADMIN_URL}"
  --keycloak-url "${LANA_KEYCLOAK_URL}"
  --keycloak-client-id "${LANA_KEYCLOAK_CLIENT_ID}"
  --username "${LANA_USERNAME}"
)
```

## Auth

```bash
"${CLI[@]}" login
"${CLI[@]}" customer list --first 1 --json >/dev/null
```

## Discover All Actions

```bash
WORKDIR="/tmp/lana-qa-$(date +%s)"
mkdir -p "${WORKDIR}"

list_subcommands() {
  awk '/^Commands:/{flag=1;next}/^Options:/{flag=0}flag' |
    sed -n 's/^  \([a-z0-9-][a-z0-9-]*\).*/\1/p' |
    grep -Ev '^(help)$' || true
}

discover_paths() {
  local -a prefix=("$@")
  local help
  if [ ${#prefix[@]} -eq 0 ]; then
    help="$(${CLI[@]} --help 2>/dev/null || true)"
  else
    help="$(${CLI[@]} "${prefix[@]}" --help 2>/dev/null || true)"
  fi
  mapfile -t subs < <(printf '%s\n' "${help}" | list_subcommands)
  if [ ${#subs[@]} -eq 0 ]; then
    printf '%s\n' "${prefix[*]}"
    return
  fi
  for sub in "${subs[@]}"; do
    discover_paths "${prefix[@]}" "${sub}"
  done
}

discover_paths | sed '/^$/d' | sort -u > "${WORKDIR}/all-actions.txt"
```

## Use Built-In Command Contracts

Before executing complex commands, read their own help:

```bash
"${CLI[@]}" accounting manual-transaction --help
"${CLI[@]}" custodian create --help
"${CLI[@]}" custodian config-update --help
"${CLI[@]}" domain-config update --help
"${CLI[@]}" accounting account-sets --help
"${CLI[@]}" accounting add-root-node --help
```

`lana-admin-cli --help` and command-level `--help` include examples and accepted values.

## Seed Prerequisites

Create an initial fixture graph and capture IDs for reuse:

1. `prospect create` -> `prospect convert` -> `customer_id`
2. `deposit-account create` + `record-deposit` + `initiate-withdrawal` (captures `withdrawal_id` and often `process_id`)
3. `terms-template create`
4. `credit-facility proposal-create` + `proposal-conclude` + `pending-get` (captures `credit_facility_id` and `collateral_id`)
5. `accounting chart-of-accounts` + `ledger-account --code ...` (captures `ledger_account_id`)
6. `domain-config list`, `report list`, `approval-process list`, `fiscal-year list`, `user roles-list`

## Coverage + Mixed Pass

1. Attempt each leaf action at least once (`all-actions.txt`).
2. Then run randomized mixed actions (`shuf ... | head -n "${RUNS:-80}"`).
3. Prefer non-destructive actions first (`list/get/find` before `close/freeze/delete/archive`).
4. Retry once after auth errors by running `login`.

Record:
- `pass.txt`
- `fail.txt`
- `skip.txt`
- `summary.json`

## Failure Buckets

Classify failures so follow-up work is obvious:

- `schema_mismatch`: command exists in CLI but mutation/field missing in deployed backend
- `missing_dependency`: required IDs/entities not available yet
- `payload_shape`: malformed `--*-json` or file payload
- `precondition`: entity state/lifecycle blocked action (frozen/closed/no active liquidation/etc.)
- `auth`: token/session issues

## Guardrails

- QA only.
- Never stop on first failure; finish full pass and report coverage.
- Leave irreversible actions (`close`, `freeze`, `delete`, `archive`, `deny`) for late in the run.
