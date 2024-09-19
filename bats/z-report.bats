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
  # fake service account used in concourse
  if echo "${BQ_SERVICE_ACCOUNT_BASE64}" | base64 -d | grep -q "abc_app"; then
    skip
  fi
  exec_admin_graphql 'report-create'
  echo $(graphql_output)
  report_id=$(graphql_output .data.reportCreate.report.reportId)
  [[ "$report_id" != "null" ]] || exit 1

  retry 60 2 wait_for_complete "$report_id"

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

  exec_admin_graphql 'report-list'
  report_id_from_list=$(graphql_output \
    --arg reportId "$report_id" \
    '.data.reports[] | select(.reportId == $reportId) .reportId'
  )
  [[ "$report_id_from_list" == "$report_id" ]] || exit 1
}
