#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "chart-of-accounts: can traverse chart of accounts" {
  skip # TODO: re-implement from entity

  for i in {1..2}; do create_customer; done

  # Fetch Chart of Accounts
  exec_admin_graphql 'chart-of-accounts'

  liabilities_name="Liabilities"
  user_checking_control_name="Customer Checking Control Account"
  user_checking_control_account_set_id=$(echo "$output" | jq -r \
    --arg category_name "$liabilities_name" \
    --arg account_name "$user_checking_control_name" \
    '
      .data.chartOfAccounts.categories[] |
      select(.name == $category_name) |
      .accounts[] |
      select(.name == $account_name)
      .queryableId
    '
  )

  # Fetch 1st page
  variables=$(
    jq -n \
      --arg account_set_id "$user_checking_control_account_set_id" \
      --arg from "$(from_utc)" \
    '{
      accountSetId: $account_set_id,
      first: 100,
      from: $from,
    }'
  )
  exec_admin_graphql 'account-set' "$variables"
  num_accounts_with_balance=$(graphql_output '.data.accountSet.subAccounts.edges | length')
  first_cursor=$(graphql_output '.data.accountSet.subAccounts.edges[0].cursor')
  btc_balance=$(graphql_output '.data.accountSet.subAccounts.edges[0].node.amounts.btc.balancesByLayer.all.netDebit')
  [[ "$num_accounts_with_balance" -gt "0" ]] || exit 1
  [[ "$btc_balance" == "0" ]] || exit 1

  # Fetch paginated page
  variables=$(
    jq -n \
      --arg account_set_id "$user_checking_control_account_set_id" \
      --arg after "$first_cursor" \
      --arg from "$(from_utc)" \
      '{
      accountSetId: $account_set_id,
      first: 100,
      after: $after,
      from: $from,
    }'
  )
  exec_admin_graphql 'account-set' "$variables"
  num_accounts_paginated_with_balance=$(graphql_output '.data.accountSet.subAccounts.edges | length')
  [[ "$num_accounts_paginated_with_balance" -gt "0" ]] || exit 1
  [[ "$num_accounts_paginated_with_balance" -lt "$num_accounts_with_balance" ]] || exit 1
}
