#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="liquidation.e2e-logs"
RUN_LOG_FILE="liquidation.run.e2e-logs"

setup_file() {
  export LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
  start_server
  login_superadmin
  login_lanacli
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_approval() {
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility proposal-get --id "$1")
  echo "approval | $i. $cli_output" >> $RUN_LOG_FILE
  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "APPROVED" ]] || return 1
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

wait_for_facility_to_be_under_liquidation_threshold() {
  credit_facility_id=$1

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  echo "liquidation | $i. $(graphql_output)" >> $RUN_LOG_FILE

  state=$(graphql_output '.data.creditFacility.collateralizationState')
  liquidations_len=$(graphql_output '[.data.creditFacility.liquidations[]] | length')

  [[ "$state" == "UNDER_LIQUIDATION_THRESHOLD" ]] || return 1
  [[ "$liquidations_len" -ge "1" ]] || return 1
}

@test "liquidation: can trigger liquidation when collateralization falls below threshold" {

  customer_id=$(create_customer)
  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")

  facility=10000000
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility proposal-create \
    --customer-id "$customer_id" \
    --facility-amount "$facility" \
    --annual-rate 12 \
    --accrual-interval END_OF_DAY \
    --accrual-cycle-interval END_OF_MONTH \
    --one-time-fee-rate 5 \
    --disbursal-policy SINGLE_DISBURSAL \
    --duration-months 3 \
    --initial-cvl 140 \
    --margin-call-cvl 125 \
    --liquidation-cvl 105 \
    --interest-due-days 0 \
    --overdue-days 50 \
    --liquidation-days 60)

  credit_facility_proposal_id=$(echo "$cli_output" | jq -r '.creditFacilityProposalId')
  [[ "$credit_facility_proposal_id" != "null" ]] || exit 1

  "$LANACLI" --json credit-facility proposal-conclude \
    --id "$credit_facility_proposal_id" \
    --approved true

  retry 60 2 wait_for_approval "$credit_facility_proposal_id"

  # Get collateral_id from pending credit facility
  cli_output=$("$LANACLI" --json credit-facility pending-get --id "$credit_facility_proposal_id")
  collateral_id=$(echo "$cli_output" | jq -r '.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1

  # Add enough collateral to activate the facility
  "$LANACLI" --json collateral update \
    --collateral-id "$collateral_id" \
    --collateral 200000000 \
    --effective "$(naive_now)"

  credit_facility_id=$credit_facility_proposal_id

  retry 60 2 wait_for_active "$credit_facility_id"
  cache_value 'credit_facility_id' "$credit_facility_id"

  # Drop collateral so CVL falls below the liquidation threshold.
  "$LANACLI" --json collateral update \
    --collateral-id "$collateral_id" \
    --collateral 100000000 \
    --effective "$(naive_now)"

  retry 60 2 wait_for_facility_to_be_under_liquidation_threshold "$credit_facility_id"

  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
    '{ id: $creditFacilityId }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"

  state=$(graphql_output '.data.creditFacility.collateralizationState')
  [[ "$state" == "UNDER_LIQUIDATION_THRESHOLD" ]] || exit 1

  liquidation_id=$(graphql_output '.data.creditFacility.liquidations[0].liquidationId')
  [[ "$liquidation_id" != "null" ]] || exit 1
  cache_value 'liquidation_id' "$liquidation_id"

  collateral_id=$(graphql_output '.data.creditFacility.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1
  cache_value 'collateral_id' "$collateral_id"
}

@test "liquidation: can send collateral out for liquidation" {
  liquidation_id=$(read_value 'liquidation_id')
  collateral_id=$(read_value 'collateral_id')

  collateral_to_send=50000000
  variables=$(
    jq -n \
      --arg collateralId "$collateral_id" \
      --argjson amount "$collateral_to_send" \
    '{
      input: {
        collateralId: $collateralId,
        amount: $amount
      }
    }'
  )
  exec_admin_graphql 'liquidation-record-collateral-sent' "$variables"

  returned_id=$(graphql_output '.data.collateralRecordSentToLiquidation.collateral.creditFacility.liquidations[0].liquidationId')
  [[ "$returned_id" == "$liquidation_id" ]] || exit 1

  sent_total=$(graphql_output '.data.collateralRecordSentToLiquidation.collateral.creditFacility.liquidations[0].sentTotal')
  [[ "$sent_total" -ge "$collateral_to_send" ]] || exit 1

  last_sent_amount=$(graphql_output '.data.collateralRecordSentToLiquidation.collateral.creditFacility.liquidations[0].sentCollateral[-1].amount')
  [[ "$last_sent_amount" -eq "$collateral_to_send" ]] || exit 1
}

@test "liquidation: can record payment received from liquidation" {
  liquidation_id=$(read_value 'liquidation_id')
  collateral_id=$(read_value 'collateral_id')

  variables=$(
    jq -n \
      --arg id "$liquidation_id" \
    '{ id: $id }'
  )
  exec_admin_graphql 'find-liquidation' "$variables"
  before_received_total=$(graphql_output '.data.liquidation.amountReceived')
  before_received_len=$(graphql_output '.data.liquidation.receivedProceeds | length')

  payment=10000000
  variables=$(
    jq -n \
      --arg collateralId "$collateral_id" \
      --argjson amount "$payment" \
    '{
      input: {
        collateralId: $collateralId,
        amount: $amount
      }
    }'
  )
  exec_admin_graphql 'liquidation-record-payment-received' "$variables"

  returned_collateral_id=$(graphql_output '.data.collateralRecordProceedsFromLiquidation.collateral.collateralId')
  [[ "$returned_collateral_id" == "$collateral_id" ]] || exit 1

  # Fetch liquidation to verify the payment was recorded
  variables=$(
    jq -n \
      --arg id "$liquidation_id" \
    '{ id: $id }'
  )
  exec_admin_graphql 'find-liquidation' "$variables"

  received_total=$(graphql_output '.data.liquidation.amountReceived')
  [[ "$received_total" -eq "$((before_received_total + payment))" ]] || exit 1

  received_len=$(graphql_output '.data.liquidation.receivedProceeds | length')
  [[ "$received_len" -eq "$((before_received_len + 1))" ]] || exit 1

  last_received_amount=$(graphql_output '.data.liquidation.receivedProceeds[-1].amount')
  [[ "$last_received_amount" -eq "$payment" ]] || exit 1
}
