#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "chart-of-accounts: can traverse chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'

  category_account_name="Bank Deposits from Users Control Account"
  category_account_set_id=$(echo "$output" | jq -r \
    --arg name "$category_account_name" '
      .data.chartOfAccounts.categories[].accounts[] |
      select(.name == $name) |
      .queryableId
    '
  )

  variables=$(
    jq -n \
      --arg account_set_id "$category_account_set_id" \
    '{
      accountSetId: $account_set_id
    }'
  )
  exec_admin_graphql 'chart-of-accounts-account-set' "$variables"
  sub_account_name=$(graphql_output '.data.chartOfAccountsAccountSet.subAccounts.edges[0].node.name')
  [[ "$sub_account_name" =~ "Bfx" ]] || exit 1
}
