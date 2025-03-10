#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "equity: can add usd equity" {
  skip
  variables=$(
    jq -n \
    --arg from "$(from_utc)" \
    '{ from: $from }'
  )
  exec_admin_graphql 'balance-sheet' "$variables"
  assets_usd_before=$(graphql_output \
    --arg category_name "Assets" \
    '.data.balanceSheet.categories[] | select(.name == $category_name) .amounts.usd.balancesByLayer.settled.netDebit'
  )
  equity_usd_before=$(graphql_output \
    --arg category_name "Equity" \
    '.data.balanceSheet.categories[] | select(.name == $category_name) .amounts.usd.balancesByLayer.settled.netDebit'
  )


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

  # assert_accounts_balanced

  variables=$(
    jq -n \
    --arg from "$(from_utc)" \
    '{ from: $from }'
  )
  exec_admin_graphql 'balance-sheet' "$variables"
  assets_usd=$(graphql_output \
    --arg category_name "Assets" \
    '.data.balanceSheet.categories[] | select(.name == $category_name) .amounts.usd.balancesByLayer.settled.netDebit'
  )
  equity_usd=$(graphql_output \
    --arg category_name "Equity" \
    '.data.balanceSheet.categories[] | select(.name == $category_name) .amounts.usd.balancesByLayer.settled.netDebit'
  )
  [[ "$assets_usd" -gt "$assets_usd_before" ]] || exit 1
  [[ "$equity_usd" -lt "$equity_usd_before" ]] || exit 1
}

