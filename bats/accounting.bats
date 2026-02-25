#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="accounting.e2e-logs"
RUN_LOG_FILE="accounting.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

@test "accounting: imported CSV file from seed into chart of accounts" {
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  chart_id=$(echo "$cli_output" | jq -r '.chartId')
  assets_code=$(echo "$cli_output" | jq -r '
    .children[]
    | select(.name == "Assets")
    | .accountCode' | head -n 1)
  [[ "$assets_code" -eq "1" ]] || exit 1
}

@test "accounting: add new root node into chart of accounts" {
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  n_children_before=$(echo "$cli_output" | jq -r '.children | length')

  new_code="$(( RANDOM % 9000 + 1000 ))"
  name="Root Account #$new_code"
  cli_output=$("$LANACLI" --json accounting add-root-node --code "$new_code" --name "$name" --normal-balance-type CREDIT)
  n_children_after=$(echo "$cli_output" | jq -r '.children | length')
  [[ "$n_children_after" -gt "$n_children_before" ]] || exit 1
}

@test "accounting: add new child node into chart of accounts" {
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  n_children_before=$(echo "$cli_output" | jq -r '
    .children[]
    | select(.accountCode == "1")
    | .children[]
    | select(.accountCode == "11")
    | .children[]
    | select(.accountCode == "11.01")
    | .children
    | length')

  new_code="11.01.$(( RANDOM % 9000 + 1000 ))"
  name="Account #$new_code"
  cli_output=$("$LANACLI" --json accounting add-child-node --parent "11.01" --code "$new_code" --name "$name")
  n_children_after=$(echo "$cli_output" | jq -r '
    .children[]
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
  cli_output=$("$LANACLI" --json accounting credit-config)
  omnibus_code=$(echo "$cli_output" | jq -r '.chartOfAccountFacilityOmnibusParentCode')
  [[ "$omnibus_code" == "81.01" ]] || exit 1
}

@test "accounting: imported deposit module config from seed into chart of accounts" {
  cli_output=$("$LANACLI" --json accounting deposit-config)
  omnibus_code=$(echo "$cli_output" | jq -r '.chartOfAccountsOmnibusParentCode')
  [[ "$omnibus_code" == "11.01.0101" ]] || exit 1
}

@test "accounting: accounting base config is set on chart of accounts" {
  cli_output=$("$LANACLI" --json accounting base-config)

  assets_code=$(echo "$cli_output" | jq -r '.assetsCode')
  [[ "$assets_code" == "1" ]] || exit 1

  liabilities_code=$(echo "$cli_output" | jq -r '.liabilitiesCode')
  [[ "$liabilities_code" == "2" ]] || exit 1

  equity_code=$(echo "$cli_output" | jq -r '.equityCode')
  [[ "$equity_code" == "3" ]] || exit 1

  revenue_code=$(echo "$cli_output" | jq -r '.revenueCode')
  [[ "$revenue_code" == "4" ]] || exit 1

  cost_of_revenue_code=$(echo "$cli_output" | jq -r '.costOfRevenueCode')
  [[ "$cost_of_revenue_code" == "5" ]] || exit 1

  expenses_code=$(echo "$cli_output" | jq -r '.expensesCode')
  [[ "$expenses_code" == "6" ]] || exit 1

  retained_earnings_gain_code=$(echo "$cli_output" | jq -r '.equityRetainedEarningsGainCode')
  [[ "$retained_earnings_gain_code" == "32.01" ]] || exit 1

  retained_earnings_loss_code=$(echo "$cli_output" | jq -r '.equityRetainedEarningsLossCode')
  [[ "$retained_earnings_loss_code" == "32.02" ]] || exit 1
}

@test "accounting: can query descendant account sets by category" {
  # Test ASSET category
  cli_output=$("$LANACLI" --json accounting account-sets --category ASSET)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^1 ]] || exit 1

  # Test LIABILITY category
  cli_output=$("$LANACLI" --json accounting account-sets --category LIABILITY)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^2 ]] || exit 1

  # Test EQUITY category
  cli_output=$("$LANACLI" --json accounting account-sets --category EQUITY)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^3 ]] || exit 1

  # Test REVENUE category
  cli_output=$("$LANACLI" --json accounting account-sets --category REVENUE)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^4 ]] || exit 1

  # Test COST_OF_REVENUE category
  cli_output=$("$LANACLI" --json accounting account-sets --category COST_OF_REVENUE)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^5 ]] || exit 1

  # Test EXPENSES category
  cli_output=$("$LANACLI" --json accounting account-sets --category EXPENSES)
  count=$(echo "$cli_output" | jq -r '. | length')
  [[ "$count" -gt 0 ]] || exit 1
  first_code=$(echo "$cli_output" | jq -r '.[0].code')
  [[ "$first_code" =~ ^6 ]] || exit 1
}

@test "accounting: can query off-balance sheet account sets" {
  cli_output=$("$LANACLI" --json accounting account-sets --category OFF_BALANCE_SHEET)
  count=$(echo "$cli_output" | jq -r '. | length')
  # The test chart has off-balance sheet accounts under codes 7 and 8
  [[ "$count" -gt 0 ]] || exit 1

  # Verify that returned account sets are not from the main statement categories (1-6)
  codes=$(echo "$cli_output" | jq -r '.[].code')
  for code in $codes; do
    # Check that code doesn't start with 1-6 (main statement categories)
    [[ ! "$code" =~ ^[1-6] ]] || exit 1
  done
}

@test "accounting: can import CSV file into chart of accounts" {
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  chart_id=$(echo "$cli_output" | jq -r '.chartId')

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

  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  res=$(echo "$cli_output" | jq -r \
      --arg code "$new_root_code" \
      '.children[]
      | select(.accountCode == $code)
      | .accountCode' | head -n 1)
  [[ "$res" == "$new_root_code" ]] || exit 1
}

@test "accounting: can traverse chart of accounts" {
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  echo "$cli_output"
  # Check that Assets exists with code "1" (from seed data)
  assets_code=$(echo "$cli_output" | jq -r \
    '.children[] | select(.name == "Assets") | .accountCode'
  )
  [[ "$assets_code" == "1" ]] || exit 1
}

@test "accounting: can execute manual transaction" {

  # Use existing accounts from seed data
  # 11.01.0101 = Operating Cash (Asset)
  # 61.01 = Salaries and Wages (Expense)

  amount=$((RANDOM % 1000))

  entries_json=$(jq -n -c \
    --arg amount "$amount" \
    '[
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
      }
    ]')

  "$LANACLI" --json accounting manual-transaction \
    --description "Manual transaction - test" \
    --effective "2025-01-01" \
    --entries-json "$entries_json"

  cli_output=$("$LANACLI" --json accounting ledger-account --code "11.01.0101")
  txId1=$(echo "$cli_output" | jq -r '.history.nodes[0].txId')
  amount1=$(echo "$cli_output" | jq -r '.history.nodes[0].amount.usd')
  direction1=$(echo "$cli_output" | jq -r '.history.nodes[0].direction')
  [[ "$direction1" != "null" ]] || exit 1
  entryType1=$(echo "$cli_output" | jq -r '.history.nodes[0].entryType')
  [[ "$entryType1" != "null" ]] || exit 1

  cli_output=$("$LANACLI" --json accounting ledger-account --code "61.01")
  txId2=$(echo "$cli_output" | jq -r '.history.nodes[0].txId')
  amount2=$(echo "$cli_output" | jq -r '.history.nodes[0].amount.usd')
  direction2=$(echo "$cli_output" | jq -r '.history.nodes[0].direction')
  [[ "$direction2" != "null" ]] || exit 1
  entryType2=$(echo "$cli_output" | jq -r '.history.nodes[0].entryType')
  [[ "$entryType2" != "null" ]] || exit 1

  [[ "$txId1" == "$txId2" ]] || exit 1
  [[ $((amount * 100)) == $amount1 ]] || exit 1
  [[ $amount1 == $amount2 ]] || exit 1
  [[ "$direction1" != "$direction2" ]] || exit 1
  [[ "$entryType1" != "$entryType2" ]] || exit 1
}

@test "accounting: can not execute transaction before system inception date" {
  cli_output=$("$LANACLI" --json fiscal-year list --first 1)
  inception_date=$(echo "$cli_output" | jq -r '.[0].openedAsOf')
  [[ "$inception_date" != "null" ]] || exit 1
  first_closed_as_of_date=$(date -d "$inception_date -1 day" +%Y-%m-%d)

  amount=$((RANDOM % 1000))
  entries_json=$(jq -n -c \
    --arg amount "$amount" \
    '[
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
      }
    ]')

  cli_output=$("$LANACLI" --json accounting manual-transaction \
    --description "Manual transaction - test" \
    --effective "$first_closed_as_of_date" \
    --entries-json "$entries_json" 2>&1 || true)
  [[ "$cli_output" =~ "VelocityError" ]] || exit 1
}

@test "accounting: can close month in fiscal year" {
  cli_output=$("$LANACLI" --json fiscal-year list --first 1)
  fiscal_year_id=$(echo "$cli_output" | jq -r '.[0].fiscalYearId')

  last_month_of_year_closed=$(echo "$cli_output" | jq -r '.[0].isLastMonthOfYearClosed')
  [[ "$last_month_of_year_closed" = "false" ]] || exit 1
  n_month_closures_before=$(echo "$cli_output" | jq -r '.[0].monthClosures | length')

  cli_output=$("$LANACLI" --json fiscal-year close-month --fiscal-year-id "$fiscal_year_id")
  n_month_closures_after=$(echo "$cli_output" | jq -r '.monthClosures | length')
  [[ "$n_month_closures_after" -gt "$n_month_closures_before" ]] || exit 1
}

@test "accounting: can close fiscal year" {
  cli_output=$("$LANACLI" --json fiscal-year list --first 1)
  fiscal_year_id=$(echo "$cli_output" | jq -r '.[0].fiscalYearId')
  last_month_of_year_closed=$(echo "$cli_output" | jq -r '.[0].isLastMonthOfYearClosed')

  is_open_before=$(echo "$cli_output" | jq -r '.[0].isOpen')
  [[ "$is_open_before" = "true" ]] || exit 1

  count=0
  while [[ "$last_month_of_year_closed" = "false" ]]; do
    cli_output=$("$LANACLI" --json fiscal-year close-month --fiscal-year-id "$fiscal_year_id")
    last_month_of_year_closed=$(echo "$cli_output" | jq -r '.isLastMonthOfYearClosed')

    count=$(( $count + 1 ))
    [[ "$count" -lt 20 ]] || exit 1
  done

  cli_output=$("$LANACLI" --json fiscal-year close --fiscal-year-id "$fiscal_year_id")
  is_open_after=$(echo "$cli_output" | jq -r '.isOpen')
  [[ "$is_open_after" = "false" ]] || exit 1
}
