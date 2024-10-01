#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "credit-facility: can create" {
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
        terms: {
          annualRate: "12",
          interval: "END_OF_MONTH",
          duration: { period: "MONTHS", units: 3 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140"
        }
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"
  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  cache_value 'credit_facility_id' "$credit_facility_id"
}

@test "credit-facility: can approve" {
  credit_facility_id=$(read_value 'credit_facility_id')

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

@test "credit-facility: can initiate disbursement" {
  credit_facility_id=$(read_value 'credit_facility_id')

  amount=50000
  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson amount "$amount" \
    '{
      input: {
        creditFacilityId: $creditFacilityId,
        amount: $amount,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-disbursement-initiate' "$variables"
  disbursement_index=$(graphql_output '.data.creditFacilityDisbursementInitiate.disbursement.index')
  [[ "$disbursement_index" != "null" ]] || exit 1

  cache_value 'disbursement_index' "$disbursement_index"
}

@test "credit-facility: can approve disbursement" {
  credit_facility_id=$(read_value 'credit_facility_id')
  disbursement_index=$(read_value 'disbursement_index')

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson disbursementIdx "$disbursement_index" \
    '{
      input: {
        creditFacilityId: $creditFacilityId,
        disbursementIdx: $disbursementIdx,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-disbursement-approve' "$variables"
  disbursement_id=$(graphql_output '.data.creditFacilityDisbursementApprove.disbursement.id')
  [[ "$disbursement_id" != "null" ]] || exit 1

  assert_accounts_balanced
}
