#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="loan-lifecycle.e2e-logs"
RUN_LOG_FILE="loan-lifecycle.run.e2e-logs"

setup_file() {
  export LANA_DOMAIN_CONFIG_ALLOW_MANUAL_CONVERSION=true
  export LANA_DOMAIN_CONFIG_CREDIT_ACCRUAL_PRECISION_DP=6
  export LANA_DOMAIN_CONFIG_CREDIT_ACCRUAL_ROUNDING_STRATEGY=half_up
  start_server
  login_superadmin
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"

  manual_custodian_id=$(get_or_create_manual_custodian)
  cache_value 'lc_manual_custodian_id' "$manual_custodian_id"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

advance_one_day() {
  exec_admin_graphql 'advance-time'
  current_date=$(graphql_output '.data.timeAdvanceToNextEndOfDay.time.currentDate')
  echo "$current_date"
}

wait_for_lc_approval() {
  variables=$(jq -n --arg creditFacilityProposalId "$1" '{ id: $creditFacilityProposalId }')
  exec_admin_graphql 'find-credit-facility-proposal' "$variables"
  echo "loan-lifecycle approval | $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.creditFacilityProposal.status')
  [[ "$status" == "APPROVED" ]] || return 1
}

wait_for_lc_active() {
  variables=$(jq -n --arg creditFacilityId "$1" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  status=$(graphql_output '.data.creditFacility.status')
  [[ "$status" == "ACTIVE" ]] || return 1
}

wait_for_lc_disbursal() {
  local credit_facility_id=$1
  local disbursal_id=$2

  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  echo "loan-lifecycle disbursal | $(graphql_output)" >> $RUN_LOG_FILE
  num_disbursals=$(
    graphql_output \
      --arg disbursal_id "$disbursal_id" \
      '[.data.creditFacility.disbursals[] | select(.id == $disbursal_id)] | length'
  )
  [[ "$num_disbursals" -eq "1" ]]
}

wait_for_interest_accrued() {
  local credit_facility_id=$1

  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  echo "loan-lifecycle interest check | $(graphql_output '.data.creditFacility.balance')" >> $RUN_LOG_FILE
  interest=$(graphql_output '.data.creditFacility.balance.interest.total.usdBalance')
  [[ "$interest" != "null" && "$interest" -gt 0 ]] || return 1
}

wait_for_interest_cleared() {
  local credit_facility_id=$1

  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  interest_outstanding=$(graphql_output '.data.creditFacility.balance.interest.outstanding.usdBalance')
  [[ "$interest_outstanding" -eq 0 ]] || return 1
}

wait_for_lc_matured() {
  local credit_facility_id=$1

  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  status=$(graphql_output '.data.creditFacility.status')
  [[ "$status" == "MATURED" ]] || return 1
}

wait_for_outstanding_zero() {
  local credit_facility_id=$1

  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  outstanding=$(graphql_output '.data.creditFacility.balance.outstanding.usdBalance')
  [[ "$outstanding" -eq 0 ]] || return 1
}

# ===== Tests =====

@test "loan-lifecycle: setup customer with deposit" {
  customer_id=$(create_customer)
  cache_value 'lc_customer_id' "$customer_id"

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")
  cache_value 'lc_deposit_account_id' "$deposit_account_id"

  # Record a deposit so the customer has funds for payments
  variables=$(jq -n \
    --arg depositAccountId "$deposit_account_id" \
    '{ input: { depositAccountId: $depositAccountId, amount: 10000000 } }')
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1
}

@test "loan-lifecycle: create and approve facility proposal" {
  customer_id=$(read_value 'lc_customer_id')
  custodian_id=$(read_value 'lc_manual_custodian_id')

  # Create proposal: $1000 facility, 12% annual rate, 3 month duration
  variables=$(jq -n \
    --arg customerId "$customer_id" \
    --arg custodianId "$custodian_id" \
    '{
      input: {
        customerId: $customerId,
        facility: 100000,
        custodianId: $custodianId,
        terms: {
          annualRate: "12",
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
          disbursalPolicy: "MULTIPLE_DISBURSAL",
          oneTimeFeeRate: "0",
          duration: { period: "MONTHS", units: 3 },
          interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
          obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
          obligationLiquidationDurationFromDue: { period: "DAYS", units: 360 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140"
        }
      }
    }')
  exec_admin_graphql 'credit-facility-proposal-create' "$variables"
  proposal_id=$(graphql_output '.data.creditFacilityProposalCreate.creditFacilityProposal.creditFacilityProposalId')
  [[ "$proposal_id" != "null" ]] || exit 1
  cache_value 'lc_proposal_id' "$proposal_id"

  # Customer approves
  variables=$(jq -n \
    --arg creditFacilityProposalId "$proposal_id" \
    '{ input: { creditFacilityProposalId: $creditFacilityProposalId, approved: true } }')
  exec_admin_graphql 'credit-facility-proposal-customer-approval-conclude' "$variables"

  # Wait for governance approval
  retry 30 2 wait_for_lc_approval "$proposal_id"
}

@test "loan-lifecycle: collateralize and activate" {
  proposal_id=$(read_value 'lc_proposal_id')

  # Get collateral_id from pending facility
  variables=$(jq -n --arg id "$proposal_id" '{ id: $id }')
  exec_admin_graphql 'find-pending-credit-facility' "$variables"
  collateral_id=$(graphql_output '.data.pendingCreditFacility.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1

  # Get current clock date for effective date
  exec_admin_graphql 'time'
  current_date=$(graphql_output '.data.time.currentDate')

  # Update collateral (50M sats = 0.5 BTC, well above 140% CVL for $1000)
  variables=$(jq -n \
    --arg collateralId "$collateral_id" \
    --arg effective "$current_date" \
    '{ input: { collateralId: $collateralId, collateral: 50000000, effective: $effective } }')
  exec_admin_graphql 'collateral-update' "$variables"

  credit_facility_id="$proposal_id"
  retry 30 2 wait_for_lc_active "$credit_facility_id"
  cache_value 'lc_credit_facility_id' "$credit_facility_id"
}

@test "loan-lifecycle: disburse full facility amount" {
  credit_facility_id=$(read_value 'lc_credit_facility_id')

  # Disburse the full $1000
  variables=$(jq -n \
    --arg creditFacilityId "$credit_facility_id" \
    '{ input: { creditFacilityId: $creditFacilityId, amount: 100000 } }')
  exec_admin_graphql 'credit-facility-disbursal-initiate' "$variables"
  disbursal_id=$(graphql_output '.data.creditFacilityDisbursalInitiate.disbursal.id')
  [[ "$disbursal_id" != "null" ]] || exit 1

  retry 30 2 wait_for_lc_disbursal "$credit_facility_id" "$disbursal_id"

  # Verify disbursed balance
  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  disbursed=$(graphql_output '.data.creditFacility.balance.disbursed.total.usdBalance')
  [[ "$disbursed" -eq 100000 ]] || exit 1
}

@test "loan-lifecycle: advance time and verify interest accrual" {
  credit_facility_id=$(read_value 'lc_credit_facility_id')

  # Advance 32 days to cross end-of-month and trigger interest accrual cycle
  for i in $(seq 1 32); do
    advance_one_day
  done

  # Wait for interest to accrue (jobs run asynchronously after clock advance)
  retry 30 2 wait_for_interest_accrued "$credit_facility_id"

  # Verify interest has accrued with reasonable amount
  # With $1000 at 12% annual, ~30 days of interest should be roughly $10 (1000 cents)
  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  interest=$(graphql_output '.data.creditFacility.balance.interest.total.usdBalance')
  echo "Accrued interest after ~1 month: $interest cents" >> $RUN_LOG_FILE
  [[ "$interest" -gt 500 ]] || exit 1
  [[ "$interest" -lt 1500 ]] || exit 1
}

@test "loan-lifecycle: pay interest obligation" {
  credit_facility_id=$(read_value 'lc_credit_facility_id')

  # Get current outstanding interest
  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  interest_outstanding=$(graphql_output '.data.creditFacility.balance.interest.outstanding.usdBalance')
  echo "Interest outstanding to pay: $interest_outstanding" >> $RUN_LOG_FILE

  # Pay the interest
  variables=$(jq -n \
    --arg creditFacilityId "$credit_facility_id" \
    --argjson amount "$interest_outstanding" \
    '{ input: { creditFacilityId: $creditFacilityId, amount: $amount } }')
  exec_admin_graphql 'credit-facility-partial-payment-record' "$variables"

  # Wait for interest outstanding to clear
  retry 30 2 wait_for_interest_cleared "$credit_facility_id"
}

@test "loan-lifecycle: advance to maturity" {
  credit_facility_id=$(read_value 'lc_credit_facility_id')

  # Advance remaining ~65 days to pass the 3-month maturity.
  # Advance day-by-day so each EndOfDay event cascades properly.
  for i in $(seq 1 65); do
    advance_one_day
  done

  # Wait for maturity processing to complete
  retry 60 2 wait_for_lc_matured "$credit_facility_id"
}

@test "loan-lifecycle: pay off remaining and verify zero outstanding" {
  credit_facility_id=$(read_value 'lc_credit_facility_id')

  # Get total outstanding (principal + any remaining interest)
  variables=$(jq -n --arg creditFacilityId "$credit_facility_id" '{ id: $creditFacilityId }')
  exec_admin_graphql 'find-credit-facility' "$variables"
  total_outstanding=$(graphql_output '.data.creditFacility.balance.outstanding.usdBalance')
  echo "Total outstanding at maturity: $total_outstanding" >> $RUN_LOG_FILE
  [[ "$total_outstanding" -gt 0 ]] || exit 1

  # Pay off everything
  variables=$(jq -n \
    --arg creditFacilityId "$credit_facility_id" \
    --argjson amount "$total_outstanding" \
    '{ input: { creditFacilityId: $creditFacilityId, amount: $amount } }')
  exec_admin_graphql 'credit-facility-partial-payment-record' "$variables"

  # Verify outstanding reaches zero
  retry 30 2 wait_for_outstanding_zero "$credit_facility_id"
}
