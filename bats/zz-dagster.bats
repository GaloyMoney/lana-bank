#!/usr/bin/env bats

load helpers

# Helper to check if BigQuery credentials are available
# SA_CREDS_BASE64 is set in data-pipeline CI, not in basic BATS CI
has_bigquery_credentials() {
  [[ -n "${SA_CREDS_BASE64:-}" ]]
}

@test "dagster: graphql endpoint responds to POST" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi

  exec_dagster_graphql_status "introspection"
  [ "$status" -eq 0 ]
  [ "$output" = "200" ]
}

@test "dagster: list assets and verify iris_dataset_size exists" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  exec_dagster_graphql "assets"
  if ! echo "$output" | jq -e '.data.assetsOrError.nodes[]?.key.path | select(. == ["iris_dataset_size"])' >/dev/null; then
    status=$?
    if [ "$status" -eq 4 ]; then
      echo "Dagster GraphQL response was not valid JSON"
    else
      echo "iris_dataset_size asset not found in Dagster assets response"
    fi
    echo "$output"
    return 1
  fi
}

@test "dagster: materialize iris_dataset_size and wait for success" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  # Launch materialization targeting only iris_dataset_size asset
  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB"
      },
      runConfigData: {},
      stepKeys: ["iris_dataset_size"]
    }
  }')

  exec_dagster_graphql "launch_run" "$variables"
  dagster_validate_json

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  [ -n "$run_id" ] || { echo "$output"; return 1; }

  dagster_poll_run_status "$run_id" 10 30 || return 1

  asset_vars=$(jq -n '{
    assetKey: { path: ["iris_dataset_size"] }
  }')
  exec_dagster_graphql "asset_materializations" "$asset_vars"

  asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
  [ "$asset_type" = "Asset" ] || { echo "Asset not found: $output"; return 1; }

  recent_run_id=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations[0].runId // empty')
  [ "$recent_run_id" = "$run_id" ] || { echo "Expected run ID $run_id, got $recent_run_id"; return 1; }
}

@test "dagster: materialize core_withdrawal_events_rollup" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB",
        assetSelection: [
          { path: ["lana", "core_withdrawal_events_rollup"] }
        ]
      },
      runConfigData: {}
    }
  }')

  exec_dagster_graphql "launch_run" "$variables"

  dagster_check_launch_run_errors || return 1

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch run - no runId returned"
    echo "Response: $output"
    return 1
  fi

  dagster_poll_run_status "$run_id" 10 30 || return 1

  asset_vars=$(jq -n '{
    assetKey: { path: ["lana", "core_withdrawal_events_rollup"] }
  }')
  exec_dagster_graphql "asset_materializations" "$asset_vars"

  dagster_validate_json || return 1

  asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
  [ "$asset_type" = "Asset" ] || { echo "Asset core_withdrawal_events_rollup not found: $output"; return 1; }

  recent_run_id=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations[0].runId // empty')
  [ "$recent_run_id" = "$run_id" ] || { echo "Expected run ID $run_id for core_withdrawal_events_rollup, got $recent_run_id"; return 1; }
}

@test "dagster: verify dbt asset automatically starts when upstream completes" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Track which sensor we started so we can stop it later
  local active_sensor_name=""

  # Helper function to stop the sensor (called at end of test)
  stop_automation_sensor() {
    if [ -n "$active_sensor_name" ]; then
      echo "Stopping sensor: $active_sensor_name"
      local stop_vars=$(jq -n --arg name "$active_sensor_name" '{
        sensorSelector: {
          repositoryLocationName: "Lana DW",
          repositoryName: "__repository__",
          sensorName: $name
        }
      }')
      exec_dagster_graphql "stop_sensor" "$stop_vars"
      if dagster_validate_json; then
        local stop_status=$(echo "$output" | jq -r '.data.stopSensor.__typename // empty')
        if [ "$stop_status" = "Sensor" ]; then
          echo "Successfully stopped sensor: $active_sensor_name"
        else
          echo "Warning: Failed to stop sensor: $stop_status"
        fi
      fi
    fi
  }

  sensor_vars=$(jq -n '{
    sensorSelector: {
      repositoryLocationName: "Lana DW",
      repositoryName: "__repository__",
      sensorName: "dbt_automation_condition_sensor"
    }
  }')
  exec_dagster_graphql "start_sensor" "$sensor_vars"
  dagster_validate_json || return 1

  sensor_status=$(echo "$output" | jq -r '.data.startSensor.__typename // empty')
  if [ "$sensor_status" = "SensorNotFoundError" ]; then
    echo "dbt_automation_condition_sensor not found, trying default_automation_condition_sensor"
    sensor_vars=$(jq -n '{
      sensorSelector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        sensorName: "default_automation_condition_sensor"
      }
    }')
    exec_dagster_graphql "start_sensor" "$sensor_vars"
    dagster_validate_json || return 1
    sensor_status=$(echo "$output" | jq -r '.data.startSensor.__typename // empty')
    if [ "$sensor_status" = "Sensor" ]; then
      active_sensor_name="default_automation_condition_sensor"
    fi
  else
    if [ "$sensor_status" = "Sensor" ]; then
      active_sensor_name="dbt_automation_condition_sensor"
    fi
  fi

  if [ "$sensor_status" != "Sensor" ]; then
    echo "Warning: Failed to start sensor: $sensor_status"
    echo "Response: $output"
  fi

  downstream_asset_path='["dbt_lana_dw","staging","rollups","stg_core_withdrawal_events_rollup"]'
  asset_runs_vars=$(jq -n '{ limit: 50 }')
  exec_dagster_graphql "asset_runs" "$asset_runs_vars"
  dagster_validate_json || return 1

  initial_run_ids=$(echo "$output" | jq -r --argjson assetPath "$downstream_asset_path" '.data.runsOrError.results[]? | select(.assetSelection != null and (.assetSelection | length > 0)) | select(any(.assetSelection[]; .path == $assetPath)) | .runId' | sort)

  upstream_variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB",
        assetSelection: [
          { path: ["lana", "core_withdrawal_events_rollup"] }
        ]
      },
      runConfigData: {}
    }
  }')

  exec_dagster_graphql "launch_run" "$upstream_variables"
  dagster_check_launch_run_errors || return 1

  upstream_run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  [ -n "$upstream_run_id" ] || { echo "Failed to launch upstream run: $output"; return 1; }

  dagster_poll_run_status "$upstream_run_id" 10 30 || return 1

  upstream_status_vars=$(jq -n --arg runId "$upstream_run_id" '{ runId: $runId }')
  exec_dagster_graphql "run_status" "$upstream_status_vars"
  dagster_validate_json || return 1

  attempts=60
  sleep_between=2
  downstream_run_started=false
  new_run_id=""

  while [ $attempts -gt 0 ]; do
    exec_dagster_graphql "asset_runs" "$asset_runs_vars"
    dagster_validate_json || return 1

    current_run_ids=$(echo "$output" | jq -r --argjson assetPath "$downstream_asset_path" '.data.runsOrError.results[]? | select(.assetSelection != null and (.assetSelection | length > 0)) | select(any(.assetSelection[]; .path == $assetPath)) | .runId' | sort)

    for run_id in $current_run_ids; do
      if [ -n "$run_id" ]; then
        if ! echo "$initial_run_ids" | grep -q "^${run_id}$" && [ "$run_id" != "$upstream_run_id" ]; then
          run_status_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
          exec_dagster_graphql "run_status" "$run_status_vars"
          dagster_validate_json || continue

          run_status=$(echo "$output" | jq -r '.data.runOrError.status // empty')
          if [ "$run_status" = "QUEUED" ] || [ "$run_status" = "STARTING" ] || [ "$run_status" = "STARTED" ] || [ "$run_status" = "SUCCESS" ]; then
            downstream_run_started=true
            new_run_id="$run_id"
            break
          fi
        fi
      fi
    done

    if [ "$downstream_run_started" = "true" ]; then
      break
    fi

    attempts=$((attempts-1))
    sleep $sleep_between
  done

  if [ "$downstream_run_started" = "false" ]; then
    echo "Downstream dbt asset did not automatically start after upstream completion"
    echo "Upstream run ID: $upstream_run_id"
    echo "Initial downstream run IDs:"
    echo "$initial_run_ids"
    echo "Current downstream run IDs:"
    echo "$current_run_ids"
    stop_automation_sensor
    return 1
  fi

  echo "Downstream dbt asset automatically started (run ID: $new_run_id) after upstream completion"
  stop_automation_sensor
}

@test "dagster: verify dbt seed asset static_ncf_01_03_row_titles_seed exists" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  seed_asset_path=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[-1] == "static_ncf_01_03_row_titles_seed")][0] // empty')
  if [ -z "$seed_asset_path" ]; then
    echo "dbt seed asset static_ncf_01_03_row_titles_seed not found in Dagster assets"
    echo "Available dbt_lana_dw assets matching static_ncf_01_03:*"
    echo "$output" | jq '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and (.[-1] | contains("static_ncf_01_03")))'
    return 1
  fi
}

@test "dagster: materialize dbt seeds and verify success" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  seed_asset_path=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[-1] == "static_ncf_01_03_row_titles_seed")][0] // empty')
  if [ -z "$seed_asset_path" ]; then
    echo "dbt seed asset static_ncf_01_03_row_titles_seed not found in Dagster assets"
    echo "Response: $output"
    return 1
  fi

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

  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  if [ -z "$run_id" ]; then
    echo "Failed to launch dbt_seeds_job - no runId returned"
    echo "Response: $output"
    return 1
  fi

  echo "Launched dbt_seeds_job with run ID: $run_id"

  dagster_poll_run_status "$run_id" 480 2 || return 1

  asset_vars=$(jq -n --argjson path "$seed_asset_path" '{
    assetKey: { path: $path }
  }')
  exec_dagster_graphql "asset_materializations" "$asset_vars"

  dagster_validate_json || return 1

  asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
  [ "$asset_type" = "Asset" ] || { echo "Asset not found: $output"; return 1; }

  recent_run_id=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations[0].runId // empty')
  [ "$recent_run_id" = "$run_id" ] || { echo "Expected run ID $run_id, got $recent_run_id"; return 1; }
}

@test "dagster: verify static_ncf_01_03_column_01_account_config depends on static_ncf_01_03_row_titles_seed" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  seed_asset_path=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[-1] == "static_ncf_01_03_row_titles_seed")][0] // empty')
  model_asset_path=$(echo "$output" | jq -c '[.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[-1] == "static_ncf_01_03_column_01_account_config")][0] // empty')

  if [ -z "$model_asset_path" ]; then
    echo "Model static_ncf_01_03_column_01_account_config not found in Dagster assets"
    echo "Available dbt_lana_dw assets matching static_ncf_01_03:*"
    echo "$output" | jq '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and (.[-1] | contains("static_ncf_01_03")))'
    return 1
  fi

  if [ -z "$seed_asset_path" ]; then
    echo "Seed static_ncf_01_03_row_titles_seed not found in Dagster assets"
    echo "Available dbt_lana_dw assets matching static_ncf_01_03:*"
    echo "$output" | jq '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and (.[-1] | contains("static_ncf_01_03")))'
    return 1
  fi

  model_asset_vars=$(jq -n --argjson path "$model_asset_path" '{
    assetKey: { path: $path }
  }')
  exec_dagster_graphql "asset_dependencies" "$model_asset_vars"

  dagster_validate_json || return 1

  asset_node=$(echo "$output" | jq -e '.data.assetNodes[0] // empty')
  [ -n "$asset_node" ] || { echo "Asset static_ncf_01_03_column_01_account_config not found: $output"; return 1; }

  if ! echo "$output" | jq -e --argjson seedPath "$seed_asset_path" '.data.assetNodes[0].dependencies[]?.asset.assetKey.path | select(. == $seedPath)' >/dev/null; then
    echo "static_ncf_01_03_column_01_account_config does not depend on static_ncf_01_03_row_titles_seed"
    echo "Dependencies found:"
    echo "$output" | jq '.data.assetNodes[0].dependencies[]?.asset.assetKey.path'
    return 1
  fi

  echo "Model correctly depends on seed asset"
}
