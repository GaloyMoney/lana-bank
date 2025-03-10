#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="chart-of-accounts.e2e-logs"
RUN_LOG_FILE="chart-of-accounts.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

@test "chart-of-accounts: can traverse chart of accounts" {
  skip # until new structure is fully integrated

  exec_admin_graphql 'chart-of-accounts'
  graphql_output
  echo "chart-of-accounts | $(graphql_output)" >> $RUN_LOG_FILE

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
