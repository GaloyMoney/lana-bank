#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "owners-equity: can add" {
  variables=$(
    jq -n \
      --arg reference "equity-${RANDOM}" \
    '{
      input: {
        amount: 1000000000,
        reference: $reference
      }
    }'
  )
  exec_admin_graphql 'owners-equity-add' "$variables"
  success=$(graphql_output '.data.ownersEquityAdd.success')
  [[ "$success" == "true" ]] || exit 1
}
