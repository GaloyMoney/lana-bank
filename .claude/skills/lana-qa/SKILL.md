---
name: lana-qa
description: Run broad exploratory testing on QA with a prebuilt lana-admin-cli by enumerating all available commands and executing coverage plus randomized mixed actions across customers, loans, and related domains.
---

# Lana Admin CLI QA Exploration

Use this skill to stress admin behavior through `lana-admin-cli` without writing one-off GraphQL calls.

## Goal

1. Discover all executable CLI actions from help output.
2. Execute every action at least once when prerequisites are satisfiable.
3. Run additional randomized mixed workflows (customer + loan + collateral + account + report flows).

## Assumptions

- Run commands from the deployed skill directory:
  - `~/.openclaw/${OPEN_CLAW_AGENT}/skills/lana-qa`
- A built CLI binary is already present in that directory as `./lana-admin-cli`.
- `bash`, `curl`, `jq`, `awk`, and `sed` are available.
- You can use the superuser email account for the target environment.

## QA Environment

Use QA only:

```bash
export LANA_USERNAME="galoysuperuser@mailinator.com"
export LANA_ADMIN_URL="https://admin.qa.lana.galoy.io/graphql"
export LANA_KEYCLOAK_URL="https://auth.qa.lana.galoy.io"
export LANA_KEYCLOAK_CLIENT_ID="${LANA_KEYCLOAK_CLIENT_ID:-admin-panel}"
```

Binary path:

```bash
CLI_BIN="./lana-admin-cli"
[[ -x "${CLI_BIN}" ]] || { echo "Missing built CLI binary in current skill directory" >&2; exit 1; }
```

Base command:

```bash
CLI=(
  "${CLI_BIN}"
  --admin-url "${LANA_ADMIN_URL}"
  --keycloak-url "${LANA_KEYCLOAK_URL}"
  --keycloak-client-id "${LANA_KEYCLOAK_CLIENT_ID}"
  --username "${LANA_USERNAME}"
)
```

## Authentication

First try normal login:

```bash
"${CLI[@]}" login
```

- QA uses passwordless Keycloak PKCE. `lana-admin-cli login` supports browserless bot login automatically.

Validate session:

```bash
"${CLI[@]}" customer list --first 1 --json >/dev/null
```

Session cache is persisted under `~/.config` by the CLI.

## 1) Discover Full Command Catalog

Enumerate command paths from help output (excluding `help`):

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

  local sub
  for sub in "${subs[@]}"; do
    discover_paths "${prefix[@]}" "${sub}"
  done
}

discover_paths | sed '/^$/d' | sort -u > "${WORKDIR}/all-actions.txt"
```

This produces every leaf action path (for example `prospect create`, `customer get`, `credit-facility proposal-create`, etc).

## 2) Build Stateful Test Data Pool

Create reusable entities so later actions have IDs to operate on.

```bash
STATE_JSON="${WORKDIR}/state.json"
jq -n '{prospect_ids:[],customer_ids:[],customer_public_ids:[],credit_facility_ids:[],credit_facility_public_ids:[],collateral_ids:[],deposit_account_ids:[]}' > "${STATE_JSON}"

TS="$(date +%s)"
EMAIL="cli-qa-${TS}@mailinator.com"
TG="cliqa_${TS}"

PROSPECT_JSON="$("${CLI[@]}" prospect create --email "${EMAIL}" --telegram-handle "${TG}" --json)"
PROSPECT_ID="$(jq -r '.prospectId // .id' <<<"${PROSPECT_JSON}")"
jq --arg v "${PROSPECT_ID}" '.prospect_ids += [$v]' "${STATE_JSON}" > "${STATE_JSON}.tmp" && mv "${STATE_JSON}.tmp" "${STATE_JSON}"

CUSTOMER_JSON="$("${CLI[@]}" prospect convert --prospect-id "${PROSPECT_ID}" --json)"
CUSTOMER_ID="$(jq -r '.customerId // .id' <<<"${CUSTOMER_JSON}")"
CUSTOMER_PUBLIC_ID="$(jq -r '.publicId // empty' <<<"${CUSTOMER_JSON}")"
jq --arg id "${CUSTOMER_ID}" --arg pid "${CUSTOMER_PUBLIC_ID}" '.customer_ids += [$id] | .customer_public_ids += [$pid]' "${STATE_JSON}" > "${STATE_JSON}.tmp" && mv "${STATE_JSON}.tmp" "${STATE_JSON}"

"${CLI[@]}" deposit-account create --customer-id "${CUSTOMER_ID}" --json >/dev/null || true
```

If credit-facility commands require a facility, create one proposal and conclude it, then store facility/collateral IDs in `state.json`.

## 3) Coverage Pass (Attempt Every Action Once)

Run all actions in random order, but each at least once.

Rules:
- Always include `--json` when supported.
- Skip only when required args cannot be satisfied from state and cannot be safely generated.
- Record each action as `PASS`, `FAIL`, or `SKIP` with reason.

Create results files:

```bash
: > "${WORKDIR}/pass.txt"
: > "${WORKDIR}/fail.txt"
: > "${WORKDIR}/skip.txt"
```

Argument resolution policy (apply in this order):

1. Parse required flags from `--help` output for the action.
2. Fill known IDs from `state.json` by flag name:
   - `--prospect-id` -> `.prospect_ids[-1]`
   - `--customer-id` -> `.customer_ids[-1]`
   - `--credit-facility-id` or `--id` in credit-facility context -> `.credit_facility_ids[-1]`
   - `--collateral-id` -> `.collateral_ids[-1]`
3. Fill simple generated values:
   - `--email` -> random mailinator email
   - `--telegram-handle` -> random handle
   - numeric amounts/rates -> use small safe defaults
4. If still missing required args, mark `SKIP missing-prerequisite`.

After any successful mutating command that returns a new ID, append that ID to `state.json` for future commands.

## 4) Randomized Mixed Pass

After coverage, run a soak pass with mixed actions:

```bash
RUNS="${RUNS:-80}"
shuf "${WORKDIR}/all-actions.txt" | head -n "${RUNS}" > "${WORKDIR}/random-actions.txt"
```

Execution strategy:
- Weight toward stateful business flows (customers, deposit accounts, credit facilities, collateral, withdrawals, loan agreements, reports).
- Reuse and mutate existing entities (do not only create new ones).
- Interleave read/write actions to verify state transitions.
- On auth failure (`401`), run `"${CLI[@]}" login` and retry once.

## 5) Completion Report

Output a machine-readable summary:

```bash
jq -n \
  --arg env "qa" \
  --arg workdir "${WORKDIR}" \
  --arg total "$(wc -l < "${WORKDIR}/all-actions.txt" | tr -d ' ')" \
  --arg pass "$(wc -l < "${WORKDIR}/pass.txt" | tr -d ' ')" \
  --arg fail "$(wc -l < "${WORKDIR}/fail.txt" | tr -d ' ')" \
  --arg skip "$(wc -l < "${WORKDIR}/skip.txt" | tr -d ' ')" \
  '{target_env:$env, catalog_actions:($total|tonumber), pass:($pass|tonumber), fail:($fail|tonumber), skip:($skip|tonumber), artifact_dir:$workdir}'
```

Also print high-value IDs gathered during the run:
- newest `customer_public_id`
- newest `credit_facility_public_id`

## Guardrails

- This skill is QA-only.
- If an action can lock or close entities irreversibly, execute it only after enough read/update coverage has already been collected.
- Never stop on first failure; continue and report complete coverage gaps.
