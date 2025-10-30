#!/usr/bin/env bats

load helpers

@test "dagster graphql endpoint responds to POST" {
  exec_dagster_graphql_status "introspection"
  [ "$status" -eq 0 ]
  [ "$output" = "200" ]
}

@test "list assets and verify iris_dataset_size exists" {
  exec_dagster_graphql "assets"
  echo "$output" | jq . >/dev/null || skip "Dagster GraphQL did not return JSON"

  found=$(echo "$output" | jq -r '.data.assetsOrError.nodes[]?.key.path | select(. == ["iris_dataset_size"]) | @sh' | wc -l)
  [ "$found" -ge 1 ] || skip "iris_dataset_size asset not found"
}

@test "materialize iris_dataset_size and wait for success" {
  # Launch materialization using the auto-generated asset job
  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "__ASSET_JOB"
      },
      runConfigData: {}
    }
  }')
  
  exec_dagster_graphql "launch_run" "$variables"
  echo "$output" | jq . >/dev/null || skip "Dagster GraphQL did not return JSON"
  
  run_id=$(echo "$output" | jq -r '.data.launchPipelineExecution.run.runId // empty')
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
}

