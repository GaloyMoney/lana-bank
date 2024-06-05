#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "user: can create a user" {
  username=$(random_uuid)
  variables=$(
    jq -n \
      --arg username "$username" \
    '{
      input: {
        bitfinexUsername: $username,
      }
    }'
  )
  exec_graphql 'user-create' "$variables"
  user=$(graphql_output '.data.userCreate.user.bitfinexUsername')
  [[ "$user" == "$username" ]] || exit 1;

  sats=$(graphql_output '.data.userCreate.user.balance.unallocatedCollateral.settled.btcBalance')
  [[ "$sats" == "0" ]] || exit 1;

  user_id=$(graphql_output '.data.userCreate.user.userId')
  cache_value 'user.id' "$user_id"
}

@test "user: can pledge unallocated collateral" {
  user_id=$(read_value 'user.id')
  variables=$(
    jq -n \
      --arg userId "$user_id" \
    '{
      input: {
        userId: $userId,
        amount: 1000000000,
        reference: ("pledge-" + $userId)
      }
    }'
  )
  exec_admin_graphql 'pledge-unallocated-collateral' "$variables"
  sats=$(graphql_output '.data.userPledgeCollateral.user.balance.unallocatedCollateral.settled.btcBalance')
  [[ "$sats" == "1000000000" ]] || exit 1;
}

@test "user: can deposit" {
  username=$(random_uuid)
  variables=$(
    jq -n \
      --arg username "$username" \
    '{
      input: {
        bitfinexUsername: $username,
      }
    }'
  )
  exec_graphql 'user-create' "$variables"
  user=$(graphql_output '.data.userCreate.user.bitfinexUsername')
  [[ "$user" == "$username" ]] || exit 1;
  user_id=$(graphql_output '.data.userCreate.user.userId')

  variables=$(
    jq -n \
      --arg userId "$user_id" \
    '{
      input: {
        userId: $userId,
        amount: 50000000,
        reference: ("deposit-" + $userId)
      }
    }'
  )
  exec_admin_graphql 'deposit-checking' "$variables"
  cents=$(graphql_output '.data.userDeposit.user.balance.checking.settled.usdBalance')
  [[ "$cents" == "50000000" ]] || exit 1;
}

@test "user: can withdraw" {
  user_id=$(read_value 'user.id')
  variables=$(
    jq -n \
    --arg userId "$user_id" \
    '{
      input: {
        userId: $userId,
      }
    }'
  )
  exec_graphql 'fixed-term-loan-create' "$variables"
  id=$(graphql_output '.data.fixedTermLoanCreate.loan.loanId')
  [[ "$id" != null ]] || exit 1
  variables=$(
    jq -n \
      --arg loanId "$id" \
    '{
      input: {
        loanId: $loanId,
        collateral: 400000000,
        principal: 25000000,
      }
    }'
  )
  exec_graphql 'approve-loan' "$variables"
  loan_id=$(graphql_output '.data.fixedTermLoanApprove.loan.loanId')
  [[ "$id" == "$loan_id" ]] || exit 1

  variables=$(
    jq -n \
    --arg userId "$user_id" \
    '{ id: $userId }'
  )
  exec_graphql 'find-user' "$variables"
  checking_balance=$(graphql_output '.data.user.balance.checking.settled.usdBalance')
  [[ "$checking_balance" == "25000000" ]] || exit 1

  variables=$(
    jq -n \
      --arg userId "$user_id" \
    '{
      input: {
        userId: $userId,
        amount: 1500000,
        destination: "tron-address",
        reference: ("initiate_withdrawal-" + $userId)
      }
    }'
  )
  exec_graphql 'initiate-withdrawal' "$variables"
  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
    --arg userId "$user_id" \
    '{ id: $userId }'
  )
  exec_graphql 'find-user' "$variables"
  checking_balance=$(graphql_output '.data.user.balance.checking.settled.usdBalance')
  [[ "$checking_balance" == "23500000" ]] || exit 1
  encumbered_checking_balance=$(graphql_output '.data.user.balance.checking.pending.usdBalance')
  [[ "$encumbered_checking_balance" == "1500000" ]] || exit 1

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
    '{
      input: {
        withdrawalId: $withdrawalId,
        reference: ("settle_withdrawal-" + $withdrawalId)
      }
    }'
  )
  exec_admin_graphql 'settle-withdrawal' "$variables"
  withdrawal_id_on_settle=$(graphql_output '.data.withdrawalSettle.withdrawal.withdrawalId')
  [[ "$withdrawal_id_on_settle" == "$withdrawal_id" ]] || exit 1

  variables=$(
    jq -n \
    --arg userId "$user_id" \
    '{ id: $userId }'
  )
  exec_graphql 'find-user' "$variables"
  checking_balance=$(graphql_output '.data.user.balance.checking.settled.usdBalance')
  [[ "$checking_balance" == "23500000" ]] || exit 1
  encumbered_checking_balance=$(graphql_output '.data.user.balance.checking.pending.usdBalance')
  [[ "$encumbered_checking_balance" == "0" ]] || exit 1
}
