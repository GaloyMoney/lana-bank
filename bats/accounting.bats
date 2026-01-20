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
  n_children_before=$(graphql_output '.data.chartOfAccounts.children | length')
  
  new_code="$(( RANDOM % 9000 + 1000 ))"
  name="Root Account #$new_code"
  variables=$(
    jq -n \
    --arg code "$new_code" \
    --arg name "$name" \
    '{
      input: {
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
    --arg code "$new_code" \
    --arg name "$name" \
    '{
      input: {
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

@test "accounting: accounting base config is set on chart of accounts" {
  exec_admin_graphql 'accounting-base-config'
  config='.data.chartOfAccounts.accountingBaseConfig'

  assets_code=$(graphql_output "${config}.assetsCode")
  [[ "$assets_code" == "1" ]] || exit 1

  liabilities_code=$(graphql_output "${config}.liabilitiesCode")
  [[ "$liabilities_code" == "2" ]] || exit 1

  equity_code=$(graphql_output "${config}.equityCode")
  [[ "$equity_code" == "3" ]] || exit 1

  revenue_code=$(graphql_output "${config}.revenueCode")
  [[ "$revenue_code" == "4" ]] || exit 1

  expenses_code=$(graphql_output "${config}.expensesCode")
  [[ "$expenses_code" == "5" ]] || exit 1
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
    '{
      input: {
        file: null,
        baseConfig: {
          assetsCode: "1",
          liabilitiesCode: "2",
          equityCode: "3",
          equityRetainedEarningsGainCode: "32.01",
          equityRetainedEarningsLossCode: "32.02",
          revenueCode: "4",
          costOfRevenueCode: "5",
          expensesCode: "6"
        }
      }
    }'
  )

  response=$(exec_admin_graphql_upload 'chart-of-accounts-csv-import-with-base-config' "$variables" "$temp_file" "input.file")
  payload_chart_id=$(echo "$response" | jq -r '.data.chartOfAccountsCsvImportWithBaseConfig.chartOfAccounts.chartId')
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
  [[ "$direction1" != "null" ]] || exit 1
  entryType1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)
  [[ "$entryType1" != "null" ]] || exit 1

  exec_admin_graphql 'ledger-account-by-code' '{"code":"202"}'
  txId2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  [[ "$direction2" != "null" ]] || exit 1
  entryType2=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)
  [[ "$entryType2" != "null" ]] || exit 1

  [[ "$txId1" == "$txId2" ]] || exit 1
  [[ $((amount * 100)) == $amount1 ]] || exit 1
  [[ $amount1 == $amount2 ]] || exit 1
  [[ "$direction1" != "$direction2" ]] || exit 1
  [[ "$entryType1" != "$entryType2" ]] || exit 1
}

@test "accounting: can not execute transaction before system inception date" {
  exec_admin_graphql 'fiscal-years' '{"first": 1}'
  graphql_output
  inception_date=$(graphql_output '.data.fiscalYears.nodes[0].openedAsOf')
  [[ "$inception_date" != "null" ]] || exit 1
  first_closed_as_of_date=$(date -d "$inception_date -1 day" +%Y-%m-%d)

  amount=$((RANDOM % 1000))
  variables=$(
    jq -n \
    --arg amount "$amount" \
    --arg effective "$first_closed_as_of_date" \
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

@test "accounting: can close month in fiscal year" {
  exec_admin_graphql 'fiscal-years' '{"first": 1}'
  fiscal_year_id=$(graphql_output '.data.fiscalYears.nodes[0].fiscalYearId')

  last_month_of_year_closed=$(graphql_output '.data.fiscalYears.nodes[0].isLastMonthOfYearClosed')
  [[ "$last_month_of_year_closed" = "false" ]] || exit 1
  n_month_closures_before=$(graphql_output '.data.fiscalYears.nodes[0].monthClosures | length')

  variables=$(
    jq -n \
    --arg fiscal_year_id "$fiscal_year_id" \
    '{
      input: {
        fiscalYearId: $fiscal_year_id
      }
    }'
  )
  exec_admin_graphql 'fiscal-year-close-month' "$variables"
  n_month_closures_after=$(graphql_output '.data.fiscalYearCloseMonth.fiscalYear.monthClosures | length')
  [[ "$n_month_closures_after" -gt "$n_month_closures_before" ]] || exit 1
}

@test "accounting: can close fiscal year" {
  exec_admin_graphql 'fiscal-years' '{"first": 1}'
  fiscal_year_id=$(graphql_output '.data.fiscalYears.nodes[0].fiscalYearId')
  last_month_of_year_closed=$(graphql_output '.data.fiscalYears.nodes[0].isLastMonthOfYearClosed')

  is_open_before=$(graphql_output '.data.fiscalYears.nodes[0].isOpen')
  [[ "$is_open_before" = "true" ]] || exit 1

  variables=$(
    jq -n \
    --arg fiscal_year_id "$fiscal_year_id" \
    '{
      input: {
        fiscalYearId: $fiscal_year_id
      }
    }'
  )

  count=0
  while [[ "$last_month_of_year_closed" = "false" ]]; do
    exec_admin_graphql 'fiscal-year-close-month' "$variables"
    last_month_of_year_closed=$(graphql_output '.data.fiscalYearCloseMonth.fiscalYear.isLastMonthOfYearClosed')

    count=$(( $count + 1 ))
    [[ "$count" -lt 20 ]] || exit 1
  done

  
  exec_admin_graphql 'fiscal-year-close' "$variables"
  is_open_after=$(graphql_output '.data.fiscalYearClose.fiscalYear.isOpen')
  [[ "$is_open_after" = "false" ]] || exit 1
}
