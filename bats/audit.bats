#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "audit: check audit logs" {
  exec_admin_graphql 'audit-logs' '{"first": 1}'
  exec_admin_graphql 'audit-logs' '{"first": 1}'
  exec_admin_graphql 'audit-logs' '{"first": 1}'

  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 1 ]] || exit 1

  action=$(graphql_output '.data.audit.edges[-1].node.action')
  [[ "$action" == "audit-list" ]] || exit 1


  exec_admin_graphql 'audit-logs' '{"first": 2}'
  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 2 ]] || exit 1

  action=$(graphql_output '.data.audit.edges[-1].node.action')
  [[ "$action" == "audit-list" ]] || exit 1

  end_cursor=$(graphql_output '.data.audit.pageInfo.endCursor')
  [[ -n "$end_cursor" ]] || exit 1  # Ensure endCursor is not empty
  echo "end_cursor: $end_cursor"

  exec_admin_graphql 'audit-logs' "{\"first\": 2, \"after\": \"$end_cursor\"}"
  echo "$output"

  edges_length=$(graphql_output '.data.audit.edges | length')
  [[ "$edges_length" -eq 2 ]] || exit 1

  id1=$(graphql_output '.data.audit.edges[0].node.id')
  id2=$(graphql_output '.data.audit.edges[1].node.id')

  expected_id1=$((end_cursor - 1))
  expected_id2=$((end_cursor - 2))

  [[ "$id1" == "audit_entry:$expected_id1" ]] || exit 1
  [[ "$id2" == "audit_entry:$expected_id2" ]] || exit 1
}
