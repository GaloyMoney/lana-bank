#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "equity: can receive btc equity" {
  exec_cala_graphql 'equity-accounts'
  debit_balance_before=$(graphql_output '.data.debit.btcBalance.settled.normalBalance.units')
  credit_balance_before=$(graphql_output '.data.credit.btcBalance.settled.normalBalance.units')

  exec_admin_graphql 'equity-address'
  btc_address=$(graphql_output '.data.shareholderEquityBtcAddressCurrent')

  variables=$(
    jq -n \
      --arg address "$btc_address" \
    '{
       address: $address,
       amount: "10",
       currency: "BTC"
    }'
  )
  exec_cala_graphql 'simulate-deposit' "$variables"

  exec_cala_graphql 'equity-accounts'
  debit_balance=$(graphql_output '.data.debit.btcBalance.settled.normalBalance.units')
  [[ "$debit_balance" -gt "$debit_balance_before" ]] || exit 1
  credit_balance=$(graphql_output '.data.credit.btcBalance.settled.normalBalance.units')
  [[ "$credit_balance" -gt "$credit_balance_before" ]] || exit 1
}

