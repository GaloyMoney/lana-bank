#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="deposit_account.e2e-logs"
RUN_LOG_FILE="deposit_account.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_customer_activation() {
  customer_id=$1
  variables=$(jq -n --arg customerId "$customer_id" '{ id: $customerId }')
  exec_admin_graphql 'customer' "$variables"
  status=$(graphql_output '.data.customer.status')
  [[ "$status" == "ACTIVE" ]] || exit 1
}

@test "deposit_accounts: fetch by id and short code after customer creation" {
  customer_id=$(create_customer)
  
  # Optional: Wait for customer activation if required by the flow
  # retry 10 1 wait_for_customer_activation "$customer_id"

  # 2. Fetch Customer to get Deposit Account Details
  # Add retry logic here if deposit account details are not immediately available
  retry_count=5
  delay=1
  for (( i=0; i<$retry_count; i++ )); do
    echo "# Attempt $((i+1)) to fetch customer $customer_id deposit details..."
    variables_fetch_customer=$(jq -n --arg id "$customer_id" '{ id: $id }')
    exec_admin_graphql 'customer' "$variables_fetch_customer"

    deposit_account_id=$(graphql_output .data.customer.depositAccount.depositAccountId)
    short_code_id=$(graphql_output .data.customer.depositAccount.shortCodeId)

    if [[ -n "$deposit_account_id" && "$deposit_account_id" != "null" && -n "$short_code_id" && "$short_code_id" != "null" ]]; then
      echo "# Successfully fetched deposit details: AccountID=$deposit_account_id, ShortCode=$short_code_id"
      break
    fi

    if [[ $i -eq $((retry_count-1)) ]]; then
       echo "# Failed to get deposit account details for customer $customer_id after $retry_count attempts."
       exit 1
    fi
    sleep $delay
  done

  # Check if short code ID is within the expected sequence range (from migration)
  [[ "$short_code_id" -ge 1000 && "$short_code_id" -le 9999999 ]] || { echo "Expected shortCodeId >= 1000, got $short_code_id"; exit 1; }

  # 3. Test fetching the deposit account by ID
  echo "# Fetching deposit account by ID: $deposit_account_id"
  variables_by_id=$(jq -n --arg id "$deposit_account_id" '{ id: $id }')
  exec_admin_graphql 'deposit-account' "$variables_by_id"

  echo "graphql_output: $(graphql_output)"

  fetched_by_id_account_id=$(graphql_output .data.depositAccount.depositAccountId)
  fetched_by_id_holder_id=$(graphql_output .data.depositAccount.customerId)
  fetched_by_id_short_code_id=$(graphql_output .data.depositAccount.shortCodeId)

  echo "# Fetched by ID: AccID=$fetched_by_id_account_id, HolderID=$fetched_by_id_holder_id, ShortCode=$fetched_by_id_short_code_id"
  [[ "$fetched_by_id_account_id" == "$deposit_account_id" ]] || { echo "ID mismatch: Expected $deposit_account_id, got $fetched_by_id_account_id"; exit 1; }
  [[ "$fetched_by_id_holder_id" == "$customer_id" ]] || { echo "Holder ID mismatch: Expected $customer_id, got $fetched_by_id_holder_id"; exit 1; }
  [[ "$fetched_by_id_short_code_id" == "$short_code_id" ]] || { echo "Short Code ID mismatch: Expected $short_code_id, got $fetched_by_id_short_code_id"; exit 1; }

  # 4. Test fetching the deposit account by short code ID
  echo "# Fetching deposit account by Short Code: $short_code_id"
  variables_by_code=$(jq -n --arg code "$short_code_id" '{ code: $code }')
  exec_admin_graphql 'deposit-account-by-code' "$variables_by_code"

  fetched_by_code_account_id=$(graphql_output .data.depositAccountByCode.depositAccountId)
  fetched_by_code_holder_id=$(graphql_output .data.depositAccountByCode.customerId)
  fetched_by_code_short_code_id=$(graphql_output .data.depositAccountByCode.shortCodeId)

  echo "# Fetched by Code: AccID=$fetched_by_code_account_id, HolderID=$fetched_by_code_holder_id, ShortCode=$fetched_by_code_short_code_id"
  [[ "$fetched_by_code_account_id" == "$deposit_account_id" ]] || { echo "ID mismatch: Expected $deposit_account_id, got $fetched_by_code_account_id"; exit 1; }
  [[ "$fetched_by_code_holder_id" == "$customer_id" ]] || { echo "Holder ID mismatch: Expected $customer_id, got $fetched_by_code_holder_id"; exit 1; }
  [[ "$fetched_by_code_short_code_id" == "$short_code_id" ]] || { echo "Short Code ID mismatch: Expected $short_code_id, got $fetched_by_code_short_code_id"; exit 1; }

  echo "# Test completed successfully."
} 