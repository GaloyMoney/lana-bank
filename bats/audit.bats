#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "audit: check audit logs" {
  local audit_action="audit:audit:list"
  local first_one_vars="{\"first\": 1, \"action\": \"$audit_action\"}"
  local first_two_vars="{\"first\": 2, \"action\": \"$audit_action\"}"

  exec_admin_graphql 'audit-logs' "$first_one_vars"
  exec_admin_graphql 'audit-logs' "$first_one_vars"
  exec_admin_graphql 'audit-logs' "$first_one_vars"

  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 1 ]] || exit 1

  action=$(graphql_output '.data.audit.edges[-1].node.action')
  [[ "$action" == "audit:audit:list" ]] || exit 1


  exec_admin_graphql 'audit-logs' "$first_two_vars"
  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 2 ]] || exit 1

  action=$(graphql_output '.data.audit.edges[-1].node.action')
  [[ "$action" == "audit:audit:list" ]] || exit 1

  end_cursor=$(graphql_output '.data.audit.pageInfo.endCursor')
  [[ -n "$end_cursor" ]] || exit 1  # Ensure endCursor is not empty
  echo "end_cursor: $end_cursor"

  exec_admin_graphql 'audit-logs' "{\"first\": 2, \"after\": \"$end_cursor\", \"action\": \"$audit_action\"}"
  echo "$output"

  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 2 ]] || exit 1
}
