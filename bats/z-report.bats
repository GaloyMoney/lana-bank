#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

wait_for_complete() {
  variables=$(
    jq -n \
    --arg reportId "$1" \
    '{
      id: $reportId,
    }'
  )
  exec_admin_graphql 'find-report' "$variables"
  echo $(graphql_output)
  progress=$(graphql_output .data.report.progress)
  [[ "$progress" == "COMPLETE" ]] || return 1
}

@test "report: create" {
  exec_admin_graphql 'report-create'
  echo $(graphql_output)
  report_id=$(graphql_output .data.reportCreate.report.reportId)
  [[ "$report_id" != "null" ]] || exit 1

  retry 30 1 wait_for_complete "$report_id"

  variables=$(
    jq -n \
    --arg reportId "$report_id" \
    '{
      input: {
        reportId: $reportId
      }
    }'
  )
  exec_admin_graphql 'report-download-links' "$variables"
  links=$(graphql_output .data.reportDownloadLinksGenerate.links)
  length=$(echo $links | jq -r 'length')
  [[ "$length" -gt "0" ]] || exit 1

  url=$(echo $links | jq -r '.[0].url')
  xml_file_contents=$(curl -fsSL "$url") || exit 1
  echo $xml_file_contents | grep "<?xml" || exit 1
}
