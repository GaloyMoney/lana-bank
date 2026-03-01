---
name: lana-qa
description: Run broad exploratory testing on QA with a prebuilt lana-admin-cli by enumerating all available commands and executing coverage plus randomized mixed actions across customers, loans, and related domains.
---

# Lana Admin CLI QA Exploration

Goal: run broad QA coverage using `lana-admin-cli` only, with deterministic fixture chaining and machine-readable command contracts.

## Setup

```bash
export LANA_USERNAME="galoysuperuser@mailinator.com"
export LANA_ADMIN_URL="https://admin.qa.lana.galoy.io/graphql"
export LANA_KEYCLOAK_URL="https://auth.qa.lana.galoy.io"
export LANA_KEYCLOAK_CLIENT_ID="${LANA_KEYCLOAK_CLIENT_ID:-admin-panel}"

CLI_BIN="./skills/lana-qa/lana-admin-cli"
if [[ ! -x "${CLI_BIN}" ]]; then
  CLI_BIN="./lana-admin-cli"
fi
[[ -x "${CLI_BIN}" ]] || {
  echo "Missing lana-admin-cli at ./skills/lana-qa/lana-admin-cli or ./lana-admin-cli" >&2
  exit 1
}

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

## Discover Commands From `spec`

```bash
WORKDIR="/tmp/lana-qa-$(date +%s)"
mkdir -p "${WORKDIR}"
"${CLI[@]}" spec > "${WORKDIR}/spec.json"

jq -r '
  def walk:
    . as $c
    | if ((.subcommands // []) | length) == 0
      then $c.path | sub("^lana-admin-cli "; "")
      else (.subcommands // [])[] | walk
      end;
  .root | walk
' "${WORKDIR}/spec.json" | sort -u > "${WORKDIR}/all-actions.txt"
```

The `spec` output is the source of truth for:
- required args
- enum/format hints
- mutating vs read-only
- lifecycle phase
- preconditions
- output ID fields

## ID Normalization Helpers

Normalize any entity-ref style IDs before using UUID args:

```bash
normalize_id() {
  # Example: FiscalYear:019c... -> 019c...
  sed -E 's/^[A-Za-z]+://'
}

is_uuid() {
  [[ "$1" =~ ^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$ ]]
}
```

Use pattern:

```bash
FY_RAW="$(jq -r '.[0].fiscalYearId // .[0].id // empty' <<<"$FISCAL_YEARS_JSON")"
FY_ID="$(printf '%s' "$FY_RAW" | normalize_id)"
if is_uuid "$FY_ID"; then
  "${CLI[@]}" fiscal-year close --fiscal-year-id "$FY_ID" --json
else
  echo "SKIP id_normalization fiscal-year close: invalid UUID from '$FY_RAW'" >> "${WORKDIR}/skip.txt"
fi
```

## Canonical Fixture Extraction Map

Use these JSON fields exactly when chaining fixtures:

- `prospect create` -> `.prospectId`
- `prospect convert` -> `.customerId`
- `deposit-account create` -> `.depositAccountId`
- `deposit-account initiate-withdrawal` -> `.withdrawalId`, `.approvalProcessId`
- `terms-template create` -> `.termsId`
- `credit-facility proposal-create` -> `.creditFacilityProposalId`
- `credit-facility pending-get` -> `.collateralId`
- `credit-facility find` -> `.creditFacilityId`, `.collateralId`
- `document attach` -> `.documentId`
- `csv-export create-ledger-csv` -> `.documentId`
- `report trigger` -> `.runId`
- `fiscal-year list` -> `.[] .fiscalYearId` (fallback `.[] .id`, then normalize)

## Deterministic Execution Stages

Run in stages to reduce false negatives:

1. `read_only`
- all `list/get/find/...` style actions first

2. `seed_or_setup_mutation`
- create foundational entities:
  - prospect -> customer
  - deposit account
  - terms template
  - credit-facility proposal
  - document attach

3. `stateful_mutation`
- run transitions that require prior state:
  - proposal conclude
  - withdrawal lifecycle
  - disbursal/payment/liquidation transitions

4. `destructive_end_state`
- only at the end:
  - close/freeze/archive/delete/revert/cancel actions

The lifecycle phase is available directly in `spec.json` under each command.

## Recommended Coverage Driver

```bash
jq -r '
  def walk:
    . as $c
    | if ((.subcommands // []) | length) == 0 then [$c]
      else [(.subcommands // [])[] | walk[]]
      end;
  .root | walk[]
  | [.lifecycle_phase, (.path | sub("^lana-admin-cli "; ""))]
  | @tsv
' "${WORKDIR}/spec.json" \
| sort -k1,1 -k2,2 > "${WORKDIR}/ordered-actions.tsv"
```

## Failure Buckets

Classify each non-pass with one of:

- `missing_dependency`
- `payload_shape`
- `precondition`
- `auth`
- `output_contract_mismatch`
- `id_format_contract`
- `permission_boundary`
- `environment_drift`
- `unknown`

## Known QA Drift (timestamped)

- `2026-03-01`: fiscal-year APIs may expose prefixed entity references in some fields (for example `FiscalYear:<uuid>`). Mutations expecting UUID must use normalized raw UUID values.

## Guardrails

- QA only.
- Use `spec` metadata instead of parsing plain help text.
- Never stop on first failure; finish full run and produce artifacts.
- Do not add commands that are absent from `spec` for this binary.
