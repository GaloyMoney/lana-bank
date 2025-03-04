#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility.e2e-logs"
RUN_LOG_FILE="credit-facility.run.e2e-logs"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_approval() {
  variables=$(
    jq -n \
      --arg withdrawId "$1" \
    '{ id: $withdrawId }'
  )
  exec_admin_graphql 'find-withdraw' "$variables"
  status=$(graphql_output '.data.withdrawal.status')
  [[ "$status" == "PENDING_CONFIRMATION" ]] || return 1
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

ymd() {
  local date_value
  read -r date_value
  echo $date_value | cut -d 'T' -f1 | tr -d '-'
}

create_credit_facility_with_terms() {
  customer_id=$(create_customer)
  cache_value "customer_id" "$customer_id"

  facility=$1
  terms=$2

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
      --argjson facility "$facility" \
      --argjson terms "$terms" \
      '{
      input: {
        customerId: $customerId,
        facility: $facility,
        terms: $terms
      }
    }'
  )

  exec_admin_graphql 'credit-facility-create' "$variables"
  credit_facility_id=$(graphql_output '.data.creditFacilityCreate.creditFacility.creditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1
  cache_value 'credit_facility_id' "$credit_facility_id"
}

update_collateral() {
  credit_facility_id=$1
  collateral=$2

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson collateral "$collateral" \
      '{
        input: {
          creditFacilityId: $creditFacilityId,
          collateral: $collateral
        }
      }'
  )
  exec_admin_graphql 'credit-facility-collateral-update' "$variables"

  retry 10 1 wait_for_active "$credit_facility_id"
}

initiate_disbursal() {
  credit_facility_id=$1
  amount=$2

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --argjson amount "$amount" \
      '{
        input: {
          creditFacilityId: $creditFacilityId,
          amount: $amount
        }
      }'
  )
  exec_admin_graphql 'credit-facility-disbursal-initiate' "$variables"
  disbursal_index=$(graphql_output '.data.creditFacilityDisbursalInitiate.disbursal.index')
  [[ "$disbursal_index" != "null" ]] || exit 1

}

record_accruals() {
  credit_facility_id=$1
  sleep $((RANDOM % 31 + 30))
}

withdraw_amount() {
  customer_id=$1
  amount=$2

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
      --arg date "$(date +%s%N)" \
      --argjson amount "$amount" \
      '{
        input: {
          customerId: $customerId,
          amount: $amount,
          reference: ("withdrawal-ref-" + $date)
        }
      }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"
  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  retry 5 1 wait_for_approval "$withdrawal_id"

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
      '{
        input: {
          withdrawalId: $withdrawalId
        }
      }'
  )
  exec_admin_graphql 'confirm-withdrawal' "$variables"
}

deposit_amount() {
  customer_id=$1
  amount=$2

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
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1

}

generate_facilities_with_multiple_terms() {
  declare -a facilities=(
    1000000000
    2000000000
    3000000000
  )

  declare -a terms=(
    '{"annualRate":"10","incurrenceInterval": "END_OF_DAY", "accrualInterval":"END_OF_MONTH", "duration": { "period": "MONTHS", "units": 6 }, "initialCvl":"150", "marginCallCvl":"130", "liquidationCvl":"110"}'
    '{"annualRate":"12","incurrenceInterval": "END_OF_DAY", "accrualInterval":"END_OF_MONTH", "duration": { "period": "MONTHS", "units": 15 }, "initialCvl":"140", "marginCallCvl":"120", "liquidationCvl":"100"}'
    '{"annualRate":"8","incurrenceInterval": "END_OF_DAY", "accrualInterval":"END_OF_MONTH", "duration": { "period": "MONTHS", "units": 7 }, "initialCvl":"160", "marginCallCvl":"140", "liquidationCvl":"120"}'
  )

  for facility in "${facilities[@]}"; do
    for term in "${terms[@]}"; do
      create_credit_facility_with_terms "$facility" "$term"

      credit_facility_id=$(read_value 'credit_facility_id')
      customer_id=$(read_value 'customer_id')

      update_collateral "$credit_facility_id" 900000000000

      initiate_disbursal "$credit_facility_id" 500000000

      record_accruals "$credit_facility_id"

      withdraw_amount "$customer_id" 100000000

      deposit_amount "$customer_id" 40000000

    done
  done
}

whale_account() {
  customer_id=$(create_customer)

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId,
        amount: 15000000000000,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1
}

@test "credit-facility: generate multiple facilities" {
  skip

  whale_account
  generate_facilities_with_multiple_terms
}
