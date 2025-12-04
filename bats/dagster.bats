#!/usr/bin/env bats

load helpers

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
  dagster_validate_json || skip "Dagster GraphQL did not return JSON"
  
  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  [ -n "$run_id" ] || { echo "$output"; return 1; }
  
  dagster_poll_run_status "$run_id" 60 2 || return 1
  
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
  
  dagster_poll_run_status "$run_id" 90 2 || return 1
  
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

@test "dagster: materialize stg_core_withdrawal_events_rollup" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi

  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB",
        assetSelection: [
          { path: ["dbt_lana_dw", "staging", "rollups", "stg_core_withdrawal_events_rollup"] }
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
  
  dagster_poll_run_status "$run_id" 90 2 || return 1

  asset_vars=$(jq -n '{
    assetKey: { path: ["dbt_lana_dw", "staging", "rollups", "stg_core_withdrawal_events_rollup"] }
  }')
  exec_dagster_graphql "asset_materializations" "$asset_vars"

  dagster_validate_json || return 1

  asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
  [ "$asset_type" = "Asset" ] || { echo "Asset stg_core_withdrawal_events_rollup not found: $output"; return 1; }

  recent_run_id=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations[0].runId // empty')
  [ "$recent_run_id" = "$run_id" ] || { echo "Expected run ID $run_id for stg_core_withdrawal_events_rollup, got $recent_run_id"; return 1; }
}

@test "dagster: verify stg_core_withdrawal_events_rollup depends on core_withdrawal_events_rollup" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi

  # Query dependencies of the staging model
  staging_asset_vars=$(jq -n '{
    assetKey: { path: ["dbt_lana_dw", "staging", "rollups", "stg_core_withdrawal_events_rollup"] }
  }')
  exec_dagster_graphql "asset_dependencies" "$staging_asset_vars"
  
  dagster_validate_json || return 1
  
  # Check if the asset node exists
  asset_node=$(echo "$output" | jq -e '.data.assetNodes[0] // empty')
  [ -n "$asset_node" ] || { echo "Asset stg_core_withdrawal_events_rollup not found: $output"; return 1; }
  
  # Check if the EL asset is in the dependencies
  if ! echo "$output" | jq -e '.data.assetNodes[0].dependencies[]?.asset.assetKey.path | select(. == ["lana", "core_withdrawal_events_rollup"])' >/dev/null; then
    echo "stg_core_withdrawal_events_rollup does not depend on [\"lana\", \"core_withdrawal_events_rollup\"]"
    echo "Dependencies found:"
    echo "$output" | jq '.data.assetNodes[0].dependencies[].asset.assetKey.path'
    return 1
  fi
}

@test "dagster: verify dbt asset automatically starts when upstream completes" {
  if [[ "${DAGSTER}" != "true" ]]; then
    skip "Skipping dagster tests"
  fi

  # Note: The automation condition must be set on dbt assets for this test to work.
  # If Dagster shows automationCondition as null in the UI, the code changes haven't been
  # picked up and Dagster needs to reload the code location.

  # Enable the dbt automation condition sensor so automation conditions are evaluated for dbt assets
  # We use the custom sensor for dbt assets, but also enable the default one as fallback
  sensor_vars=$(jq -n '{
    sensorSelector: {
      repositoryLocationName: "Lana DW",
      repositoryName: "__repository__",
      sensorName: "dbt_automation_condition_sensor"
    }
  }')
  exec_dagster_graphql "start_sensor" "$sensor_vars"
  dagster_validate_json || return 1
  
  # Check if sensor was started successfully
  sensor_status=$(echo "$output" | jq -r '.data.startSensor.__typename // empty')
  if [ "$sensor_status" = "SensorNotFoundError" ]; then
    # Fallback to default sensor if custom one doesn't exist
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

  # Get the current runs for the downstream asset to establish baseline
  # Query recent runs and filter for ones that target our downstream asset
  downstream_asset_path='["dbt_lana_dw","staging","rollups","stg_core_withdrawal_events_rollup"]'
  asset_runs_vars=$(jq -n '{ limit: 50 }')
  exec_dagster_graphql "asset_runs" "$asset_runs_vars"
  dagster_validate_json || return 1
  
  # Filter runs to find those that target the downstream asset
  # Get the runIds of existing runs before we start
  initial_run_ids=$(echo "$output" | jq -r --argjson assetPath "$downstream_asset_path" '.data.runsOrError.results[]? | select(.assetSelection != null and (.assetSelection | length > 0)) | select(any(.assetSelection[]; .path == $assetPath)) | .runId' | sort)
  
  # Materialize the upstream asset
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
  
  # Wait for upstream to complete
  dagster_poll_run_status "$upstream_run_id" 90 2 || return 1
  
  # Get the completion timestamp of the upstream run
  upstream_status_vars=$(jq -n --arg runId "$upstream_run_id" '{ runId: $runId }')
  exec_dagster_graphql "run_status" "$upstream_status_vars"
  dagster_validate_json || return 1
  
  # Poll for a new run of the downstream asset that was automatically triggered
  # We check runs directly (not just materializations) to detect when automation starts a run
  # Automation sensor may take a moment to evaluate and trigger, so we poll with retries
  attempts=60
  sleep_between=2
  downstream_run_started=false
  new_run_id=""
  
  while [ $attempts -gt 0 ]; do
    exec_dagster_graphql "asset_runs" "$asset_runs_vars"
    dagster_validate_json || return 1
    
    # Filter runs to find those that target the downstream asset
    current_run_ids=$(echo "$output" | jq -r --argjson assetPath "$downstream_asset_path" '.data.runsOrError.results[]? | select(.assetSelection != null and (.assetSelection | length > 0)) | select(any(.assetSelection[]; .path == $assetPath)) | .runId' | sort)
    
    # Check if we have any new runs (runs that weren't in the initial set)
    for run_id in $current_run_ids; do
      if [ -n "$run_id" ]; then
        # Check if this run is new (not in initial set and not the upstream run)
        if ! echo "$initial_run_ids" | grep -q "^${run_id}$" && [ "$run_id" != "$upstream_run_id" ]; then
          # Found a new run! Check its status to confirm it was started
          run_status_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
          exec_dagster_graphql "run_status" "$run_status_vars"
          dagster_validate_json || continue
          
          run_status=$(echo "$output" | jq -r '.data.runOrError.status // empty')
          # Accept runs that are queued, started, or in progress (not just completed)
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
    echo ""
    echo "NOTE: This test requires:"
    echo "  1. Automation condition (AutomationCondition.eager()) must be set on dbt assets"
    echo "  2. The default_automation_condition_sensor must be enabled in Dagster UI"
    echo "  3. Dagster must have reloaded the code location to pick up code changes"
    echo ""
    echo "Check the Dagster UI to verify the automation condition is set on the asset."
    return 1
  fi
  
  echo "✅ Downstream dbt asset automatically started (run ID: $new_run_id) after upstream completion"
}

