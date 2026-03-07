#!/usr/bin/env bash
set -euo pipefail

DEFAULT_WORKFLOW=".claude/skills/lana-qa/workflows/seed_customer_credit_facility.yaml"

usage() {
  cat <<'USAGE'
Usage:
  dev/bin/workflow-step-deps.sh --step <STEP_ID> [--workflow <FILE>] [--all]

Description:
  Prints dependency ancestry for a workflow step as an ordered list of steps.
  By default, only mutation steps are shown (read-only steps are filtered out).

Options:
  --step <STEP_ID>     Target workflow step id (required)
  --workflow <FILE>    Workflow YAML path (default: .claude/skills/lana-qa/workflows/seed_customer_credit_facility.yaml)
  --all                Include read-only steps in output
  -h, --help           Show this help

Examples:
  dev/bin/workflow-step-deps.sh --step credit_facility_partial_payment_record
  dev/bin/workflow-step-deps.sh --step deposit_close --all
USAGE
}

workflow="$DEFAULT_WORKFLOW"
step_id=""
include_all="false"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --workflow)
      workflow="${2:-}"
      shift 2
      ;;
    --step)
      step_id="${2:-}"
      shift 2
      ;;
    --all)
      include_all="true"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 1
      ;;
  esac
done

if [[ -z "$step_id" ]]; then
  echo "--step is required" >&2
  usage >&2
  exit 1
fi

if ! command -v yq >/dev/null 2>&1; then
  echo "yq is required but not found in PATH" >&2
  exit 1
fi

if [[ ! -f "$workflow" ]]; then
  echo "Workflow file not found: $workflow" >&2
  exit 1
fi

mapfile -t ordered_ids < <(yq -r '.steps[].id' "$workflow")

declare -A exists
for id in "${ordered_ids[@]}"; do
  exists["$id"]="1"
done

if [[ -z "${exists[$step_id]:-}" ]]; then
  echo "Step not found: $step_id" >&2
  echo "Available step ids:" >&2
  printf '  - %s\n' "${ordered_ids[@]}" >&2
  exit 1
fi

declare -A step_cmd
declare -A step_deps
declare -A needed
declare -A visiting

for id in "${ordered_ids[@]}"; do
  cmd="$(yq -r ".steps[] | select(.id == \"$id\") | .command" "$workflow")"
  step_cmd["$id"]="$cmd"
  mapfile -t deps < <(yq -r ".steps[] | select(.id == \"$id\") | (.depends_on // [])[]" "$workflow")
  step_deps["$id"]="${deps[*]:-}"
done

collect() {
  local id="$1"
  if [[ -n "${needed[$id]:-}" ]]; then
    return
  fi
  if [[ -n "${visiting[$id]:-}" ]]; then
    echo "Cycle detected at step: $id" >&2
    exit 1
  fi
  visiting["$id"]="1"
  local dep
  for dep in ${step_deps[$id]:-}; do
    if [[ -z "${exists[$dep]:-}" ]]; then
      echo "Unknown dependency '$dep' referenced by step '$id'" >&2
      exit 1
    fi
    collect "$dep"
  done
  unset visiting["$id"]
  needed["$id"]="1"
}

is_mutation_command() {
  local cmd="$1"
  local leaf="${cmd##* }"

  case "$leaf" in
    list|get|find|get-by-email|proposal-get|proposal-list|pending-get|download-link|account-entry|\
    chart-of-accounts|base-config|credit-config|deposit-config|account-sets|ledger-account|\
    balance-sheet|trial-balance|profit-and-loss|version|info)
      return 1
      ;;
    *)
      return 0
      ;;
  esac
}

collect "$step_id"

echo "workflow: $workflow"
echo "target-step: $step_id"
echo "required-steps:"

count=0
for id in "${ordered_ids[@]}"; do
  [[ -n "${needed[$id]:-}" ]] || continue

  cmd="${step_cmd[$id]}"
  if [[ "$include_all" == "false" ]]; then
    is_mutation_command "$cmd" || continue
  fi

  count=$((count + 1))
  printf "  %2d. %s -> %s\n" "$count" "$id" "$cmd"
done

if [[ "$count" -eq 0 ]]; then
  echo "  (no matching steps after filtering)"
fi
