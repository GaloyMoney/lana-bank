#!/usr/bin/env bats

load "helpers"

RUN_LOG_FILE="contract-generation.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

wait_for_loan_agreement_completion() {
  variables=$(
    jq -n \
      --arg loanAgreementId "$1" \
    '{ id: $loanAgreementId }'
  )
  exec_admin_graphql 'find-loan-agreement' "$variables"
  echo "loan agreement | $i. $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.loanAgreement.status')
  [[ "$status" == "COMPLETED" ]] || return 1
}

@test "contract-generation: can generate loan agreement and download PDF" {
  customer_id=$(create_customer)

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId
      }
    }'
  )
  
  exec_admin_graphql 'loan-agreement-generate' "$variables"  
  echo "loan agreement generation | $(graphql_output)" >> $RUN_LOG_FILE
  
  loan_agreement_id=$(graphql_output '.data.loanAgreementGenerate.loanAgreement.id')
  [[ "$loan_agreement_id" != "null" ]] || exit 1
  [[ "$loan_agreement_id" != "" ]] || exit 1
  
  status=$(graphql_output '.data.loanAgreementGenerate.loanAgreement.status')
  [[ "$status" == "PENDING" ]] || exit 1
  
  retry 30 2 wait_for_loan_agreement_completion $loan_agreement_id
  
  variables=$(
    jq -n \
      --arg loanAgreementId "$loan_agreement_id" \
    '{
      input: {
        loanAgreementId: $loanAgreementId
      }
    }'
  )
  
  exec_admin_graphql 'loan-agreement-download-link-generate' "$variables"
  echo "download link generation | $(graphql_output)" >> $RUN_LOG_FILE
  
  download_link=$(graphql_output '.data.loanAgreementDownloadLinkGenerate.link')
  returned_loan_agreement_id=$(graphql_output '.data.loanAgreementDownloadLinkGenerate.loanAgreementId')
  
  [[ "$download_link" != "null" ]] || exit 1
  [[ "$download_link" != "" ]] || exit 1
  [[ "$returned_loan_agreement_id" == "$loan_agreement_id" ]] || exit 1
  
  temp_pdf="/tmp/loan_agreement_${loan_agreement_id}.pdf"
  temp_txt="/tmp/loan_agreement_${loan_agreement_id}.txt"
  
  # Handle file:// URLs by copying the file directly
  if [[ "$download_link" =~ ^file:// ]]; then
    # Remove file:// prefix and copy the file
    file_path="${download_link#file://}"
    cp "$file_path" "$temp_pdf" || exit 1
  else
    # Use curl for HTTP/HTTPS URLs
    curl -s -o "$temp_pdf" "$download_link" || exit 1
  fi
  
  [[ -f "$temp_pdf" ]] || exit 1
  file_size=$(stat -f%z "$temp_pdf" 2>/dev/null || stat -c%s "$temp_pdf" 2>/dev/null)
  [[ "$file_size" -gt 0 ]] || exit 1
  
  file_header=$(head -c 4 "$temp_pdf")
  [[ "$file_header" == "%PDF" ]] || exit 1
  
  pdftotext "$temp_pdf" "$temp_txt" || exit 1
  grep -i "loan agreement" "$temp_txt" || exit 1
  
  rm -f "$temp_pdf" "$temp_txt"
} 