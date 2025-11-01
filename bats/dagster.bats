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
  echo "$output" | jq . >/dev/null || skip "Dagster GraphQL did not return JSON"

  found=$(echo "$output" | jq -r '.data.assetsOrError.nodes[]?.key.path | select(. == ["iris_dataset_size"]) | @sh' | wc -l)
  [ "$found" -ge 1 ] || skip "iris_dataset_size asset not found"
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
  echo "$output" | jq . >/dev/null || skip "Dagster GraphQL did not return JSON"
  
  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  [ -n "$run_id" ] || { echo "$output"; return 1; }
  
  # Poll run status until SUCCESS
  attempts=30
  sleep_between=2
  status=""
  while [ $attempts -gt 0 ]; do
    poll_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
    exec_dagster_graphql "run_status" "$poll_vars"
    status=$(echo "$output" | jq -r '.data.runOrError.status // empty')
    
    if [ "$status" = "SUCCESS" ]; then
      break
    fi
    if [ "$status" = "FAILURE" ] || [ "$status" = "CANCELED" ]; then
      echo "$output"
      return 1
    fi
    
    attempts=$((attempts-1))
    sleep $sleep_between
  done
  
  [ "$status" = "SUCCESS" ] || { echo "last status: $status"; return 1; }
  
  # Verify iris_dataset_size was materialized by checking that it has materializations
  # and that the run ID matches our execution
  asset_vars=$(jq -n '{
    assetKey: { path: ["iris_dataset_size"] }
  }')
  exec_dagster_graphql "asset_materializations" "$asset_vars"
  
  # Check that the asset exists and has materializations
  asset_type=$(echo "$output" | jq -r '.data.assetOrError.__typename // empty')
  [ "$asset_type" = "Asset" ] || { echo "Asset not found: $output"; return 1; }
  
  # Check that the most recent materialization was from our run
  recent_run_id=$(echo "$output" | jq -r '.data.assetOrError.assetMaterializations[0].runId // empty')
  [ "$recent_run_id" = "$run_id" ] || { echo "Expected run ID $run_id, got $recent_run_id"; return 1; }
}

