#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility.e2e-logs"
RUN_LOG_FILE="credit-facility.run.e2e-logs"

setup_file() {
  start_server
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

wait_for_disbursal() {
  credit_facility_id=$1

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  echo "disbursal | $i. $(graphql_output)" >> $RUN_LOG_FILE
  disbursals=$(graphql_output '.data.creditFacility.disbursals')
  num_disbursals=$(echo $disbursals | jq -r '. | length')
  [[ "$num_disbursals" -gt "1" ]]
}

wait_for_accruals() {
  expected_num_accruals=$1
  credit_facility_id=$2

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  echo "accrual | $i. $(graphql_output)" >> $RUN_LOG_FILE
  num_accruals=$(
    graphql_output '[
      .data.creditFacility.transactions[]
      | select(.__typename == "CreditFacilityInterestAccrued")
      ] | length'
  )

  [[ "$num_accruals" == "$expected_num_accruals" ]] || exit 1
}

ymd() {
  local date_value
  read -r date_value
  echo $date_value | cut -d 'T' -f1 | tr -d '-'
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
          oneTimeFeeRate: "5",
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
  echo "credit_facility_id: $credit_facility_id"

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
  exec_admin_graphql 'credit-facility-collateral-update' "$variables"
  echo $(graphql_output)
  credit_facility_id=$(graphql_output '.data.creditFacilityCollateralUpdate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  retry 10 1 wait_for_active "$credit_facility_id"
}

@test "credit-facility: can initiate disbursal" {
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
  exec_admin_graphql 'credit-facility-disbursal-initiate' "$variables"
  echo $(graphql_output)
  disbursal_index=$(graphql_output '.data.creditFacilityDisbursalInitiate.disbursal.index')
  [[ "$disbursal_index" != "null" ]] || exit 1

  retry 10 1 wait_for_disbursal "$credit_facility_id"
}

@test "credit-facility: records accrual" {
  credit_facility_id=$(read_value 'credit_facility_id')
  retry 30 2 wait_for_accruals 4 "$credit_facility_id"

  cat_logs | grep "interest accrual job completed.*$credit_facility_id" || exit 1

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  last_accrual=$(
    graphql_output '[
      .data.creditFacility.transactions[]
      | select(.__typename == "CreditFacilityInterestAccrued")
      ][0]'
  )

  amount=$(echo $last_accrual | jq -r '.cents')
  [[ "$amount" -gt "0" ]] || exit 1

  last_accrual_at=$(echo $last_accrual | jq -r '.recordedAt' | ymd)
  expires_at=$(graphql_output '.data.creditFacility.expiresAt' | ymd)
  [[ "$last_accrual_at" == "$expires_at" ]] || exit 1

  # assert_accounts_balanced
}
