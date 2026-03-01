#!/usr/bin/env bats

load "helpers"

RUN_LOG_FILE="checking-account.run.e2e-logs"

setup_file() {
  export LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
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

@test "checking-account: can deposit" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")
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

  usd_balance_settled=$(graphql_output '.data.depositRecord.deposit.account.balance.settled')
  usd_balance_pending=$(graphql_output '.data.depositRecord.deposit.account.balance.pending')
  [[ "$usd_balance_settled" == "150000" ]] || exit 1
  [[ "$usd_balance_pending" == "0" ]] || exit 1
}

@test "checking-account: withdraw can be cancelled" {
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
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "150000" ]] || exit 1

  # assert_accounts_balanced

  retry 60 2 wait_for_approval $withdrawal_id

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
  settled_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "150000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  # assert_accounts_balanced
}

@test "checking-account: can withdraw" {
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
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "30000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "120000" ]] || exit 1

  # assert_accounts_balanced

  retry 60 2 wait_for_approval $withdrawal_id

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
  settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "30000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  # assert_accounts_balanced
}

@test "checking-account: confirmed withdrawal can be reverted" {
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

  retry 60 2 wait_for_approval $withdrawal_id

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

@test "checking-account: deposit account can be frozen" {
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

@test "checking-account: cannot withdraw from frozen account" {
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

@test "checking-account: deposit account can be unfrozen" {
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
  exec_admin_graphql 'deposit-account-unfreeze' "$variables"

  status=$(graphql_output '.data.depositAccountUnfreeze.account.status')
  [[ "$status" == "ACTIVE" ]] || exit 1
}

@test "checking-account: can deposit and withdraw after unfreeze" {
  deposit_account_id=$(read_value 'deposit_account_id')

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
      --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 40000,
        reference: ("deposit-after-unfreeze-" + $date)
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"

  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
      --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 20000,
        reference: ("withdraw-after-unfreeze-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  retry 60 2 wait_for_approval $withdrawal_id

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

  settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "50000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1
}

@test "checking-account: can not close a deposit account with non-zero balance" {
  deposit_account_id=$(read_value 'deposit_account_id')

  # close account with settled balance 50000 (from previous test)
  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId
      }
    }'
  )
  exec_admin_graphql 'deposit-account-close' "$variables"
  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "BalanceIsNotZero" ]] || exit 1
}

@test "checking-account: can not close a frozen account with zero balance" {
  deposit_account_id=$(read_value 'deposit_account_id')

  # withdraw the total balance (of 50000)
  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
      --arg date "$(date +%s%N)" \
    '{
      input: {
        depositAccountId: $depositAccountId,
        amount: 50000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  retry 60 2 wait_for_approval $withdrawal_id

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

  settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.account.balance.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  # freeze the empty account
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

  status=$(graphql_output '.data.depositAccountFreeze.account.status')
  [[ "$status" == "FROZEN" ]] || exit 1

  # close the frozen account
  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId
      }
    }'
  )
  exec_admin_graphql 'deposit-account-close' "$variables"

  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "CannotUpdateFrozenAccount" ]] || exit 1
}

@test "checking-account: can close account" {
  deposit_account_id=$(read_value 'deposit_account_id')

  # unfreeze the frozen account
  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId
      }
    }'
  )
  exec_admin_graphql 'deposit-account-unfreeze' "$variables"

  status=$(graphql_output '.data.depositAccountUnfreeze.account.status')
  [[ "$status" == "ACTIVE" ]] || exit 1

  # close the unfrozen(active) account
  variables=$(
    jq -n \
      --arg depositAccountId "$deposit_account_id" \
    '{
      input: {
        depositAccountId: $depositAccountId
      }
    }'
  )
  exec_admin_graphql 'deposit-account-close' "$variables"

  status=$(graphql_output '.data.depositAccountClose.account.status')
  [[ "$status" == "CLOSED" ]] || exit 1
}
