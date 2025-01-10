#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
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

@test "customer: unauthorized" {
  cache_value "alice" "invalid-token"
  exec_graphql 'alice' 'me'
  error_code=$(graphql_output '.error.code')
  [[ "$error_code" == 401 ]] || exit 1

  error_status=$(graphql_output '.error.status')
  [[ "$error_status" == "Unauthorized" ]] || exit 1
}

@test "customer: can create a customer" {
  customer_email=$(generate_email)
  telegramId=$(generate_email)

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    --arg telegramId "$telegramId" \
    '{
      input: {
        email: $email,
        telegramId: $telegramId
      }
    }'
  )
  
  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  echo $(graphql_output) | jq .
  [[ "$customer_id" != "null" ]] || exit 1

  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer-audit-log' "$variables"
  echo $(graphql_output) | jq .
}

@test "customer: can deposit" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  variables=$(
    jq -n \
      --arg id "$customer_id" \
    '{ id: $id }'
  )

  exec_admin_graphql 'customer' "$variables"
  echo $(graphql_output) | jq .
  deposit_account_id=$(graphql_output .data.customer.depositAccount.depositAccountId)
  cache_value "deposit_account_id" $deposit_account_id

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 150000,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1

  # usd_balance=$(graphql_output '.data.depositRecord.deposit.customer.depositAccount.balance.checking.settled')
  # [[ "$usd_balance" == "150000" ]] || exit 1
}

@test "customer: withdraw can be cancelled" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  echo $(graphql_output) 
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalInitiate.withdrawal.status')
  # PENDING_APPROVAL is skipped due to status being updated on read
  [[ "$status" == "PENDING_CONFIRMATION" ]] || exit 1
  # settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  # [[ "$settled_usd_balance" == "0" ]] || exit 1
  # pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  # [[ "$pending_usd_balance" == "150000" ]] || exit 1

  # assert_accounts_balanced

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
    '{
      input: {
        withdrawalId: $withdrawalId
      }
    }'
  )
  exec_admin_graphql 'withdrawal-cancel' "$variables"
  echo $(graphql_output) | jq .

  withdrawal_id=$(graphql_output '.data.withdrawalCancel.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalCancel.withdrawal.status')
  [[ "$status" == "CANCELLED" ]] || exit 1
  # settled_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.settled')
  # [[ "$settled_usd_balance" == "150000" ]] || exit 1
  # pending_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.pending')
  # [[ "$pending_usd_balance" == "0" ]] || exit 1

  # assert_accounts_balanced
}

@test "customer: can withdraw" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  # settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  # [[ "$settled_usd_balance" == "0" ]] || exit 1
  # pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  # [[ "$pending_usd_balance" == "150000" ]] || exit 1

  # assert_accounts_balanced

  retry 5 1 wait_for_approval $withdrawal_id

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

  echo $(graphql_output) | jq . 
  withdrawal_id=$(graphql_output '.data.withdrawalConfirm.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalConfirm.withdrawal.status')
  [[ "$status" == "CONFIRMED" ]] || exit 1
  # settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.settled')
  # [[ "$settled_usd_balance" == "0" ]] || exit 1
  # pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.pending')
  # [[ "$pending_usd_balance" == "0" ]] || exit 1

  # assert_accounts_balanced
}
