#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="chart-of-accounts.e2e-logs"
RUN_LOG_FILE="chart-of-accounts.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

@test "chart-of-accounts: can create a new chart" {
  name="COA-$(date +%s)"
  reference="COA-$(date +%s)"
  
  variables=$(
    jq -n \
    --arg name "$name" \
    --arg reference "$reference" \
    '{
      input: {
        name: $name,
        reference: $reference
      }
    }'
  )

  exec_admin_graphql 'chart-of-accounts-create' "$variables"

  chart_id=$(graphql_output '.data.chartOfAccountsCreate.chartOfAccounts.chartId')
  [[ "$chart_id" != "null" ]] || exit 1

  cache_value "chart_id" "$chart_id"

}

@test "chart-of-accounts: can import CSV file" {
  chart_id=$(read_value "chart_id")
  echo "chart_id: $chart_id"
  
  temp_file=$(mktemp)
  echo "
    1,,,Assets ,,
    ,,,,,
    11,,,Assets,,
    ,,,,,
    ,01,,Effective,,
    ,,0101,Central Office,
    " > "$temp_file"

  variables=$(
    jq -n \
    --arg chart_id "$chart_id" \
    '{
      input: {
        chartId: $chart_id,
        file: null
      }
    }'
  )

  response=$(exec_admin_graphql_upload 'chart-of-accounts-csv-import' "$variables" "$temp_file" "input.file")

  success=$(echo "$response" | jq -r '.data.chartOfAccountsCsvImport.success')
  [[ "$success" == "true" ]] || exit 1
}
