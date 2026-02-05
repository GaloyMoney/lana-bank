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

  cost_of_revenue_code=$(graphql_output "${config}.costOfRevenueCode")
  [[ "$cost_of_revenue_code" == "5" ]] || exit 1

  expenses_code=$(graphql_output "${config}.expensesCode")
  [[ "$expenses_code" == "6" ]] || exit 1

  retained_earnings_gain_code=$(graphql_output "${config}.equityRetainedEarningsGainCode")
  [[ "$retained_earnings_gain_code" == "32.01" ]] || exit 1

  retained_earnings_loss_code=$(graphql_output "${config}.equityRetainedEarningsLossCode")
  [[ "$retained_earnings_loss_code" == "32.02" ]] || exit 1
}

@test "accounting: can query account sets by category" {
  # Test ASSET category
  exec_admin_graphql 'account-sets-by-category' '{"category": "ASSET"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^1 ]] || exit 1

  # Test LIABILITY category
  exec_admin_graphql 'account-sets-by-category' '{"category": "LIABILITY"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^2 ]] || exit 1

  # Test EQUITY category
  exec_admin_graphql 'account-sets-by-category' '{"category": "EQUITY"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^3 ]] || exit 1

  # Test REVENUE category
  exec_admin_graphql 'account-sets-by-category' '{"category": "REVENUE"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^4 ]] || exit 1

  # Test COST_OF_REVENUE category
  exec_admin_graphql 'account-sets-by-category' '{"category": "COST_OF_REVENUE"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^5 ]] || exit 1

  # Test EXPENSES category
  exec_admin_graphql 'account-sets-by-category' '{"category": "EXPENSES"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(graphql_output '.data.accountSetsByCategory[0].code')
  [[ "$first_code" =~ ^6 ]] || exit 1
}

@test "accounting: can query off-balance sheet account sets" {
  exec_admin_graphql 'account-sets-by-category' '{"category": "OFF_BALANCE_SHEET"}'
  count=$(graphql_output '.data.accountSetsByCategory | length')
  # The test chart has off-balance sheet accounts under codes 7 and 8
  [[ "$count" -gt 0 ]] || exit 1

  # Verify that returned account sets are not from the main statement categories (1-6)
  codes=$(graphql_output '.data.accountSetsByCategory[].code')
  for code in $codes; do
    # Check that code doesn't start with 1-6 (main statement categories)
    [[ ! "$code" =~ ^[1-6] ]] || exit 1
  done
}

@test "accounting: can import CSV file into chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  chart_id=$(graphql_output '.data.chartOfAccounts.chartId')

  temp_file=$(mktemp)
  new_root_code=$((RANDOM % 100 + 900))
  echo "
    $new_root_code,,,CSV Import Test Root,,
    ,$((RANDOM % 100)),,CSV Import Test Child,,
  " > "$temp_file"

  variables=$(
    jq -n \
    '{
      input: {
        file: null
      }
    }'
  )

  response=$(exec_admin_graphql_upload 'chart-of-accounts-csv-import' "$variables" "$temp_file" "input.file")
  payload_chart_id=$(echo "$response" | jq -r '.data.chartOfAccountsCsvImport.chartOfAccounts.chartId')
  [[ "$payload_chart_id" == "$chart_id" ]] || exit 1

  exec_admin_graphql 'chart-of-accounts'
  res=$(graphql_output \
      --arg code "$new_root_code" \
      '.data.chartOfAccounts.children[]
      | select(.accountCode == $code)
      | .accountCode' | head -n 1)
  [[ "$res" == "$new_root_code" ]] || exit 1
}

@test "accounting: can traverse chart of accounts" {
  exec_admin_graphql 'chart-of-accounts'
  echo $(graphql_output)
  # Check that Assets exists with code "1" (from seed data)
  assets_code=$(echo "$(graphql_output)" | jq -r \
    '.data.chartOfAccounts.children[] | select(.name == "Assets") | .accountCode'
  )
  [[ "$assets_code" == "1" ]] || exit 1
}

@test "accounting: can execute manual transaction" {

  # Use existing accounts from seed data
  # 11.01.0101 = Operating Cash (Asset)
  # 61.01 = Salaries and Wages (Expense)

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
             "accountRef": "11.01.0101",
             "amount": $amount,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 1 description"
          },
          {
             "accountRef": "61.01",
             "amount": $amount,
             "currency": "USD",
             "direction": "DEBIT",
             "description": "Entry 2 description"
          }]
        }
      }'
  )

  exec_admin_graphql 'manual-transaction-execute' "$variables"

  exec_admin_graphql 'ledger-account-by-code' '{"code":"11.01.0101"}'
  txId1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].txId)
  amount1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].amount.usd)
  direction1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].direction)
  [[ "$direction1" != "null" ]] || exit 1
  entryType1=$(graphql_output .data.ledgerAccountByCode.history.nodes[0].entryType)
  [[ "$entryType1" != "null" ]] || exit 1

  exec_admin_graphql 'ledger-account-by-code' '{"code":"61.01"}'
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
             "accountRef": "11.01.0101",
             "amount": $amount,
             "currency": "USD",
             "direction": "CREDIT",
             "description": "Entry 1 description"
          },
          {
             "accountRef": "61.01",
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
