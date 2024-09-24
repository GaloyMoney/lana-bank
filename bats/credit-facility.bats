#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "credit-facility: credit-facility create" {
  # Setup prerequisites
  customer_id=$(create_customer)

  amount=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --argjson amount "$amount" \
    '{
      input: {
        customerId: $customerId,
        amount: $amount,
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"
  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1
}

