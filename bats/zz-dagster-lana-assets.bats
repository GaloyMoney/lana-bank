#!/usr/bin/env bats

load helpers

# Helper to check if BigQuery credentials are available
# SA_CREDS_BASE64 is set in data-pipeline CI, not in basic BATS CI
has_bigquery_credentials() {
  [[ -n "${SA_CREDS_BASE64:-}" ]]
}

# Helper to check if Sumsub credentials are available
has_sumsub_credentials() {
  [[ -n "${SUMSUB_KEY:-}" && -n "${SUMSUB_SECRET:-}" ]]
}

# Lana source assets
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

# Bitfinex source assets
BITFINEX_ASSETS=(
  "bitfinex_order_book_dlt"
  "bitfinex_ticker_dlt"
  "bitfinex_trades_dlt"
)

# Sumsub source assets
SUMSUB_ASSETS=(
  "sumsub_applicants_dlt"
)

# Helper: verify assets exist for a given group
verify_assets_exist() {
  local group=$1
  shift
  local assets=("$@")

  local missing_assets=()
  for asset in "${assets[@]}"; do
    if ! echo "$output" | jq -e --arg group "$group" --arg asset "$asset" '.data.assetsOrError.nodes[]?.key.path | select(. == [$group, $asset])' >/dev/null 2>&1; then
      missing_assets+=("$asset")
    fi
  done

  if [ ${#missing_assets[@]} -gt 0 ]; then
    echo "Missing assets in group '$group':"
    printf '  - %s\n' "${missing_assets[@]}"
    echo ""
    echo "Available $group assets:"
    echo "$output" | jq -r --arg group "$group" '.data.assetsOrError.nodes[]?.key.path | select(.[0] == $group) | .[1]' | sort
    return 1
  fi

  echo "All ${#assets[@]} $group assets verified to exist"
}

# Helper: build asset selection JSON for a group
build_asset_selection() {
  local group=$1
  shift
  local assets=("$@")

  local selection=""
  for asset in "${assets[@]}"; do
    if [ -n "$selection" ]; then
      selection="${selection},"
    fi
    selection="${selection}{\"path\":[\"${group}\",\"${asset}\"]}"
  done
  echo "$selection"
}

# Helper: verify materializations for a group
verify_materializations() {
  local group=$1
  shift
  local assets=("$@")

  local failed_assets=()

  for asset in "${assets[@]}"; do
    asset_vars=$(jq -n --arg group "$group" --arg asset "$asset" '{
      assetKey: { path: [$group, $asset] }
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
    echo "Assets with issues in group '$group':"
    printf '  - %s\n' "${failed_assets[@]}"
    return 1
  fi

  echo "All ${#assets[@]} $group assets have successful materializations"
}

@test "dagster: verify all source assets exist" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  local failed=0
  local total=$((${#LANA_ASSETS[@]} + ${#BITFINEX_ASSETS[@]}))

  verify_assets_exist "lana" "${LANA_ASSETS[@]}" || failed=1
  verify_assets_exist "bitfinex" "${BITFINEX_ASSETS[@]}" || failed=1

  if has_sumsub_credentials; then
    verify_assets_exist "sumsub" "${SUMSUB_ASSETS[@]}" || failed=1
    total=$((total + ${#SUMSUB_ASSETS[@]}))
  else
    echo "Skipping sumsub assets verification (SUMSUB_KEY or SUMSUB_SECRET not set)"
  fi

  [ $failed -eq 0 ] || return 1

  echo "All $total source assets verified to exist"
}

@test "dagster: materialize all source assets" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Build combined asset selection from all groups
  local lana_selection=$(build_asset_selection "lana" "${LANA_ASSETS[@]}")
  local bitfinex_selection=$(build_asset_selection "bitfinex" "${BITFINEX_ASSETS[@]}")

  local asset_selection="${lana_selection},${bitfinex_selection}"
  local total=$((${#LANA_ASSETS[@]} + ${#BITFINEX_ASSETS[@]}))

  if has_sumsub_credentials; then
    local sumsub_selection=$(build_asset_selection "sumsub" "${SUMSUB_ASSETS[@]}")
    asset_selection="${asset_selection},${sumsub_selection}"
    total=$((total + ${#SUMSUB_ASSETS[@]}))
  else
    echo "Skipping sumsub assets materialization (SUMSUB_KEY or SUMSUB_SECRET not set)"
  fi

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

  echo "Launched materialization job for $total assets with run ID: $run_id"

  # Allow longer timeout for multiple assets (20 min = 600 attempts * 2 sec)
  dagster_poll_run_status "$run_id" 600 2 || return 1

  echo "All $total source assets materialized successfully"
}

@test "dagster: verify materializations for all source assets" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  local failed=0
  local total=$((${#LANA_ASSETS[@]} + ${#BITFINEX_ASSETS[@]}))

  verify_materializations "lana" "${LANA_ASSETS[@]}" || failed=1
  verify_materializations "bitfinex" "${BITFINEX_ASSETS[@]}" || failed=1

  if has_sumsub_credentials; then
    verify_materializations "sumsub" "${SUMSUB_ASSETS[@]}" || failed=1
    total=$((total + ${#SUMSUB_ASSETS[@]}))
  else
    echo "Skipping sumsub assets verification (SUMSUB_KEY or SUMSUB_SECRET not set)"
  fi

  [ $failed -eq 0 ] || return 1

  echo "All $total source assets have successful materializations"
}

@test "dagster: run dbt seed then dbt run" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Step 1: Run dbt_seeds_job (equivalent to 'dbt seed')
  echo "=== Step 1: Running dbt_seeds_job (dbt seed) ==="

  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "dbt_seeds_job"
      },
      runConfigData: {}
    }
  }')

  exec_dagster_graphql "launch_run" "$variables"
  dagster_check_launch_run_errors || return 1

  seed_run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$seed_run_id" ]; then
    echo "Failed to launch dbt_seeds_job - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched dbt_seeds_job with run ID: $seed_run_id"

  # Wait for seeds to complete (20 min timeout)
  dagster_poll_run_status "$seed_run_id" 600 2 || return 1

  echo "dbt_seeds_job completed successfully"

  # Step 2: Materialize all dbt model assets (equivalent to 'dbt run')
  echo ""
  echo "=== Step 2: Materializing dbt models (dbt run) ==="

  # Get all dbt_lana_dw assets dynamically
  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  # Build asset selection for all dbt_lana_dw assets
  dbt_asset_selection=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw")] | map({path: .}) | @json' | jq -r '.')

  if [ "$dbt_asset_selection" = "[]" ] || [ -z "$dbt_asset_selection" ]; then
    echo "No dbt_lana_dw assets found"
    return 1
  fi

  dbt_asset_count=$(echo "$output" | jq '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw")] | length')
  echo "Found $dbt_asset_count dbt_lana_dw assets to materialize"

  # Launch materialization for all dbt models
  run_variables=$(echo "$output" | jq '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB",
        assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw") | {path: .}]
      },
      runConfigData: {}
    }
  }')

  exec_dagster_graphql "launch_run" "$run_variables"
  dagster_check_launch_run_errors || return 1

  dbt_run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$dbt_run_id" ]; then
    echo "Failed to launch dbt models materialization - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched dbt models materialization with run ID: $dbt_run_id"

  # Wait for dbt run to complete (30 min timeout for all models)
  dagster_poll_run_status "$dbt_run_id" 900 2 || return 1

  echo ""
  echo "=== dbt seed + dbt run completed successfully ==="
  echo "Seeds run ID: $seed_run_id"
  echo "Models run ID: $dbt_run_id"
}
