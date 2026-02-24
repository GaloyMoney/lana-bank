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
  variables=$(
    jq -n \
      --arg deposit_account_id "$1" \
    '{
      input: {
        depositAccountId: $deposit_account_id,
        amount: 1000000,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"

  variables=$(
    jq -n \
      --arg deposit_account_id "$1" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $deposit_account_id,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"
  process_id=$(graphql_output .data.withdrawalInitiate.withdrawal.approvalProcessId)
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
