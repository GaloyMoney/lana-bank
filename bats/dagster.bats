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
    return 1
  fi
  
  echo "Downstream dbt asset automatically started (run ID: $new_run_id) after upstream completion"
}

@test "dagster: verify dbt seed asset exists" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials for code location to load"
  fi

  exec_dagster_graphql "assets"
  dagster_validate_json || return 1

  # Check if the test seed asset exists
  if ! echo "$output" | jq -e '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw" and .[-1] == "_TEST_DO_NOT_USE_example_seed")' >/dev/null; then
    echo "dbt seed asset _TEST_DO_NOT_USE_example_seed not found in Dagster assets"
    echo "Available dbt_lana_dw assets:"
    echo "$output" | jq '.data.assetsOrError.nodes[]?.key.path | select(.[0] == "dbt_lana_dw")'
    return 1
  fi
}

@test "dagster: run dbt_seeds_job and verify success" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Launch the dbt_seeds_job
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
  
  dagster_poll_run_status "$run_id" 300 1 || return 1
  
  echo "dbt_seeds_job completed successfully"
}

@test "dagster: verify model depends on seed" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi
  if ! has_bigquery_credentials; then
    skip "Skipping - requires BigQuery credentials"
  fi

  # Query dependencies of the test model that consumes the seed
  model_asset_vars=$(jq -n '{
    assetKey: { path: ["dbt_lana_dw", "staging", "_TEST_DO_NOT_USE_seed_consumer"] }
  }')
  exec_dagster_graphql "asset_dependencies" "$model_asset_vars"
  
  dagster_validate_json || return 1
  
  # Check if the asset node exists
  asset_node=$(echo "$output" | jq -e '.data.assetNodes[0] // empty')
  [ -n "$asset_node" ] || { echo "Asset _TEST_DO_NOT_USE_seed_consumer not found: $output"; return 1; }
  
  # Check if the seed is in the dependencies
  if ! echo "$output" | jq -e '.data.assetNodes[0].dependencies[]?.asset.assetKey.path | select(. == ["dbt_lana_dw", "seeds", "_TEST_DO_NOT_USE_example_seed"])' >/dev/null; then
    echo "_TEST_DO_NOT_USE_seed_consumer does not depend on seed"
    echo "Dependencies found:"
    echo "$output" | jq '.data.assetNodes[0].dependencies[].asset.assetKey.path'
    return 1
  fi
  
  echo "Model correctly depends on seed asset"
}
