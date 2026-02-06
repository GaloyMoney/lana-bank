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

# Models that depend on Sumsub data (to skip when credentials unavailable)
# Staging models
SUMSUB_STAGING_MODELS=(
  "stg_sumsub_applicants"
)

# Intermediate and output models that depend on Sumsub
SUMSUB_DEPENDENT_MODELS=(
  "int_sumsub_applicants"
  "int_customer_identities"
  "int_loan_status_change"
  "int_loan_statements"
  "int_loan_portfolio"
  "int_nrsf_03_01_cliente"
  "report_nrsf_03_01_cliente"
  "int_nrsf_03_03_documentos_clientes"
  "report_nrsf_03_03_documentos_clientes"
  "int_nrsf_03_05_agencias"
  "report_nrsf_03_05_agencias"
  "int_nrsf_03_07_funcionarios_y_empleados"
  "report_nrsf_03_07_funcionarios_y_empleados"
  "int_nrsf_03_08_resumen_de_depositos_garantizados"
  "report_nrsf_03_08_resumen_de_depositos_garantizados"
  "int_nrp_41_01_persona"
  "report_nrp_41_01_persona"
  "report_reporte_de_cambios_de_estado"
  "report_other_estado_de_cuenta_de_prestamo"
  "report_other_reporte_de_cartera_de_prestamos"
)

# Build jq filter array for sumsub staging models
sumsub_staging_jq_array() {
  local arr='['
  local first=true
  for model in "${SUMSUB_STAGING_MODELS[@]}"; do
    if [ "$first" = true ]; then
      first=false
    else
      arr+=','
    fi
    arr+="\"$model\""
  done
  arr+=']'
  echo "$arr"
}

# Build jq filter array for sumsub dependent models
sumsub_dependent_jq_array() {
  local arr='['
  local first=true
  for model in "${SUMSUB_DEPENDENT_MODELS[@]}"; do
    if [ "$first" = true ]; then
      first=false
    else
      arr+=','
    fi
    arr+="\"$model\""
  done
  arr+=']'
  echo "$arr"
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

  # Allow longer timeout for multiple assets (10 min = 300 attempts * 2 sec)
  dagster_poll_run_status "$run_id" 300 2 || return 1

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

@test "dagster: dbt seed" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  echo "=== Running dbt_seeds_job (dbt seed) ==="

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
}

@test "dagster: dbt run staging models" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  echo "=== Materializing dbt staging models ==="

  # Get all dbt_lana_dw assets
  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  # Filter for staging models only (not seeds)
  # Skip sumsub models if credentials are not available
  if has_sumsub_credentials; then
    staging_assets=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] == "staging")]')
  else
    echo "Skipping sumsub staging models (SUMSUB_KEY or SUMSUB_SECRET not set)"
    local skip_models
    skip_models=$(sumsub_staging_jq_array)
    staging_assets=$(echo "$output" | jq -c --argjson skip "$skip_models" '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] == "staging" and (.[-1] | IN($skip[]) | not))]')
  fi

  staging_count=$(echo "$staging_assets" | jq 'length')

  if [ "$staging_count" -eq 0 ]; then
    echo "No dbt staging assets found"
    return 1
  fi

  echo "Found $staging_count dbt staging assets to materialize"

  # Build asset selection for staging models
  if has_sumsub_credentials; then
    run_variables=$(echo "$output" | jq '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] == "staging") | {path: .}]
        },
        runConfigData: {}
      }
    }')
  else
    local skip_models
    skip_models=$(sumsub_staging_jq_array)
    run_variables=$(echo "$output" | jq --argjson skip "$skip_models" '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] == "staging" and (.[-1] | IN($skip[]) | not)) | {path: .}]
        },
        runConfigData: {}
      }
    }')
  fi

  exec_dagster_graphql "launch_run" "$run_variables"
  dagster_check_launch_run_errors || return 1

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch dbt staging models materialization - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched dbt staging models materialization with run ID: $run_id"

  # Wait for staging models to complete (15 min timeout)
  dagster_poll_run_status "$run_id" 450 2 || return 1

  echo "dbt staging models materialized successfully"
}

@test "dagster: dbt run remaining models" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  echo "=== Materializing remaining dbt models ==="

  # Get all dbt_lana_dw assets
  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  # Filter for non-staging models (marts, intermediate, etc.)
  # These depend on staging models which should have been materialized in the previous test
  # Skip sumsub models and all downstream dependents if credentials are not available
  if has_sumsub_credentials; then
    remaining_assets=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "staging" and .[1] != "seeds")]')
  else
    echo "Skipping sumsub models and downstream dependents (SUMSUB_KEY or SUMSUB_SECRET not set)"
    local skip_models
    skip_models=$(sumsub_dependent_jq_array)
    remaining_assets=$(echo "$output" | jq -c --argjson skip "$skip_models" '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "staging" and .[1] != "seeds" and (.[-1] | IN($skip[]) | not))]')
  fi

  remaining_count=$(echo "$remaining_assets" | jq 'length')

  if [ "$remaining_count" -eq 0 ]; then
    echo "No remaining dbt assets found (all were staging/seeds)"
    return 0
  fi

  echo "Found $remaining_count remaining dbt assets to materialize"

  # Build asset selection for remaining models
  if has_sumsub_credentials; then
    run_variables=$(echo "$output" | jq '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "staging" and .[1] != "seeds") | {path: .}]
        },
        runConfigData: {}
      }
    }')
  else
    local skip_models
    skip_models=$(sumsub_dependent_jq_array)
    run_variables=$(echo "$output" | jq --argjson skip "$skip_models" '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "staging" and .[1] != "seeds" and (.[-1] | IN($skip[]) | not)) | {path: .}]
        },
        runConfigData: {}
      }
    }')
  fi

  exec_dagster_graphql "launch_run" "$run_variables"
  dagster_check_launch_run_errors || return 1

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch remaining dbt models materialization - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched remaining dbt models materialization with run ID: $run_id"

  # Wait for remaining models to complete (20 min timeout)
  dagster_poll_run_status "$run_id" 600 2 || return 1

  echo "All dbt models materialized successfully"
}
