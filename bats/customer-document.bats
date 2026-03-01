#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

@test "documents: can upload a file, retrieve, archive, delete, and verify deletion" {
  if [[ -z "${SA_CREDS_BASE64}" ]]; then
    skip "Skipping test because SA_CREDS_BASE64 is not defined"
  fi

  # Create a customer via prospect flow
  customer_id=$(create_customer)
  [[ "$customer_id" != "null" ]] || exit 1

  # Generate a temporary file
  temp_file=$(mktemp)
  echo "Test content" > "$temp_file"
  
  # Upload the file via admin CLI
  cli_output=$("$LANACLI" --json document attach --customer-id "$customer_id" --file "$temp_file")
  document_id=$(echo "$cli_output" | jq -r '.documentId')
  [[ "$document_id" != null ]] || exit 1
  
  rm "$temp_file"

  local cli_output
  cli_output=$("$LANACLI" --json document get --id "$document_id")
  fetched_document_id=$(echo "$cli_output" | jq -r '.documentId')
  [[ "$fetched_document_id" == "$document_id" ]] || exit 1

  fetched_customer_id=$(echo "$cli_output" | jq -r '.customerId')
  [[ "$fetched_customer_id" == "$customer_id" ]] || exit 1

  # Fetch documents for the customer
  cli_output=$("$LANACLI" --json document list --customer-id "$customer_id")

  documents_count=$(echo "$cli_output" | jq '. | length')
  [[ "$documents_count" -ge 1 ]] || exit 1

  first_document_id=$(echo "$cli_output" | jq -r '.[0].documentId')
  [[ "$first_document_id" == "$document_id" ]] || exit 1

  # Generate download link for the document
  cli_output=$("$LANACLI" --json document download-link --document-id "$document_id")

  download_link=$(echo "$cli_output" | jq -r '.link')
  echo "Download link: $download_link"

  [[ "$download_link" != "null" && "$download_link" != "" ]] || exit 1

  # Handle both local file:// URLs and HTTP URLs
  if [[ "$download_link" == file://* ]]; then
    # For local storage, check if the file exists
    local_path="${download_link#file://}"
    [[ -f "$local_path" ]] || exit 1
    echo "Local file verified: $local_path"
  else
    # For HTTP URLs (GCP), use curl
    response=$(curl -s -o /dev/null -w "%{http_code}" "$download_link")
    [[ "$response" == "200" ]] || exit 1
  fi

  # archive the document
  cli_output=$("$LANACLI" --json document archive --document-id "$document_id")

  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "ARCHIVED" ]] || exit 1

  # Delete the document
  cli_output=$("$LANACLI" --json document delete --document-id "$document_id")

  deleted_document_id=$(echo "$cli_output" | jq -r '.deletedDocumentId')
  [[ "$deleted_document_id" == "$document_id" ]] || exit 1

  # Verify that the deleted document is no longer accessible
  # Fetch documents for the customer again
  cli_output=$("$LANACLI" --json document list --customer-id "$customer_id")

  # Check if the deleted document is not in the list
  deleted_document_exists=$(echo "$cli_output" | jq --arg id "$document_id" 'any(.[]; .id == $id)')
  [[ "$deleted_document_exists" == "false" ]] || exit 1

  cli_output=$("$LANACLI" --json document get --id "$document_id")
  [[ "$cli_output" == "null" ]] || exit 1
}
