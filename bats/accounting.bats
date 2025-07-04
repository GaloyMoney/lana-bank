#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="accounting.e2e-logs"
RUN_LOG_FILE="accounting.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

@test "accounting: imported CSV file from seed into chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')
  assets_code=$(graphql_output '
    .data.chartOfAccounts.children[]
    | select(.name == "Assets")
    | .accountCode' | head -n 1)
  [[ "$assets_code" -eq "1" ]] || exit 1
}

@test "accounting: add new node into chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')
  n_children_before=$(graphql_output '
    .data.chartOfAccounts.children[]
    | select(.accountCode == "1")
    | .children[]
    | select(.accountCode == "11")
    | .children[]
    | select(.accountCode == "11.01")
    | .children
    | length')
  
  new_code="11.01.$(( RANDOM % 9000 + 1000 ))"
  name="Account #$new_code"
  variables=$(
    jq -n \
    --arg id "$chart_id" \
    --arg code "$new_code" \
    --arg name "$name" \
    '{
      input: {
        chartId: $id,
        parent: "11.01",
        code: $code,
        name: $name,
        normalBalanceType: "DEBIT",
      }
    }'
  )
  exec_admin_graphql 'chart-of-accounts-add-node' "$variables"
  n_children_after=$(graphql_output '
    .data.chartOfAccountsAddNode.chartOfAccounts.children[]
    | select(.accountCode == "1")
    | .children[]
    | select(.accountCode == "11")
    | .children[]
    | select(.accountCode == "11.01")
    | .children
    | length')
  [[ "$n_children_after" -gt "$n_children_before" ]] || exit 1
}

@test "accounting: imported credit module config from seed into chart of accounts" {
  exec_admin_graphql 'credit-config'
  omnibus_code=$(graphql_output '.data.creditConfig.chartOfAccountFacilityOmnibusParentCode')
  [[ "$omnibus_code" == "81.01" ]] || exit 1
}

@test "accounting: imported deposit module config from seed into chart of accounts" {
  exec_admin_graphql 'deposit-config'
  omnibus_code=$(graphql_output '.data.depositConfig.chartOfAccountsOmnibusParentCode')
  [[ "$omnibus_code" == "11.01.0101" ]] || exit 1
}

@test "accounting: imported balance sheet module config from seed into chart of accounts" {
  exec_admin_graphql 'balance-sheet-config'
  omnibus_code=$(graphql_output '.data.balanceSheetConfig.chartOfAccountsRevenueCode')
  [[ "$omnibus_code" == "4" ]] || exit 1
}

@test "accounting: imported profit and loss module config from seed into chart of accounts" {
  exec_admin_graphql 'profit-and-loss-config'
  omnibus_code=$(graphql_output '.data.profitAndLossStatementConfig.chartOfAccountsRevenueCode')
  [[ "$omnibus_code" == "4" ]] || exit 1
}

@test "accounting: can import CSV file into chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')

  temp_file=$(mktemp)
  liabilities_code=$((((RANDOM % 1000)) + 1000))
  echo "
    201,,,Manuals 1,,
    202,,,Manuals 2,,
    $liabilities_code,,,Alt Liabilities,,
    ,,,,,
    ,$((RANDOM % 100)),,Checking Accounts,,
    ,,$((RANDOM % 1000)),Northern Office,
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
  payload_chart_id=$(echo "$response" | jq -r '.data.chartOfAccountsCsvImport.chartOfAccounts.chartId')
  [[ "$payload_chart_id" == "$chart_id" ]] || exit 1

  exec_admin_graphql 'chart-of-accounts'
  res=$(graphql_output \
      --arg liabilitiesCode "$liabilities_code" \
      '.data.chartOfAccounts.children[]
      | select(.accountCode == $liabilitiesCode )
      | .accountCode' | head -n 1)
  [[ $res -eq "$liabilities_code" ]] || exit 1
}

@test "accounting: can traverse chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  echo $(graphql_output)
  control_name="Manuals 1"
  control_account_code=$(echo "$(graphql_output)" | jq -r \
    --arg account_name "$control_name" \
    '.data.chartOfAccounts.children[] | select(.name == $account_name) | .accountCode'
  )
  [[ "$control_account_code" == "201" ]] || exit 1
}

@test "accounting: can execute manual transaction" {

  # expects chart of accounts from 'import CSV' step to exist

  amount=$((RANDOM % 1000))

  variables=$(
    jq -n \
    --arg amount "$amount" \
    '{
      input: {
        description: "Manual transaction - test",
        effective: "2025-01-01",
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
