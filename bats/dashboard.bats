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

wait_for_pending_facilities(){
  exec_admin_graphql 'dashboard'
  pending_facilities=$(read_value 'pending_facilities')

  new_pending_facilities=$(graphql_output '.data.dashboard.pendingFacilities')
  [[ "$new_pending_facilities" != "$pending_facilities" ]] || exit 1
}

wait_for_active_facilities(){
  exec_admin_graphql 'dashboard'
  active_facilities=$(read_value 'active_facilities')

  new_active_facilities=$(graphql_output '.data.dashboard.activeFacilities')
  [[ "$new_active_facilities" != "$active_facilities" ]] || exit 1
}

wait_for_total_disbursed() {
  exec_admin_graphql 'dashboard'
  total_disbursed=$(read_value 'total_disbursed')

  new_total_disbursed=$(graphql_output '.data.dashboard.totalDisbursed')
  [[ "$new_total_disbursed" != "$total_disbursed" ]] || exit 1
}

@test "dashboard: counts facilities" {
  skip
  # test is broken

  customer_id=$(create_customer)

  exec_admin_graphql 'dashboard'
  pending_facilities=$(graphql_output '.data.dashboard.pendingFacilities')
  [[ "$pending_facilities" != "null" ]] || exit 1
  cache_value 'pending_facilities' "$pending_facilities"

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
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
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
  collateral_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.collateralId')

  retry 60 wait_for_pending_facilities

  exec_admin_graphql 'dashboard'
  active_facilities=$(graphql_output '.data.dashboard.activeFacilities')
  [[ "$active_facilities" != "null" ]] || exit 1
  cache_value 'active_facilities' "$active_facilities"

  "$LANACLI" --json collateral update \
    --collateral-id "$collateral_id" \
    --collateral 50000000 \
    --effective "$(naive_now)"

  retry 60 wait_for_active_facilities

  exec_admin_graphql 'dashboard'
  total_disbursed=$(graphql_output '.data.dashboard.totalDisbursed')
  cache_value 'total_disbursed' "$total_disbursed"

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
  exec_admin_graphql 'credit-facility-disbursal-initiate' "$variables"
  disbursal_index=$(graphql_output '.data.creditFacilityDisbursalInitiate.disbursal.index')

    variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson disbursalIdx "$disbursal_index" \
    '{
      input: {
        creditFacilityId: $creditFacilityId,
        disbursalIdx: $disbursalIdx,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-disbursal-confirm' "$variables"

  retry 60 wait_for_total_disbursed
}
