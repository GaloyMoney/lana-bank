#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="csv-export.e2e-logs"
RUN_LOG_FILE="csv-export.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_csv_export_completion() {
  local ledger_account_id=$1
  local cli_output
  cli_output=$("$LANACLI" --json csv-export account-entry --ledger-account-id "$ledger_account_id")
  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "ACTIVE" ]] || return 1
}

@test "CSV export: can create and download CSV export" {
  local cli_output
  cli_output=$("$LANACLI" --json accounting chart-of-accounts)
  ledger_account_code=$(echo "$cli_output" | jq -r '.children[0].accountCode')
  [[ "$ledger_account_code" != "null" ]] || exit 1

  cli_output=$("$LANACLI" --json accounting ledger-account --code "$ledger_account_code")
  ledger_account_id=$(echo "$cli_output" | jq -r '.ledgerAccountId')
  [[ "$ledger_account_id" != "null" ]] || exit 1

  cli_output=$("$LANACLI" --json csv-export create-ledger-csv --ledger-account-id "$ledger_account_id")
  document_id=$(echo "$cli_output" | jq -r '.documentId')
  [[ "$document_id" != "null" ]] || exit 1

  # Wait for the async CSV generation job to complete
  retry 30 1 wait_for_csv_export_completion "$ledger_account_id"

  cli_output=$("$LANACLI" --json csv-export download-link --document-id "$document_id")
  download_url=$(echo "$cli_output" | jq -r '.url')

  [[ "$download_url" != "null" ]] || exit 1
}
