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

  category_account_code=$(echo "$output" | jq -r \
    '.data.chartOfAccounts.categories.assets.accountCode'
  )
  [[ "$category_account_code" == "10000" ]] || exit 1


  control_name="Deposits"
  control_account_code=$(echo "$output" | jq -r \
    --arg account_name "$control_name" \
    '
      .data.chartOfAccounts.categories.liabilities |
      .controlAccounts[] |
      select(.name == $account_name)
      .accountCode
    '
  )
  [[ "$control_account_code" == "20100" ]] || exit 1

  control_name="Credit Facilities Interest Income"
  control_sub_name="Fixed Term Credit Facilities Interest Income"
  control_account_code=$(echo "$output" | jq -r \
    --arg account_name "$control_name" \
    --arg account_sub_name "$control_sub_name" \
    '
      .data.chartOfAccounts.categories.revenues |
      .controlAccounts[] |
      select(.name == $account_name)
      .controlSubAccounts[] |
      select(.name == $account_sub_name)
      .accountCode
    '
  )
  [[ "$control_account_code" == "40101" ]] || exit 1
}
