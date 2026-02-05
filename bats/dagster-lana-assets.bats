#!/usr/bin/env bats

load helpers

# Helper to check if BigQuery credentials are available
# SA_CREDS_BASE64 is set in data-pipeline CI, not in basic BATS CI
has_bigquery_credentials() {
  [[ -n "${SA_CREDS_BASE64:-}" ]]
}

# All lana source assets to materialize
LANA_ASSETS=(
  "inbox_events"
  "cala_balance_history"
  "cala_account_set_member_account_sets"
  "cala_account_set_member_accounts"
  "cala_account_sets"
  "cala_accounts"
  "core_public_ids"
  "core_chart_events"
  "core_chart_node_events"
  "core_chart_events_rollup"
  "core_collateral_events_rollup"
  "core_credit_facility_events_rollup"
  "core_credit_facility_proposal_events_rollup"
  "core_customer_events_rollup"
  "core_deposit_account_events_rollup"
  "core_deposit_events_rollup"
  "core_disbursal_events_rollup"
  "core_interest_accrual_cycle_events_rollup"
  "core_liquidation_events_rollup"
  "core_obligation_events_rollup"
  "core_payment_allocation_events_rollup"
  "core_payment_events_rollup"
  "core_pending_credit_facility_events_rollup"
  "core_withdrawal_events_rollup"
)

@test "dagster: verify all lana source assets exist" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  local missing_assets=()
  for asset in "${LANA_ASSETS[@]}"; do
    if ! echo "$output" | jq -e --arg asset "$asset" '.data.assetsOrError.nodes[]?.key.path | select(. == ["lana", $asset])' >/dev/null 2>&1; then
      missing_assets+=("$asset")
    fi
  done

  if [ ${#missing_assets[@]} -gt 0 ]; then
    echo "Missing assets:"
    printf '  - %s\n' "${missing_assets[@]}"
    echo ""
    echo "Available lana assets:"
    echo "$output" | jq -r '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "lana") | .[1]' | sort
    return 1
  fi

  echo "All ${#LANA_ASSETS[@]} lana source assets verified to exist"
}

@test "dagster: materialize all lana source assets" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Build asset selection array
  local asset_selection=""
  for asset in "${LANA_ASSETS[@]}"; do
    if [ -n "$asset_selection" ]; then
      asset_selection="${asset_selection},"
    fi
    asset_selection="${asset_selection}{\"path\":[\"lana\",\"${asset}\"]}"
  done

  variables=$(cat <<EOF
{
  "executionParams": {
    "selector": {
      "repositoryLocationName": "Lana DW",
      "repositoryName": "__repository__",
      "jobName": "__ASSET_JOB",
      "assetSelection": [${asset_selection}]
    },
    "runConfigData": {}
  }
}
EOF
)

  exec_dagster_graphql "launch_run" "$variables"
  dagster_check_launch_run_errors || return 1

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch run - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched materialization job for ${#LANA_ASSETS[@]} assets with run ID: $run_id"

  # Allow longer timeout for multiple assets (10 min = 300 attempts * 2 sec)
  dagster_poll_run_status "$run_id" 300 2 || return 1

  echo "All ${#LANA_ASSETS[@]} lana source assets materialized successfully"
}

@test "dagster: verify materializations for all lana source assets" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  local failed_assets=()
  
  for asset in "${LANA_ASSETS[@]}"; do
    asset_vars=$(jq -n --arg asset "$asset" '{
      assetKey: { path: ["lana", $asset] }
    }')
    exec_dagster_graphql "asset_materializations" "$asset_vars"
    
    if ! dagster_validate_json; then
      failed_assets+=("$asset (invalid JSON response)")
      continue
    fi
    
    asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
    if [ "$asset_type" != "Asset" ]; then
      failed_assets+=("$asset (not found)")
      continue
    fi
    
    materialization_count=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations | length')
    if [ "$materialization_count" -eq 0 ]; then
      failed_assets+=("$asset (no materializations)")
      continue
    fi
  done

  if [ ${#failed_assets[@]} -gt 0 ]; then
    echo "Assets with issues:"
    printf '  - %s\n' "${failed_assets[@]}"
    return 1
  fi

  echo "All ${#LANA_ASSETS[@]} lana source assets have successful materializations"
}
