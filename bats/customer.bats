#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
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
  [[ "$customer_id" != "null" ]] || exit 1

  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer-audit-log' "$variables"
  echo $(graphql_output) | jq .

  audit_entries=$(graphql_output '.data.customer.audit')
  [[ "$audit_entries" != "null" ]] || exit 1

  action=$(graphql_output '.data.customer.audit[0].action')
  [[ "$action" == "customer:create" ]] || exit 1

  authorized=$(graphql_output '.data.customer.audit[0].authorized')
  [[ "$authorized" == "true" ]] || exit 1
}

@test "customer: can deposit" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1
  echo $(graphql_output) | jq .

  usd_balance=$(graphql_output '.data.depositRecord.deposit.customer.balance.checking.settled')
  [[ "$usd_balance" == "150000" ]] || exit 1
}

@test "customer: withdraw can be cancelled" {
  customer_id=$(read_value 'customer_id')

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
        id: $customerId
    }'
  )

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalInitiate.withdrawal.status')
  [[ "$status" == "INITIATED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "150000" ]] || exit 1

  assert_accounts_balanced

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
  settled_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "150000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  assert_accounts_balanced
}

@test "customer: can withdraw" {
  customer_id=$(read_value 'customer_id')

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalInitiate.withdrawal.status')
  [[ "$status" == "INITIATED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "150000" ]] || exit 1

  assert_accounts_balanced

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

  echo $(graphql_output)
  withdrawal_id=$(graphql_output '.data.withdrawalConfirm.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalConfirm.withdrawal.status')
  [[ "$status" == "CONFIRMED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  assert_accounts_balanced
}
