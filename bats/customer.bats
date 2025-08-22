#!/usr/bin/env bats

load "helpers"

RUN_LOG_FILE="customer.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
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
  echo "withdrawal | $i. $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.withdrawal.status')
  [[ "$status" == "PENDING_CONFIRMATION" ]] || return 1
}

@test "customer: can create a customer" {
  customer_email=$(generate_email)
  telegramId=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    --arg telegramId "$telegramId" \
    --arg customerType "$customer_type" \
    '{
      input: {
        email: $email,
        telegramId: $telegramId,
        customerType: $customerType
      }
    }'
  )
  
  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1
  
  # Verify customerType in response
  response_customer_type=$(graphql_output .data.customerCreate.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1

  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer-audit-log' "$variables"
  audit_nodes_count=$(graphql_output '.data.audit.nodes | length')
  [[ "$audit_nodes_count" -gt 0 ]] || exit 1
}

@test "customer: can login" {
  skip # does not work on concourse

  customer_email=$(generate_email)
  telegramId=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    --arg telegramId "$telegramId" \
    --arg customerType "$customer_type" \
    '{
      input: {
        email: $email,
        telegramId: $telegramId,
        customerType: $customerType
      }
    }'
  )

  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1

  retry 80 1 wait_for_checking_account "$customer_id"

  login_customer $customer_email
  exec_customer_graphql $customer_email 'me'
  echo $(graphql_output) | jq .
  [[ "$(graphql_output .data.me.customer.customerId)" == "$customer_id" ]] || exit 1
  
  response_customer_type=$(graphql_output .data.me.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1
}

@test "customer: can deposit" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  retry 80 1 wait_for_checking_account "$customer_id"

  variables=$(
    jq -n \
      --arg id "$customer_id" \
    '{ id: $id }'
  )

  exec_admin_graphql 'customer' "$variables"
  echo "deposit | $i. $(graphql_output)" >> $RUN_LOG_FILE
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
        amount: 120000,
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
  # [[ "$pending_usd_balance" == "120000" ]] || exit 1

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

@test "customer: confirmed withdrawal can be reverted" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 100,
        reference: ("void-withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"
  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')

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

  status=$(graphql_output '.data.withdrawalConfirm.withdrawal.status')
  [[ "$status" == "CONFIRMED" ]] || exit 1

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
    '{
      input: {
        withdrawalId: $withdrawalId
      }
    }'
  )
  exec_admin_graphql 'withdrawal-revert' "$variables"
  status=$(graphql_output '.data.withdrawalRevert.withdrawal.status')
  [[ "$status" == "REVERTED" ]] || exit 1
}

@test "customer: deposit account can be frozen" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId
      }
    }'
  )
  exec_admin_graphql 'deposit-account-freeze' "$variables"
  echo $(graphql_output)

  status=$(graphql_output '.data.depositAccountFreeze.account.status')
  [[ "$status" == "FROZEN" ]] || exit 1

  balance=$(graphql_output '.data.depositAccountFreeze.account.balance.settled')
  [[ "$balance" == 0 ]] || exit 1
}

@test "customer: cannot withdraw from frozen account" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 100,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "DepositAccountFrozen" ]] || exit 1
}
