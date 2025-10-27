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

@test "accounting: add new root node into chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')
  n_children_before=$(graphql_output '.data.chartOfAccounts.children | length')
  
  new_code="$(( RANDOM % 9000 + 1000 ))"
  name="Root Account #$new_code"
  variables=$(
    jq -n \
    --arg id "$chart_id" \
    --arg code "$new_code" \
    --arg name "$name" \
    '{
      input: {
        chartId: $id,
        code: $code,
        name: $name,
        normalBalanceType: "CREDIT"
      }
    }'
  )
  exec_admin_graphql 'chart-of-accounts-add-root-node' "$variables"
  n_children_after=$(graphql_output '.data.chartOfAccountsAddRootNode.chartOfAccounts.children | length')
  [[ "$n_children_after" -gt "$n_children_before" ]] || exit 1
}

@test "accounting: add new child node into chart of accounts" {
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
        name: $name
      }
    }'
  )
  exec_admin_graphql 'chart-of-accounts-add-child-node' "$variables"
  n_children_after=$(graphql_output '
    .data.chartOfAccountsAddChildNode.chartOfAccounts.children[]
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

  echo "
    1,,,Assets,Debit,
    ,,,,,
    11,,,Current Assets,,
    ,,,,,
    ,01,,Cash and Equivalents,,
    ,,,,,
    ,,0101,Operating Cash,,
    ,,,,,
    ,,0102,Petty Cash,,
    ,,,,,
    ,02,,Receivables,,
    ,,,,,
    ,,0201,Interest Receivable,,
    ,,,,,
    ,,0202,Loans Receivable,,
    ,,,,,
    ,,0203,Overdue Loans Receivable,,
    ,,,,,
    ,03,,Inventory,,
    ,,,,,
    ,,0301,Raw Materials,,
    ,,,,,
    ,,0302,Work In Progress,,
    ,,,,,
    ,,0303,Finished Goods,,
    ,,,,,
    12,,,Non-Current Assets,,
    ,,,,,
    ,01,,Property and Equipment,,
    ,,,,,
    ,,0101,Land,,
    ,,,,,
    ,,0102,Buildings,,
    ,,,,,
    ,,0103,Equipment,,
    ,,,,,
    ,02,,Intangible Assets,,
    ,,,,,
    ,,0201,Goodwill,,
    ,,,,,
    ,,0202,Intellectual Property,,
    ,,,,,
    3,,,Equity,Credit,
    ,,,,,
    31,,,Contributed Capital,,
    ,,,,,
    ,01,,Common Stock,,
    ,,,,,
    ,02,,Preferred Stock,,
    ,,,,,
    32,,,Retained Earnings,,
    ,,,,,
    ,01,,Prior Year Earnings,,
    ,,,,,
    ,02,,Prior Year Losses,,
    ,,,,,
    4,,,Revenue,Credit,
    ,,,,,
    41,,,Operating Revenue,,
    ,,,,,
    ,01,,Sales Revenue,,
    ,,,,,
    ,,0101,Product A Sales,,
    ,,,,,
    ,,0102,Product B Sales,,
    ,,,,,
    ,02,,Service Revenue,,
    ,,,,,
    ,,0201,Consulting Services,,
    ,,,,,
    ,,0202,Maintenance Services,,
    ,,,,,
    5,,,Cost of Revenue,Debit,
    ,,,,,
    51,,,Capital Cost,,
    ,,,,,
    ,01,,Custody,,
    ,,,,,
    ,,0101,Custodian Fees,,
    ,,,,,
    6,,,Expenses,Debit,
    ,,,,,
    61,,,Fixed Expenses,,
    ,,,,,
    ,01,,Regulatory,,
    ,,,,,
    ,,0101,Regulatory Fees,,
    ,,,,,
    201,,,Manuals 1,,
    202,,,Manuals 2,,
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
    --arg effective "2025-01-01" \
    '{
      input: {
        description: "Manual transaction - test",
        effective: $effective,
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

@test "accounting: can execute manual transaction into profit and loss accounts" {

  # expects chart of accounts from 'import CSV' step to exist

  revenue=500
  cost_of_revenue=200
  expenses=200

  variables=$(
    jq -n \
    --arg revenue "$revenue" \
    --arg cost_of_rev "$cost_of_revenue" \
    --arg expenses "$expenses" \
    --arg effective "2025-01-01" \
    '{
      input: {
        description: "Manual transaction - test profit and loss",
        effective: $effective,
        entries: [
          {
             "accountRef": "41.01.0102",
             "amount": $revenue,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 1C description"
          },
          {
             "accountRef": "51.01.0101",
             "amount": $cost_of_rev,
             "currency": "USD",
             "direction": "DEBIT",
             "description": "Entry 2C description"
          },
                    {
             "accountRef": "61.01.0101",
             "amount": $expenses,
             "currency": "USD",
             "direction": "DEBIT",
             "description": "Entry 3C description"
          },
          {
             "accountRef": "202",
             "amount": $revenue,
             "currency": "USD",
             "direction": "DEBIT",
             "description": "Entry 1D description"
          },
          {
             "accountRef": "202",
             "amount": $cost_of_rev,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 2C description"
          },
          {
             "accountRef": "202",
             "amount": $expenses,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 3C description"
          }]
        }
      }'
  )

  exec_admin_graphql 'manual-transaction-execute' "$variables"

  exec_admin_graphql 'ledger-account-by-code' '{"code":"41.01.0102"}'
  txId1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  entryType1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)

  exec_admin_graphql 'ledger-account-by-code' '{"code":"202"}'
  txId2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  entryType2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)
}

@test "accounting: cannot execute transaction before last closing date" {
  exec_admin_graphql 'chart-of-accounts-closing'
  graphql_output
  closing_date=$(graphql_output '.data.chartOfAccounts.monthlyClosing.closedAsOf')
  [[ "$closing_date" != "null" ]] || exit 1

  amount=$((RANDOM % 1000))
  variables=$(
    jq -n \
    --arg amount "$amount" \
    --arg effective "$closing_date" \
    '{
      input: {
        description: "Manual transaction - test",
        effective: $effective,
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
  graphql_output
  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "VelocityError" ]] || exit 1
}

@test "accounting: confirm annual closing transfers net income to balance sheet" {
    exec_admin_graphql 'chart-of-accounts'
    chart_id=$(graphql_output '.data.chartOfAccounts.chartId')

    variables=$(
      jq -n \
      --arg chart_id "$chart_id" \
      '{
        input: {
          chartId: $chart_id
        }
      }'
    )

    exec_admin_graphql 'accounting-period-close-year' "$variables"
    
    exec_admin_graphql 'ledger-account-by-code' '{"code":"32.02"}'
}
