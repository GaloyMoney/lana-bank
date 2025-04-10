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

@test "chart-of-accounts: can import CSV file" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')

  temp_file=$(mktemp)
  echo "
    201,,,Manuals 1,,
    202,,,Manuals 2,,
    $((RANDOM % 100)),,,Assets ,,
    ,,,,,
    $((RANDOM % 100)),,,Assets,,
    ,,,,,
    ,$((RANDOM % 100)),,Effective,,
    ,,$((RANDOM % 1000)),Central Office,
  " > "$temp_file"

  variables=$(
    jq -n \
    --arg chart_id "$chart_id" \
    '{
      input: {
        chartId: $chart_id,
        file: null
      }
    }'
  )

  response=$(exec_admin_graphql_upload 'chart-of-accounts-csv-import' "$variables" "$temp_file" "input.file")
  success=$(echo "$response" | jq -r '.data.chartOfAccountsCsvImport.success')
  [[ "$success" == "true" ]] || exit 1
}

@test "chart-of-accounts: executes manual transaction" {

  amount=$((RANDOM % 1000))

  variables=$(
    jq -n \
    --arg amount "$amount" \
    '{
      input: {
        description: "Manual transaction - test",
        entries: [
          {
             "accountRef": "201",
             "amount": $amount,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 1 description"
          },
          {
             "accountRef": "202",
             "amount": $amount,
             "currency": "USD",
             "direction": "DEBIT",
             "description": "Entry 2 description"
          }]
        }
      }'
  )

  exec_admin_graphql 'manual-transaction-execute' "$variables"

  exec_admin_graphql 'ledger-account-by-code' '{"code":"201"}'
  txId1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  entryType1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)

  exec_admin_graphql 'ledger-account-by-code' '{"code":"202"}'
  txId2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  entryType2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)

  [[ "$txId1" == "$txId2" ]] || exit 1
  [[ $((amount * 100)) == $amount1 ]] || exit 1
  [[ $amount1 == $amount2 ]] || exit 1
  [[ "$direction1" != "$direction2" ]] || exit 1
  [[ "$entryType1" != "$entryType2" ]] || exit 1
}
