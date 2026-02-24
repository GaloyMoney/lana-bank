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
  variables=$(
    jq -n \
      --arg ledgerAccountId "$ledger_account_id" \
    '{ ledgerAccountId: $ledgerAccountId }'
  )
  exec_admin_graphql 'account-entry-csv' "$variables"
  status=$(graphql_output '.data.accountEntryCsv.status')
  [[ "$status" == "ACTIVE" ]] || return 1
}

@test "CSV export: can create and download CSV export" {
  exec_admin_graphql 'chart-of-accounts'
  ledger_account_code=$(graphql_output '.data.chartOfAccounts.children[0].accountCode')
  [[ "$ledger_account_code" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg code "$ledger_account_code" \
      '{
        code: $code
      }'
  )

  exec_admin_graphql 'ledger-account-by-code' "$variables"
  ledger_account_id=$(graphql_output '.data.ledgerAccountByCode.ledgerAccountId')
  [[ "$ledger_account_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg ledgerAccountId "$ledger_account_id" \
      '{
        input: {
          ledgerAccountId: $ledgerAccountId
        }
      }'
  )

  exec_admin_graphql 'ledger-account-csv-create' "$variables"
  document_id=$(graphql_output '.data.ledgerAccountCsvCreate.accountingCsvDocument.documentId')
  [[ "$document_id" != "null" ]] || exit 1

  # Wait for the async CSV generation job to complete
  retry 30 1 wait_for_csv_export_completion "$ledger_account_id"

  variables=$(
    jq -n \
      --arg documentId "$document_id" \
      '{
        input: {
          documentId: $documentId
        }
      }'
  )

  exec_admin_graphql 'accounting-csv-download-link-generate' "$variables"
  download_url=$(graphql_output '.data.accountingCsvDownloadLinkGenerate')

  [[ "$download_url" != "null" ]] || exit 1
}
