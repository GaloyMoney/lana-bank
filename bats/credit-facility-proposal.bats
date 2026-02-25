#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-proposal.e2e-logs"
RUN_LOG_FILE="credit-facility-proposal.run.e2e-logs"

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
  echo "withdrawal | $i. $cli_output" >> $RUN_LOG_FILE
  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "APPROVED" ]] || return 1
}

wait_for_active() {
  credit_facility_id=$1

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")

  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "ACTIVE" ]] || exit 1
}

wait_for_disbursal() {
  credit_facility_id=$1
  disbursal_id=$2

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")
  echo "disbursal | $i. $cli_output" >> $RUN_LOG_FILE
  num_disbursals=$(
    echo "$cli_output" | jq \
      --arg disbursal_id "$disbursal_id" \
      '[
        .disbursals[]
        | select(.disbursalId == $disbursal_id)
        ] | length'
  )
  [[ "$num_disbursals" -eq "1" ]]
}

wait_for_accruals() {
  expected_num_accruals=$1
  credit_facility_id=$2

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")
  echo "accrual | $i. $cli_output" >> $RUN_LOG_FILE
  num_accruals=$(
    echo "$cli_output" | jq '[
      .history[]
      | select(.__typename == "CreditFacilityInterestAccrued")
      ] | length'
  )

  [[ "$num_accruals" == "$expected_num_accruals" ]] || exit 1
}

wait_for_dashboard_disbursed() {
  before=$1
  disbursed_amount=$2

  expected_after="$(( $before + $disbursed_amount ))"

  local cli_output
  cli_output=$("$LANACLI" --json dashboard get)
  after=$(echo "$cli_output" | jq -r '.totalDisbursed')

  [[ "$after" -eq "$expected_after" ]] || exit 1
}

wait_for_payment() {
  credit_facility_id=$1
  outstanding_before=$2
  payment_amount=$3

  expected_after="$(( $outstanding_before - $payment_amount ))"

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")

  balance=$(echo "$cli_output" | jq -r '.balance')
  after=$(echo $balance | jq -r '.outstanding.usdBalance')

  [[ "$after" -eq "$expected_after" ]] || exit 1
}

wait_for_dashboard_payment() {
  before=$1
  payment_amount=$2

  expected_after="$(( $before - $payment_amount ))"

  local cli_output
  cli_output=$("$LANACLI" --json dashboard get)
  after=$(echo "$cli_output" | jq -r '.totalDisbursed')

  [[ "$after" -eq "$expected_after" ]] || exit 1
}

ymd() {
  local date_value
  read -r date_value
  echo $date_value | cut -d 'T' -f1 | tr -d '-'
}

@test "credit-facility-proposal: can create" {
  # Setup prerequisites
  customer_id=$(create_customer)

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")

  facility=100000
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility proposal-create \
    --customer-id "$customer_id" \
    --facility-amount "$facility" \
    --annual-rate 12 \
    --accrual-interval END_OF_DAY \
    --accrual-cycle-interval END_OF_MONTH \
    --one-time-fee-rate 5 \
    --disbursal-policy MULTIPLE_DISBURSAL \
    --duration-months 3 \
    --initial-cvl 140 \
    --margin-call-cvl 125 \
    --liquidation-cvl 105 \
    --interest-due-days 0 \
    --overdue-days 50 \
    --liquidation-days 360)

  credit_facility_proposal_id=$(echo "$cli_output" | jq -r '.creditFacilityProposalId')
  [[ "$credit_facility_proposal_id" != "null" && -n "$credit_facility_proposal_id" ]] || exit 1

  cache_value 'credit_facility_proposal_id' "$credit_facility_proposal_id"

  "$LANACLI" --json credit-facility proposal-conclude \
    --id "$credit_facility_proposal_id" \
    --approved true
}

@test "pending-credit-facility: can update collateral" {
  retry 30 2 wait_for_approval "$(read_value 'credit_facility_proposal_id')"

  pending_credit_facility_id=$(read_value 'credit_facility_proposal_id')

  # Get collateral_id from pending credit facility
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility pending-get --id "$pending_credit_facility_id")
  collateral_id=$(echo "$cli_output" | jq -r '.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1

  cli_output=$("$LANACLI" --json collateral update \
    --collateral-id "$collateral_id" \
    --collateral 50000000 \
    --effective "$(naive_now)")
  result_collateral_id=$(echo "$cli_output" | jq -r '.collateralId')
  [[ "$result_collateral_id" != "null" ]] || exit 1

  credit_facility_id=$pending_credit_facility_id

  retry 30 2 wait_for_active "$credit_facility_id"

  cache_value 'credit_facility_id' "$credit_facility_id"
}

@test "credit-facility: can initiate disbursal" {
  credit_facility_id=$(read_value 'credit_facility_id')

  local dashboard_output
  dashboard_output=$("$LANACLI" --json dashboard get)
  disbursed_before=$(echo "$dashboard_output" | jq -r '.totalDisbursed')

  amount=50000
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility disbursal-initiate \
    --credit-facility-id "$credit_facility_id" \
    --amount "$amount")
  disbursal_id=$(echo "$cli_output" | jq -r '.disbursalId')
  [[ "$disbursal_id" != "null" && -n "$disbursal_id" ]] || exit 1

  retry 30 2 wait_for_disbursal "$credit_facility_id" "$disbursal_id"
  retry 30 2 wait_for_dashboard_disbursed "$disbursed_before" "$amount"
}

@test "credit-facility: records accruals" {

  credit_facility_id=$(read_value 'credit_facility_id')
  retry 30 2 wait_for_accruals 4 "$credit_facility_id"

  cat_logs | grep "interest accrual cycles completed for.*$credit_facility_id" || exit 1

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")
  num_accruals=$(
    echo "$cli_output" | jq '[
      .history[]
      | select(.__typename == "CreditFacilityInterestAccrued")
      ] | length'
  )
  [[ "$num_accruals" -eq "4" ]] || exit 1

  # assert_accounts_balanced
}

@test "credit-facility: record payment" {
  credit_facility_id=$(read_value 'credit_facility_id')

  local dashboard_output
  dashboard_output=$("$LANACLI" --json dashboard get)
  disbursed_before=$(echo "$dashboard_output" | jq -r '.totalDisbursed')

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")
  balance=$(echo "$cli_output" | jq -c '.balance')

  interest=$(echo $balance | jq -r '.interest.total.usdBalance')
  interest_outstanding=$(echo $balance | jq -r '.interest.outstanding.usdBalance')
  [[ "$interest" -eq "$interest_outstanding" ]] || exit 1

  disbursed=$(echo $balance | jq -r '.disbursed.total.usdBalance')
  disbursed_outstanding=$(echo $balance | jq -r '.disbursed.outstanding.usdBalance')
  [[ "$disbursed" -eq "$disbursed_outstanding" ]] || exit 1

  total_outstanding=$(echo $balance | jq -r '.outstanding.usdBalance')
  [[ "$total_outstanding" -eq "$(( $interest_outstanding + $disbursed_outstanding ))" ]] || exit 1

  payments_unapplied=$(echo $balance | jq -r '.paymentsUnapplied.usdBalance')
  [[ "$payments_unapplied" != "null" ]] || exit 1
  [[ "$payments_unapplied" -eq 0 ]] || exit 1

  disbursed_payment=25000
  amount="$(( $disbursed_payment + $interest_outstanding ))"
  local payment_output
  payment_output=$("$LANACLI" --json credit-facility partial-payment-record \
    --credit-facility-id "$credit_facility_id" \
    --amount "$amount")
  balance_after_payment=$(echo "$payment_output" | jq -c '.balance')
  payments_unapplied_after=$(echo $balance_after_payment | jq -r '.paymentsUnapplied.usdBalance')
  [[ "$payments_unapplied_after" -gt 0 ]] || exit 1

  retry 30 2 wait_for_payment "$credit_facility_id" "$total_outstanding" "$amount"

  cli_output=$("$LANACLI" --json credit-facility find --id "$credit_facility_id")
  updated_balance=$(echo "$cli_output" | jq -c '.balance')

  updated_interest=$(echo $updated_balance | jq -r '.interest.total.usdBalance')
  [[ "$interest" -eq "$updated_interest" ]] || exit 1
  updated_disbursed=$(echo $updated_balance | jq -r '.disbursed.total.usdBalance')
  [[ "$disbursed" -eq "$updated_disbursed" ]] || exit 1

  updated_total_outstanding=$(echo $updated_balance | jq -r '.outstanding.usdBalance')
  [[ "$updated_total_outstanding" -lt "$total_outstanding" ]] || exit 1

  updated_interest_outstanding=$(echo $updated_balance | jq -r '.interest.outstanding.usdBalance')
  [[ "$updated_interest_outstanding" -eq "0" ]] || exit 1

  retry 30 2 wait_for_dashboard_payment "$disbursed_before" "$disbursed_payment"

  # assert_accounts_balanced
}
