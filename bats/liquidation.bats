#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="liquidation.e2e-logs"
RUN_LOG_FILE="liquidation.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_approval() {
  variables=$(
    jq -n \
      --arg creditFacilityProposalId "$1" \
    '{ id: $creditFacilityProposalId }'
  )
  exec_admin_graphql 'find-credit-facility-proposal' "$variables"
  echo "approval | $i. $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.creditFacilityProposal.status')
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
          disbursalPolicy: "SINGLE_DISBURSAL",
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

  exec_admin_graphql 'credit-facility-proposal-create' "$variables"

  credit_facility_proposal_id=$(graphql_output '.data.creditFacilityProposalCreate.creditFacilityProposal.creditFacilityProposalId')
  [[ "$credit_facility_proposal_id" != "null" ]] || exit 1

  variables=$(
     jq -n \
      --arg creditFacilityProposalId "$credit_facility_proposal_id" \
    '{
      input: {
        creditFacilityProposalId: $creditFacilityProposalId,
        approved: true
      }
    }'
  )
  exec_admin_graphql 'credit-facility-proposal-customer-approval-conclude' "$variables"

  retry 30 2 wait_for_approval "$credit_facility_proposal_id"

  # Add enough collateral to activate the facility
  variables=$(
    jq -n \
      --arg pending_credit_facility_id "$credit_facility_proposal_id" \
      --arg effective "$(naive_now)" \
    '{
      input: {
        pendingCreditFacilityId: $pending_credit_facility_id,
        collateral: 200000000,
        effective: $effective,
      }
    }'
  )
  exec_admin_graphql 'pending-credit-facility-collateral-update' "$variables"

  credit_facility_id=$(graphql_output '.data.pendingCreditFacilityCollateralUpdate.pendingCreditFacility.pendingCreditFacilityId')
  [[ "$credit_facility_id" != "null" ]] || exit 1

  retry 30 2 wait_for_active "$credit_facility_id"
  cache_value 'credit_facility_id' "$credit_facility_id"

  # Drop collateral so CVL falls below the liquidation threshold.
  variables=$(
    jq -n \
      --arg creditFacilityId "$credit_facility_id" \
      --arg effective "$(naive_now)" \
    '{
      input: {
        creditFacilityId: $creditFacilityId,
        collateral: 100000000,
        effective: $effective,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-collateral-update' "$variables"

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
}

@test "liquidation: can send collateral out for liquidation" {
  liquidation_id=$(read_value 'liquidation_id')

  collateral_to_send=50000000
  variables=$(
    jq -n \
      --arg liquidationId "$liquidation_id" \
      --argjson amount "$collateral_to_send" \
    '{
      input: {
        liquidationId: $liquidationId,
        amount: $amount
      }
    }'
  )
  exec_admin_graphql 'liquidation-record-collateral-sent' "$variables"

  returned_id=$(graphql_output '.data.liquidationRecordCollateralSent.liquidation.liquidationId')
  [[ "$returned_id" == "$liquidation_id" ]] || exit 1

  sent_total=$(graphql_output '.data.liquidationRecordCollateralSent.liquidation.sentTotal')
  [[ "$sent_total" -ge "$collateral_to_send" ]] || exit 1

  last_sent_amount=$(graphql_output '.data.liquidationRecordCollateralSent.liquidation.sentCollateral[-1].amount')
  [[ "$last_sent_amount" -eq "$collateral_to_send" ]] || exit 1
}

@test "liquidation: can record payment received from liquidation" { 
  liquidation_id=$(read_value 'liquidation_id')
  
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
      --arg liquidationId "$liquidation_id" \
      --argjson amount "$payment" \
    '{
      input: {
        liquidationId: $liquidationId,
        amount: $amount
      }
    }'
  )
  exec_admin_graphql 'liquidation-record-payment-received' "$variables"

  returned_id=$(graphql_output '.data.liquidationRecordProceedsReceived.liquidation.liquidationId')
  [[ "$returned_id" == "$liquidation_id" ]] || exit 1

  received_total=$(graphql_output '.data.liquidationRecordProceedsReceived.liquidation.amountReceived')
  [[ "$received_total" -eq "$((before_received_total + payment))" ]] || exit 1

  received_len=$(graphql_output '.data.liquidationRecordProceedsReceived.liquidation.receivedProceeds | length')
  [[ "$received_len" -eq "$((before_received_len + 1))" ]] || exit 1

  last_received_amount=$(graphql_output '.data.liquidationRecordProceedsReceived.liquidation.receivedProceeds[-1].amount')
  [[ "$last_received_amount" -eq "$payment" ]] || exit 1
}
