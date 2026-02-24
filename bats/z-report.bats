#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

wait_for_report_run_success() {
  local run_id=$1
  variables=$(
    jq -n \
    --arg id "$run_id" \
    '{ id: $id }'
  )
  exec_admin_graphql 'find-report' "$variables"
  state=$(graphql_output .data.reportRun.state)
  [[ "$state" == "SUCCESS" ]] || return 1
}

@test "report: trigger run, wait for completion, and download files" {
  if [[ "${DAGSTER:-}" != "true" ]]; then
    skip "Skipping report test because DAGSTER is not enabled"
  fi
  if [[ -z "${SA_CREDS_BASE64:-}" ]]; then
    skip "Skipping report test because SA_CREDS_BASE64 is not defined"
  fi

  # Trigger a report run
  exec_admin_graphql 'trigger-report-run'
  echo "triggerReportRun response: $(graphql_output)"

  # Wait briefly for the run to be created
  sleep 5

  # Find the most recent report run
  variables=$(jq -n '{ first: 1 }')
  exec_admin_graphql 'report-runs' "$variables"
  echo "reportRuns response: $(graphql_output)"
  run_id=$(graphql_output '.data.reportRuns.nodes[0].reportRunId')
  [[ "$run_id" != "null" && -n "$run_id" ]] || exit 1

  # Wait for the report run to complete (up to ~8 minutes)
  retry 240 2 wait_for_report_run_success "$run_id"

  # Fetch completed run details
  variables=$(jq -n --arg id "$run_id" '{ id: $id }')
  exec_admin_graphql 'find-report' "$variables"
  echo "Completed run: $(graphql_output)"

  # Extract reports and their files, then generate download links
  reports_json=$(graphql_output '.data.reportRun.reports')
  reports_length=$(echo "$reports_json" | jq 'length')
  [[ "$reports_length" -gt 0 ]] || exit 1

  for i in $(seq 0 $((reports_length - 1))); do
    report_id=$(echo "$reports_json" | jq -r ".[$i].reportId")
    files_json=$(echo "$reports_json" | jq ".[$i].files")
    files_length=$(echo "$files_json" | jq 'length')
    [[ "$files_length" -gt 0 ]] || exit 1

    for j in $(seq 0 $((files_length - 1))); do
      extension=$(echo "$files_json" | jq -r ".[$j].extension")

      variables=$(jq -n \
        --arg reportId "$report_id" \
        --arg extension "$extension" \
        '{
          input: {
            reportId: $reportId,
            extension: $extension
          }
        }')

      exec_admin_graphql 'report-file-download-link' "$variables"
      echo "Download link response: $(graphql_output)"

      url=$(graphql_output '.data.reportFileGenerateDownloadLink.url')
      [[ "$url" != "null" && -n "$url" ]] || exit 1

      # Handle both local file:// URLs and HTTP URLs
      if [[ "$url" == file://* ]]; then
        local_path="${url#file://}"
        [[ -f "$local_path" ]] || exit 1
        echo "Local file verified: $local_path"
      else
        response=$(curl -s -o /dev/null -w "%{http_code}" "$url")
        [[ "$response" == "200" ]] || exit 1
        echo "HTTP download verified: $url"
      fi
    done
  done
}
