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

  # Launch materialization using the explicit iris_dataset_size job
  variables=$(jq -n '{
    executionParams: {
      selector: {
        repositoryLocationName: "Lana DW",
        repositoryName: "__repository__",
        jobName: "iris_dataset_size_job"
      },
      runConfigData: {}
    }
  }')
  
  exec_dagster_graphql "launch_run" "$variables"
  run_id=$(echo "$output" | jq -r '.data.launchRun.run.runId // empty')
  [ -n "$run_id" ] || { echo "$output"; return 1; }
  
  # Poll run status until SUCCESS
  attempts=30
  sleep_between=2
  run_status=""
  while [ $attempts -gt 0 ]; do
    poll_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
    exec_dagster_graphql "run_status" "$poll_vars"
    run_status=$(echo "$output" | jq -r '.data.runOrError.status // empty')
    
    if [ "$run_status" = "SUCCESS" ]; then
      break
    fi
    if [ "$run_status" = "FAILURE" ] || [ "$run_status" = "CANCELED" ]; then
      echo "$output"
      return 1
    fi
    
    attempts=$((attempts-1))
    sleep $sleep_between
  done
  
  [ "$run_status" = "SUCCESS" ] || { echo "last status: $run_status"; return 1; }
}

