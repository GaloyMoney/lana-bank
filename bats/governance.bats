#!/usr/bin/env bats

load "helpers"

setup_file() {
  export LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

trigger_withdraw_approval_process() {
  local cli_output
  cli_output=$("$LANACLI" --json deposit-account record-deposit \
    --deposit-account-id "$1" \
    --amount 1000000)

  cli_output=$("$LANACLI" --json deposit-account initiate-withdrawal \
    --deposit-account-id "$1" \
    --amount 150000 \
    --reference "withdrawal-ref-$(date +%s%N)")
  process_id=$(echo "$cli_output" | jq -r '.approvalProcessId')
  [[ "$process_id" != "null" ]] || exit 1
  echo $process_id
}

@test "governance: auto-approve" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")
  cache_value "deposit_account_id" $deposit_account_id

  process_id=$(trigger_withdraw_approval_process $deposit_account_id)
  local cli_output
  cli_output=$("$LANACLI" --json approval-process get --id "$process_id")
  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "APPROVED" ]] || exit 1
}
