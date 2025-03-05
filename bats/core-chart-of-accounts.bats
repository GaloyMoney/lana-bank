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


@test "chart-of-accounts: can import CSV file" {
  skip
  chart_id="8414eb0e-389e-4ed2-933b-b27d061202fc"

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

  exec_admin_graphql_upload 'chart-of-accounts-csv-import' "$variables" "$temp_file" "input.file"
  success=$(graphql_output '.data.chartOfAccountsCsvImport.success')
  [[ "$success" == "true" ]] || exit 1
}
