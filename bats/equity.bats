#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "equity: can add usd equity" {
  exec_cala_graphql 'equity-accounts'
  debit_balance_before=$(graphql_output '.data.debit.usdBalance.settled.normalBalance.units')
  credit_balance_before=$(graphql_output '.data.credit.usdBalance.settled.normalBalance.units')


  variables=$(
    jq -n \
      --arg reference "equity-$(random_uuid)" \
    '{
      input: {
        amount: 500000000,
        reference: $reference,
      }
    }'
  )
  exec_admin_graphql 'add-shareholder-equity' "$variables"

  exec_cala_graphql 'equity-accounts'
  debit_balance=$(graphql_output '.data.debit.usdBalance.settled.normalBalance.units')
  [[ "$debit_balance" -gt "$debit_balance_before" ]] || exit 1
  credit_balance=$(graphql_output '.data.credit.usdBalance.settled.normalBalance.units')
  [[ "$credit_balance" -gt "$credit_balance_before" ]] || exit 1
}

