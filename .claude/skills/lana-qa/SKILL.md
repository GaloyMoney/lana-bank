---
name: lana-qa
description: Run deterministic staging/QA testing with a prebuilt lana-admin using declarative YAML workflows for dependency sequencing and state capture.
---

# Lana Admin QA/Staging Workflows

Goal: run repeatable exploratory coverage with `lana-admin`, with workflow sequencing defined in YAML instead of ad-hoc shell logic.

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
WORKDIR="/tmp/lana-qa-$(date +%s)"
mkdir -p "${WORKDIR}"
```

Default target is `staging`. To target QA, set `export LANA_ENV=qa`.

If staging returns HTTP `500`, assume deployment is in progress. Wait `10-20 minutes`, then retry.

## Login Once

```bash
"${CLI[@]}" auth login --username "${LANA_USERNAME}"
"${CLI[@]}" auth info --json
```

The `auth login` command reads `LANA_ADMIN_URL`, `LANA_KEYCLOAK_URL`, `LANA_KEYCLOAK_CLIENT_ID`, and `LANA_USERNAME` from the environment, saves the session locally, and all subsequent commands should omit auth flags.

## Workflow Source Of Truth

Use YAML files under `workflows/`:

- `workflows/seed_customer_credit_facility.yaml`

Treat YAML as canonical for:

- step order and dependency edges (`depends_on`)
- command path and argument templates
- state capture (`capture` jq paths)
- failure handling policy (`best_effort`, `continue_on_failure`)

To inspect prerequisites for a step:

```bash
./skills/lana-qa/workflow-step-deps.sh --step credit_facility_partial_payment_record
```

## Discover Commands From `--help`

```bash
crawl_help() {
  local prefix=("$@")
  local help_text
  help_text="$("${CLI[@]}" "${prefix[@]}" --help)"
  local subs
  subs="$(printf '%s\n' "${help_text}" | awk '
    /^Commands:/ {in_cmd=1; next}
    /^Options:/ {in_cmd=0}
    in_cmd && $1 != "help" && NF > 0 {print $1}
  ')"

  if [[ -z "${subs}" ]]; then
    printf '%s\n' "${prefix[*]}"
    return
  fi

  while IFS= read -r sub; do
    [[ -z "${sub}" ]] && continue
    crawl_help "${prefix[@]}" "${sub}"
  done <<< "${subs}"
}

crawl_help | sed '/^$/d' | sort -u > "${WORKDIR}/all-actions.txt"
```

Use `<action> --help` as source of truth for:

- required args
- defaults
- accepted enum values
- payload format hints (`--*-json`, `--file`, etc.)

Current namespace examples:

- `iam role list`
- `iam user create`
- `deposit account create`
- `deposit withdrawal initiate`
- `credit facility proposal-create`
- `credit collateral update`
- `credit loan-agreement generate`

## Workflow Execution Contract

For each workflow:

1. Resolve template vars (`${LANA_ENV}`, `${TS}`, state references).
2. Execute only when `depends_on` are satisfied.
3. Run commands with `--json`.
4. Save raw output under `${WORKDIR}`.
5. Extract/capture IDs into workflow state using jq expressions from YAML.
6. Continue according to workflow policy.

Prefer running independent steps in parallel when they have no dependency edge.

## Failure Buckets

Classify failures as:

- `missing_dependency`
- `payload_shape`
- `precondition`
- `auth`
- `output_contract_mismatch`
- `permission_boundary`
- `environment_drift`
- `unknown`

## Known Environment Notes

- Supports `staging` (default) and `qa`.
- Staging may return transient HTTP `500` during active deployment windows. Wait `10-20 minutes` and retry.
- For GraphQL `Date` inputs such as `credit collateral update --effective`, prefer `YYYY-MM-DD`.

## Guardrails

- Do not execute commands absent from the discovered help tree.
- Keep execution dependency-ordered via `depends_on`.
- Preserve artifacts: action lists, pass/fail summaries, state snapshots, and per-step outputs.
