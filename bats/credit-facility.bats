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
          accrualInterval: "END_OF_MONTH",
          incurrenceInterval: "END_OF_DAY",
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

@test "credit-facility: can update collateral" {
  credit_facility_id=$(read_value 'credit_facility_id')

  variables=$(
    jq -n \
      --arg credit_facility_id "$credit_facility_id" \
    '{
      input: {
        creditFacilityId: $credit_facility_id,
        collateral: 50000000,
      }
    }'
  )
  echo 'updating collateral'
  exec_admin_graphql 'credit-facility-collateral-update' "$variables"
  echo $(graphql_output)
  credit_facility_id=$(graphql_output '.data.creditFacilityCollateralUpdate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.creditFacilityCollateralUpdate.creditFacility.status')
  [[ "$status" == "ACTIVE" ]] || exit 1
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
  echo $(graphql_output)
  disbursement_index=$(graphql_output '.data.creditFacilityDisbursementInitiate.disbursement.index')
  [[ "$disbursement_index" != "null" ]] || exit 1

  cache_value 'disbursement_index' "$disbursement_index"
}

@test "credit-facility: can create and confirm disbursement" {
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
  exec_admin_graphql 'credit-facility-disbursement-confirm' "$variables"
  disbursement_id=$(graphql_output '.data.creditFacilityDisbursementConfirm.disbursement.id')
  [[ "$disbursement_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.creditFacilityDisbursementConfirm.disbursement.status')
  [[ "$status" == "CONFIRMED" ]] || exit 1

  assert_accounts_balanced
}
