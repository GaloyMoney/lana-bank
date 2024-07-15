#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "chart-of-accounts: can traverse chart of accounts" {
  for i in {1..2}; do create_user; done

  # Fetch Chart of Accounts
  exec_admin_graphql 'chart-of-accounts'

  liabilities_name="Liabilities"
  user_checking_control_name="User Checking Control Account"
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
    '{
      accountSetId: $account_set_id,
      first: 100,
    }'
  )
  exec_admin_graphql 'chart-of-accounts-account-set' "$variables"
  num_accounts=$(graphql_output '.data.accountSet.subAccounts.edges | length')
  first_cursor=$(graphql_output '.data.accountSet.subAccounts.edges[0].cursor')
  [[ "$num_accounts" -gt "0" ]] || exit 1

  # Fetch paginated page
  variables=$(
    jq -n \
      --arg account_set_id "$user_checking_control_account_set_id" \
      --arg after "$first_cursor" \
    '{
      accountSetId: $account_set_id,
      first: 100,
      after: $after
    }'
  )
  exec_admin_graphql 'chart-of-accounts-account-set' "$variables"
  num_accounts_paginated=$(graphql_output '.data.accountSet.subAccounts.edges | length')
  [[ "$num_accounts_paginated" -gt "0" ]] || exit 1
  [[ "$num_accounts_paginated" -lt "$num_accounts" ]] || exit 1
}
