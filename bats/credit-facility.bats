#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "credit-facility: credit-facility lifecycle" {
  # Setup prerequisites
  customer_id=$(create_customer)

  facility=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --argjson facility "$facility" \
    '{
      input: {
        customerId: $customerId,
        facility: $facility,
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"
  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg credit_facility_id "$credit_facility_id" \
    '{
      input: {
        creditFacilityId: $credit_facility_id,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-approve' "$variables"
  loan_id=$(graphql_output '.data.creditFacilityApprove.creditFacility.creditFacilityId')
  [[ "$loan_id" != "null" ]] || exit 1

}

