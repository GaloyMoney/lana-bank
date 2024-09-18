#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "report: create" {
  exec_admin_graphql 'report-create'
  echo $(graphql_output)
  report_id=$(graphql_output .data.reportCreate.report.reportId)
  [[ "$report_id" != "null" ]] || exit 1
}
