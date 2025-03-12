#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "documents: can upload a file, retrieve, archive, delete, and verify deletion" {
  if [[ -z "${SA_CREDS_BASE64}" ]]; then
    skip "Skipping test because SA_CREDS_BASE64 is not defined"
  fi

  # Create a customer
  customer_email=$(generate_email)
  telegramId=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    --arg telegramId "$telegramId" \
    --arg customerType "$customer_type" \
    '{
      input: {
        email: $email,
        telegramId: $telegramId,
        customerType: $customerType
      }
    }'
  )
  
  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1

  # Generate a temporary file
  temp_file=$(mktemp)
  echo "Test content" > "$temp_file"
  
  # Prepare the variables for file upload
  variables=$(jq -n \
    --arg customerId "$customer_id" \
    '{
      "customerId": $customerId,
      "file": null
    }')

  # Execute the GraphQL mutation for file upload
  response=$(exec_admin_graphql_upload "customer-document-attach" "$variables" "$temp_file")  
  document_id=$(echo "$response" | jq -r '.data.customerDocumentAttach.document.documentId')
  [[ "$document_id" != "" ]] || exit 1
  
  # Clean up the temporary file
  rm "$temp_file"

  # Fetch the document by ID
  variables=$(jq -n \
    --arg documentId "$document_id" \
    '{
      "id": $documentId
    }')

  exec_admin_graphql 'document' "$variables"
  fetched_document_id=$(graphql_output .data.document.documentId)
  [[ "$fetched_document_id" == "$document_id" ]] || exit 1

  fetched_customer_id=$(graphql_output .data.document.customerId)
  [[ "$fetched_customer_id" == "$customer_id" ]] || exit 1

  # Fetch documents for the customer
  variables=$(jq -n \
    --arg customerId "$customer_id" \
    '{
      "customerId": $customerId
    }')

  exec_admin_graphql 'documents-for-customer' "$variables"

  documents_count=$(graphql_output '.data.customer.documents | length')
  [[ "$documents_count" -ge 1 ]] || exit 1

  first_document_id=$(graphql_output '.data.customer.documents[0].documentId')
  [[ "$first_document_id" == "$document_id" ]] || exit 1

  # Generate download link for the document
  variables=$(jq -n \
    --arg documentId "$document_id" \
    '{
      input: {
        documentId: $documentId
      }
    }')

  exec_admin_graphql 'document-download-link-generate' "$variables"

  download_link=$(graphql_output .data.documentDownloadLinkGenerate.link)
  [[ "$download_link" != "null" && "$download_link" != "" ]] || exit 1

  response=$(curl -s -o /dev/null -w "%{http_code}" "$download_link")
  [[ "$response" == "200" ]] || exit 1

  # archive the document
  variables=$(jq -n \
    --arg documentId "$document_id" \
    '{
      input: {
        documentId: $documentId
      }
    }')

  exec_admin_graphql 'document-archive' "$variables"

  status=$(graphql_output .data.documentArchive.document.status)
  [[ "$status" == "ARCHIVED" ]] || exit 1

  # Delete the document
  variables=$(jq -n \
    --arg documentId "$document_id" \
    '{
      input: {
        documentId: $documentId
      }
    }')

  exec_admin_graphql 'document-delete' "$variables"

  deleted_document_id=$(graphql_output .data.documentDelete.deletedDocumentId)
  [[ "$deleted_document_id" == "$document_id" ]] || exit 1

  # Verify that the deleted document is no longer accessible
  # Fetch documents for the customer again
  variables=$(jq -n \
    --arg customerId "$customer_id" \
    '{
      "customerId": $customerId
    }')

  exec_admin_graphql 'documents-for-customer' "$variables"

  # Check if the deleted document is not in the list
  documents=$(graphql_output '.data.customer.documents')
  deleted_document_exists=$(echo "$documents" | jq --arg id "$document_id" 'any(.[]; .id == $id)')
  [[ "$deleted_document_exists" == "false" ]] || exit 1

  variables=$(jq -n \
    --arg documentId "$document_id" \
    '{
      "id": $documentId
    }')

  exec_admin_graphql 'document' "$variables"
  document=$(graphql_output '.document')
  [[ "$document" == "null" ]] || exit 1
}
