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

