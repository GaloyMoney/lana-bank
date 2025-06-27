#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-custody.e2e-logs"
RUN_LOG_FILE="credit-facility-custody.run.e2e-logs"

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

  disbursals=$(graphql_output '.data.creditFacility.disbursals')
  num_disbursals=$(echo $disbursals | jq -r '. | length')
  [[ "$num_disbursals" -gt "0" ]]
}


@test "credit-facility-custody: can create custodian" {
  variables=$(
    jq -n --arg name "Mock Custodian $((RANDOM % 1000))" \
    '{
      input: {
        mock: {
          name: $name
        }
      }
    }'
  )

  exec_admin_graphql 'custodian-create' "$variables"

  custodian_id=$(graphql_output '.data.custodianCreate.custodian.custodianId')

  echo $custodian_id

  cache_value 'custodian_id' "$custodian_id"
}

@test "credit-facility-custody: can create" {
  # Setup prerequisites
  customer_id=$(create_customer)

  retry 30 1 wait_for_checking_account "$customer_id"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  exec_admin_graphql 'customer' "$variables"

  deposit_account_id=$(graphql_output '.data.customer.depositAccount.depositAccountId')
  [[ "$deposit_account_id" != "null" ]] || exit 1

  custodian_id=$(read_value 'custodian_id')

  echo $custodian_id
  
  facility=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --arg custodianId "$custodian_id" \
    --arg disbursal_credit_account_id "$deposit_account_id" \
    --argjson facility "$facility" \
    '{
      input: {
        customerId: $customerId,
        facility: $facility,
        disbursalCreditAccountId: $disbursal_credit_account_id,
        custodianId: $custodianId,
        terms: {
          annualRate: "12",
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
          oneTimeFeeRate: "5",
          duration: { period: "MONTHS", units: 3 },
          interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
          obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
          obligationLiquidationDurationFromDue: { period: "DAYS", units: 60 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140"
        }
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"

  echo $(graphql_output)
  
  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  cache_value 'credit_facility_id' "$credit_facility_id"

  address=$(graphql_output '.data.creditFacilityCreate.creditFacility.wallet.address')
  [[ "$address" == "address" ]] || exit 1
}

@test "credit-facility-custody: cannot update collateral with a custodian" {
  credit_facility_id=$(read_value 'credit_facility_id')

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
  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "ManualUpdateError" ]] || exit 1
}
