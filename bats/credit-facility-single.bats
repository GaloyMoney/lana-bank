#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-single.e2e-logs"
RUN_LOG_FILE="credit-facility-single.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_active() {
  credit_facility_id=$1

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"

  status=$(graphql_output '.data.creditFacility.status')
  [[ "$status" == "ACTIVE" ]] || exit 1
}

@test "credit-facility: single full draw on activation blocks further disbursals" {
  customer_id=$(create_customer)

  retry 80 1 wait_for_checking_account "$customer_id"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{ id: $customerId }'
  )
  exec_admin_graphql 'customer' "$variables"

  deposit_account_id=$(graphql_output '.data.customer.depositAccount.depositAccountId')
  [[ "$deposit_account_id" != "null" ]] || exit 1

  facility=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --arg disbursal_credit_account_id "$deposit_account_id" \
    --argjson facility "$facility" \
    '{
      input: {
        customerId: $customerId,
        facility: $facility,
        disbursalCreditAccountId: $disbursal_credit_account_id,
        terms: {
          annualRate: "12",
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
          oneTimeFeeRate: "5",
          duration: { period: "MONTHS", units: 3 },
          interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
          obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
          obligationLiquidationDurationFromDue: { period: "DAYS", units: 360 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140",
          disbursalPolicy: "SINGLE_FULL_ON_ACTIVATION"
        }
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"

  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg credit_facility_id "$credit_facility_id" \
      --arg effective "$(naive_now)" \
    '{
      input: {
        creditFacilityId: $credit_facility_id,
        collateral: 50000000,
        effective: $effective,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-collateral-update' "$variables"
  retry 20 1 wait_for_active "$credit_facility_id"

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  num_disbursals=$(graphql_output '.data.creditFacility.disbursals | length')
  [[ "$num_disbursals" -ge 1 ]] || exit 1

  policy=$(graphql_output '.data.creditFacility.creditFacilityTerms.disbursalPolicy')
  [[ "$policy" == "SINGLE_FULL_ON_ACTIVATION" ]] || exit 1

  amount=1000
  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson amount "$amount" \
    '{ input: { creditFacilityId: $creditFacilityId, amount: $amount } }'
  )
  exec_admin_graphql 'credit-facility-disbursal-initiate' "$variables"

  err_count=$(graphql_output '.errors | length // 0')
  [[ "$err_count" -ge 1 ]]
}

