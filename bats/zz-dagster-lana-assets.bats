#!/usr/bin/env bats

load helpers

# Helper to check if BigQuery credentials are available
# SA_CREDS_BASE64 is set in data-pipeline CI, not in basic BATS CI
has_bigquery_credentials() {
  [[ -n "${SA_CREDS_BASE64:-}" ]]
}

# Helper to check if Sumsub credentials are available
has_sumsub_credentials() {
  [[ -n "${LANA_DOMAIN_CONFIG_SUMSUB_API_KEY:-}" && -n "${LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET:-}" ]]
}

# All sumsub-dependent models (staging + intermediate + output)
SUMSUB_ALL_MODELS=(
  "stg_sumsub_applicants"
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

# Build jq filter array for sumsub models
sumsub_all_jq_array() {
  local arr='['
  local first=true
  for model in "${SUMSUB_ALL_MODELS[@]}"; do
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
  "cala_cumulative_effective_balances"
  "core_public_ids"
  "core_chart_events"
  "core_chart_node_events"
  "core_chart_events_rollup"
  "core_credit_facility_events_rollup"
  "core_credit_facility_proposal_events_rollup"
  "core_customer_events_rollup"
  "core_party_events_rollup"
  "core_deposit_account_events_rollup"
  "core_deposit_events_rollup"
  "core_disbursal_events_rollup"
  "core_interest_accrual_cycle_events_rollup"
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
    echo "Skipping sumsub assets materialization (LANA_DOMAIN_CONFIG_SUMSUB_API_KEY or LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET not set)"
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

@test "dagster: dbt run all models" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  echo "=== Materializing all dbt models ==="

  # Get all dbt_lana_dw assets
  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  # Filter for all non-seed dbt models
  # Skip sumsub models if credentials are not available
  if has_sumsub_credentials; then
    dbt_assets=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "seeds")]')
  else
    echo "Skipping sumsub models and downstream dependents (LANA_DOMAIN_CONFIG_SUMSUB_API_KEY or LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET not set)"
    local skip_models
    skip_models=$(sumsub_all_jq_array)
    dbt_assets=$(echo "$output" | jq -c --argjson skip "$skip_models" '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "seeds" and (.[-1] | IN($skip[]) | not))]')
  fi

  dbt_count=$(echo "$dbt_assets" | jq 'length')

  if [ "$dbt_count" -eq 0 ]; then
    echo "No dbt assets found"
    return 1
  fi

  echo "Found $dbt_count dbt assets to materialize"

  # Build asset selection for all non-seed dbt models
  # Dagster respects the DAG and will execute in dependency order
  if has_sumsub_credentials; then
    run_variables=$(echo "$output" | jq '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "seeds") | {path: .}]
        },
        runConfigData: {}
      }
    }')
  else
    local skip_models
    skip_models=$(sumsub_all_jq_array)
    run_variables=$(echo "$output" | jq --argjson skip "$skip_models" '{
      executionParams: {
        selector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          jobName: "__ASSET_JOB",
          assetSelection: [.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[1] != "seeds" and (.[-1] | IN($skip[]) | not)) | {path: .}]
        },
        runConfigData: {}
      }
    }')
  fi

  exec_dagster_graphql "launch_run" "$run_variables"
  dagster_check_launch_run_errors || return 1

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch dbt models materialization - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched dbt models materialization with run ID: $run_id"

  # Wait for all models to complete (30 min timeout)
  dagster_poll_run_status "$run_id" 900 2 || return 1

  echo "All $dbt_count dbt models materialized successfully"
}
