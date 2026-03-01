#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

@test "audit: check audit logs" {
  local cli_output

  # Call audit list three times to generate audit entries
  "$LANACLI" --json audit list --first 1 > /dev/null
  "$LANACLI" --json audit list --first 1 > /dev/null
  cli_output=$("$LANACLI" --json audit list --first 1)

  edges_length=$(echo "$cli_output" | jq '. | length')
  [[ "$edges_length" -eq 1 ]] || exit 1

  action=$(echo "$cli_output" | jq -r '.[-1].node.action')
  [[ "$action" == "audit:audit:list" ]] || exit 1

  cli_output=$("$LANACLI" --json audit list --first 2)
  edges_length=$(echo "$cli_output" | jq '. | length')
  [[ "$edges_length" -eq 2 ]] || exit 1

  action=$(echo "$cli_output" | jq -r '.[-1].node.action')
  [[ "$action" == "audit:audit:list" ]] || exit 1

  end_cursor=$(echo "$cli_output" | jq -r '.[-1].cursor')
  [[ -n "$end_cursor" ]] || exit 1  # Ensure endCursor is not empty
  echo "end_cursor: $end_cursor"

  cli_output=$("$LANACLI" --json audit list --first 2 --after "$end_cursor")
  echo "$cli_output"

  edges_length=$(echo "$cli_output" | jq '. | length')
  [[ "$edges_length" -eq 2 ]] || exit 1
}
