---
name: lana-qa
description: Run exploratory staging/QA testing with a prebuilt lana-admin using declarative YAML workflows for dependency sequencing and state capture.
---

# Lana Admin QA/Staging Workflows

Goal: run deterministic exploratory coverage using `lana-admin`, with workflow sequencing defined in YAML (not ad-hoc shell flow logic).

## Setup

```bash
export LANA_USERNAME="galoysuperuser@mailinator.com"
export LANA_ENV="${LANA_ENV:-staging}"
export LANA_KEYCLOAK_CLIENT_ID="${LANA_KEYCLOAK_CLIENT_ID:-admin-panel}"

case "${LANA_ENV}" in
  staging)
    export LANA_ADMIN_URL="https://admin.staging.lana.galoy.io/graphql"
    export LANA_KEYCLOAK_URL="https://auth.staging.lana.galoy.io"
    ;;
  qa)
    export LANA_ADMIN_URL="https://admin.qa.lana.galoy.io/graphql"
    export LANA_KEYCLOAK_URL="https://auth.qa.lana.galoy.io"
    ;;
  *)
    echo "Unsupported LANA_ENV='${LANA_ENV}'. Expected 'staging' or 'qa'." >&2
    exit 1
    ;;
esac

CLI_BIN="./skills/lana-qa/lana-admin"
if [[ ! -x "${CLI_BIN}" ]]; then
  CLI_BIN="./lana-admin"
fi
[[ -x "${CLI_BIN}" ]] || {
  echo "Missing lana-admin at ./skills/lana-qa/lana-admin or ./lana-admin" >&2
  exit 1
}

CLI=("${CLI_BIN}")
WORKDIR="/tmp/lana-qa-$(date +%s)"
mkdir -p "${WORKDIR}"
```

By default this targets `staging`. To target QA, set `export LANA_ENV=qa`.

If staging returns HTTP `500`, assume deployment is in progress. Wait `10-20 minutes`, then retry.

## Login Once

```bash
"${CLI[@]}" login \
  --admin-url "${LANA_ADMIN_URL}" \
  --keycloak-url "${LANA_KEYCLOAK_URL}" \
  --keycloak-client-id "${LANA_KEYCLOAK_CLIENT_ID}" \
  --username "${LANA_USERNAME}"
```

All subsequent non-login commands should omit auth/endpoint flags and rely on saved session profile.

## Workflow Source Of Truth

Use the YAML files under `workflows/`:

- `workflows/seed_customer_credit_facility.yaml`

Treat YAML as canonical for:

- step order and dependency edges (`depends_on`)
- command path and argument templates
- state capture (`capture` jq paths)
- failure handling policy (`best_effort`, `continue_on_failure`)

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

Use `<action> --help` as the source of truth for:

- required args
- defaults
- accepted enum values
- payload format hints (`--*-json`, `--file`, etc.)

## Workflow Execution Contract

For each workflow:

1. resolve template vars (`${LANA_ENV}`, `${TS}`, state references)
2. execute only when `depends_on` are satisfied
3. run commands with `--json`
4. save raw output under `${WORKDIR}`
5. extract/capture IDs into workflow state using jq expressions from YAML
6. resolve missing flags from the target command `--help` output
7. continue according to workflow policy (do not abort full run on first failure unless workflow says so)

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
- For GraphQL `Date` inputs (for example `collateral update --effective`), prefer `YYYY-MM-DD`.

## Guardrails

- Do not execute commands absent from discovered help tree (`all-actions.txt`).
- Keep execution dependency-ordered via `depends_on`.
- Preserve all artifacts (action lists, pass/fail summaries, state snapshot, per-step outputs).
