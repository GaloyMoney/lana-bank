#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "user: unauthorized" {
  cache_value "alice" "invalid-token"
  exec_graphql 'alice' 'me'
  error_code=$(graphql_output '.error.code')
  [[ "$error_code" == 401 ]] || exit 1

  error_status=$(graphql_output '.error.status')
  [[ "$error_status" == "Unauthorized" ]] || exit 1
}

@test "user: can create a user" {
  token=$(create_user)
  cache_value "alice" "$token"

  exec_graphql 'alice' 'me'
  user_id=$(graphql_output '.data.me.userId')
  [[ "$user_id" != "null" ]] || exit 1

  btc_address=$(graphql_output '.data.me.btcDepositAddress')
  cache_value 'user.btc' "$btc_address"

  ust_address=$(graphql_output '.data.me.ustDepositAddress')
  cache_value 'user.ust' "$ust_address"
}

@test "user: can deposit" {
  ust_address=$(read_value 'user.ust')
  btc_address=$(read_value 'user.btc')

  variables=$(
    jq -n \
      --arg address "$ust_address" \
    '{
       address: $address,
       amount: "10000",
       currency: "UST"
    }'
  )
  exec_cala_graphql 'simulate-deposit' "$variables"

  exec_graphql 'alice' 'me'
  usd_balance=$(graphql_output '.data.me.balance.checking.settled.usdBalance')
  [[ "$usd_balance" == 1000000 ]] || exit 1

  assert_assets_liabilities_equity

  btc_address=$(read_value 'user.btc')

  variables=$(
    jq -n \
      --arg address "$btc_address" \
    '{
       address: $address,
       amount: "1",
       currency: "BTC"
    }'
  )
  exec_cala_graphql 'simulate-deposit' "$variables"

  exec_graphql 'alice' 'me'
  btc_balance=$(graphql_output '.data.me.balance.unallocatedCollateral.settled.btcBalance')
  [[ "$btc_balance" == 100000000 ]] || exit 1

  assert_assets_liabilities_equity
}

@test "user: can withdraw" {
  variables=$(
    jq -n \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        amount: 150000,
        destination: "tron-address",
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_graphql 'alice' 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  exec_graphql 'alice' 'me'
  checking_balance=$(graphql_output '.data.me.balance.checking.settled.usdBalance')
  [[ "$checking_balance" == 850000 ]] || exit 1

  assert_assets_liabilities_equity
}
