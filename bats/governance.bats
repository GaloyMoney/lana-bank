#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
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

  variables=$(
    jq -n \
      --arg id "$customer_id" \
    '{ id: $id }'
  )
  exec_admin_graphql 'customer' "$variables"
  deposit_account_id=$(graphql_output .data.customer.depositAccount.depositAccountId)
  cache_value "deposit_account_id" $deposit_account_id

  process_id=$(trigger_withdraw_approval_process $deposit_account_id)
  variables=$(
    jq -n \
      --arg id "$process_id" \
    '{ id: $id }'
  )
  exec_admin_graphql 'find-approval-process' "$variables"
  status=$(graphql_output .data.approvalProcess.status)
  [[ "$status" == "APPROVED" ]] || exit 1
}
