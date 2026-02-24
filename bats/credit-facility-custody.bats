#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-custody.e2e-logs"
RUN_LOG_FILE="credit-facility-custody.run.e2e-logs"

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

wait_for_collateral() {
  pending_credit_facility_id=$1

  local cli_output
  cli_output=$("$LANACLI" --json credit-facility pending-get --id "$pending_credit_facility_id")
  echo "$cli_output" | jq .
  collateral=$(echo "$cli_output" | jq -r '.collateral.btcBalance')
  [[ "$collateral" -eq 1000 ]] || exit 1
}


@test "credit-facility-custody: can create with mock custodian" {
  # Setup prerequisites
  customer_id=$(create_customer)

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")

  facility=100000
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility proposal-create \
    --customer-id "$customer_id" \
    --facility-amount "$facility" \
    --custodian-id "00000000-0000-0000-0000-000000000000" \
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

  cache_value 'credit_facility_proposal_id' "$credit_facility_proposal_id"

  "$LANACLI" --json credit-facility proposal-conclude \
    --id "$credit_facility_proposal_id" \
    --approved true

  retry 30 2 wait_for_approval "$credit_facility_proposal_id"

  cli_output=$("$LANACLI" --json credit-facility pending-get --id "$credit_facility_proposal_id")
  echo "$cli_output" | jq .

  address=$(echo "$cli_output" | jq -r '.wallet.address')
  [[ "$address" == "bt1qaddressmock" ]] || exit 1
}

@test "credit-facility-custody: cannot update manually collateral with a custodian" {
  pending_credit_facility_id=$(read_value 'credit_facility_proposal_id')

  # Get collateral_id from pending credit facility
  local cli_output
  cli_output=$("$LANACLI" --json credit-facility pending-get --id "$pending_credit_facility_id")
  collateral_id=$(echo "$cli_output" | jq -r '.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1

  cli_output=$("$LANACLI" --json collateral update \
    --collateral-id "$collateral_id" \
    --collateral 50000000 \
    --effective "$(naive_now)" 2>&1 || true)
  [[ "$cli_output" =~ "ManualUpdateError" ]] || exit 1
}

@test "credit-facility-custody: can update collateral by a custodian" {
  pending_credit_facility_id=$(read_value 'credit_facility_proposal_id')

  variables=$(
    jq -n \
      --arg pending_credit_facility_id "$pending_credit_facility_id" \
      '{ id: $pending_credit_facility_id }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  collateral=$(graphql_output '.data.creditFacility.balance.collateral.btcBalance')
  [[ "$collateral" -eq 0 ]] || exit 1

  # external wallet ID 123 is hard coded in mock custodian
  curl -s -X POST --json '{"wallet": "123", "balance": 1000}' http://localhost:5253/webhook/custodian/mock

  retry 30 2 wait_for_collateral "$pending_credit_facility_id"
}
