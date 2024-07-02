#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "equity: can add usd equity" {
  exec_cala_graphql 'assets-liabilities-equity'
  assets_usd_before=$(graphql_output '.data.balanceSheet.byJournalId.assets.usdBalance.settled.normalBalance.units')
  equity_usd_before=$(graphql_output '.data.balanceSheet.byJournalId.equity.usdBalance.settled.normalBalance.units')


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

  assert_accounts_balanced

  exec_cala_graphql 'assets-liabilities-equity'
  assets_usd=$(graphql_output '.data.balanceSheet.byJournalId.assets.usdBalance.settled.normalBalance.units')
  equity_usd=$(graphql_output '.data.balanceSheet.byJournalId.equity.usdBalance.settled.normalBalance.units')
  [[ "$assets_usd" -gt "$assets_usd_before" ]] || exit 1
  [[ "$equity_usd" -gt "$equity_usd_before" ]] || exit 1
}

