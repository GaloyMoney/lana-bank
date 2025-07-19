#!/usr/bin/env bats

load "helpers"

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
  status=$(graphql_output '.data.loanAgreement.status')
  [[ "$status" == "COMPLETED" ]] || return 1
}

@test "sumsub: integrate with gql" {
  if [[ -z "${SUMSUB_KEY}" || -z "${SUMSUB_SECRET}" ]]; then
    skip "Skipping test because SUMSUB_KEY or SUMSUB_SECRET is not defined"
  fi

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

  echo "customer_id: $customer_id"
  [[ "$customer_id" != "null" ]] || exit 1

  # Create permalink (for reference and fallback testing)

  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId
      }
    }'
  )

  exec_admin_graphql 'sumsub-permalink-create' "$variables"
  url=$(graphql_output .data.sumsubPermalinkCreate.url)
  [[ "$url" != "null" ]] || exit 1
  echo "Created permalink: $url"

  # Test complete KYC flow via GraphQL mutation
  echo "Testing complete KYC flow via sumsubTestApplicantCreate..."
  exec_admin_graphql 'sumsub-test-applicant-create' "$variables"
  echo "graphql_output: $(graphql_output)"

  test_applicant_id=$(graphql_output .data.sumsubTestApplicantCreate.applicantId)

  echo "Created test applicant_id: $test_applicant_id"
  [[ "$test_applicant_id" != "null" ]] || exit 1
  [[ -n "$test_applicant_id" ]] || exit 1

  # Simulate Sumsub webhook callbacks since Sumsub can't reach our local server
  echo "Simulating applicantCreated webhook..."
  curl -s -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$test_applicant_id"'",
      "inspectionId": "test-inspection-id",
      "correlationId": "test-correlation-id",
      "levelName": "basic-kyc-level",
      "externalUserId": "'"$customer_id"'",
      "type": "applicantCreated",
      "sandboxMode": true,
      "reviewStatus": "init",
      "createdAtMs": "2024-10-05 13:23:19.002",
      "clientId": "testClientId"
    }'

  echo "Simulating applicantReviewed (GREEN) webhook..."
  curl -s -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$test_applicant_id"'",
      "inspectionId": "test-inspection-id",
      "correlationId": "test-correlation-id",
      "externalUserId": "'"$customer_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantReviewed",
      "reviewResult": {
        "reviewAnswer": "GREEN"
      },
      "reviewStatus": "completed",
      "createdAtMs": "2024-10-05 13:23:19.002",
      "sandboxMode": true
    }'

  # Wait briefly for webhook processing
  echo "Waiting for webhook processing..."
  sleep 1

  # Verify the customer status after the complete KYC flow
  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  
  exec_admin_graphql 'customer' "$variables"
  level=$(graphql_output '.data.customer.level')
  status=$(graphql_output '.data.customer.status')
  final_applicant_id=$(graphql_output '.data.customer.applicantId')

  # After status check
  echo "After test applicant creation - level: $level, status: $status, applicant_id: $final_applicant_id"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{ input: { customerId: $customerId } }'
  )
  
  exec_admin_graphql 'loan-agreement-generate' "$variables"  
  
  loan_agreement_id=$(graphql_output '.data.loanAgreementGenerate.loanAgreement.id')
  [[ "$loan_agreement_id" != "null" ]] || exit 1
  [[ "$loan_agreement_id" != "" ]] || exit 1
  
  status=$(graphql_output '.data.loanAgreementGenerate.loanAgreement.status')
  [[ "$status" == "PENDING" ]] || exit 1
  
  retry 30 2 wait_for_loan_agreement_completion $loan_agreement_id
  
  variables=$(
    jq -n \
      --arg loanAgreementId "$loan_agreement_id" \
    '{ input: { loanAgreementId: $loanAgreementId } }'
  )
  
  exec_admin_graphql 'loan-agreement-download-link-generate' "$variables"
  
  download_link=$(graphql_output '.data.loanAgreementDownloadLinkGenerate.link')
  returned_loan_agreement_id=$(graphql_output '.data.loanAgreementDownloadLinkGenerate.loanAgreementId')
  
  [[ "$download_link" != "null" ]] || exit 1
  [[ "$download_link" != "" ]] || exit 1
  [[ "$returned_loan_agreement_id" == "$loan_agreement_id" ]] || exit 1
  
  temp_pdf="/tmp/loan_agreement_${loan_agreement_id}.pdf"
  temp_txt="/tmp/loan_agreement_${loan_agreement_id}.txt"
  
  if [[ "$download_link" =~ ^file:// ]]; then
    file_path="${download_link#file://}"
    cp "$file_path" "$temp_pdf" || exit 1
  else
    curl -s -o "$temp_pdf" "$download_link" || exit 1
  fi
  
  [[ -f "$temp_pdf" ]] || exit 1
  file_size=$(stat -f%z "$temp_pdf" 2>/dev/null || stat -c%s "$temp_pdf" 2>/dev/null)
  [[ "$file_size" -gt 0 ]] || exit 1
  
  file_header=$(head -c 4 "$temp_pdf")
  [[ "$file_header" == "%PDF" ]] || exit 1
  
  pdftotext "$temp_pdf" "$temp_txt" || exit 1
  
  echo "temp_txt: $temp_txt"
  grep "John Mock-Doe" "$temp_txt" || exit 1
  grep "DEU" "$temp_txt" || exit 1
  
  rm -f "$temp_pdf" "$temp_txt"

  # The complete test applicant should result in BASIC level and ACTIVE status
  [[ "$level" == "BASIC" ]] || exit 1
  [[ "$status" == "ACTIVE" ]] || exit 1
  [[ "$final_applicant_id" == "$test_applicant_id" ]] || exit 1

  # Test webhook callback integration (original functionality)
  echo "Testing webhook callback functionality..."
  
  # Test intermediate webhook calls should not return 500
  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "66f1f52c27a518786597c113",
      "inspectionId": "66f1f52c27a518786597c113",
      "applicantType": "individual",
      "correlationId": "feb6317b2f13441784668eaa87dd14ef",
      "levelName": "basic-kyc-level",
      "sandboxMode": true,
      "externalUserId": "'"$customer_id"'",
      "type": "applicantPending",
      "reviewStatus": "pending",
      "createdAt": "2024-09-23 23:10:24+0000",
      "createdAtMs": "2024-09-23 23:10:24.704",
      "clientId": "galoy.io"
  }')

  [[ "$status_code" -eq 200 ]] || exit 1

  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
    "applicantId": "66f1f52c27a518786597c113",
    "inspectionId": "66f1f52c27a518786597c113",
    "applicantType": "individual",
    "correlationId": "feb6317b2f13441784668eaa87dd14ef",
    "levelName": "basic-kyc-level",
    "sandboxMode": true,
    "externalUserId": "'"$customer_id"'",
    "type": "applicantPersonalInfoChanged",
    "reviewStatus": "pending",
    "createdAt": "2024-09-23 23:10:24+0000",
    "createdAtMs": "2024-09-23 23:10:24.763",
    "clientId": "galoy.io"
  }')

  [[ "$status_code" -eq 200 ]] || exit 1

  # Test rejection webhook (should change status back to INACTIVE)
  echo "Testing rejection webhook with actual applicant ID..."
  curl -s -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "'"$test_applicant_id"'",
        "inspectionId": "5cb744200a975a67ed1798a5",
        "correlationId": "req-fa94263f-0b23-42d7-9393-ab10b28ef42d",
        "externalUserId": "'"$customer_id"'",
        "levelName": "basic-kyc-level",
        "type": "applicantReviewed",
        "reviewResult": {
            "moderationComment": "We could not verify your profile. If you have any questions, please contact the Company where you try to verify your profile ${clientSupportEmail}",
            "clientComment": "Suspected fraudulent account.",
            "reviewAnswer": "RED",
            "rejectLabels": ["UNSATISFACTORY_PHOTOS", "GRAPHIC_EDITOR", "FORGERY"],
            "reviewRejectType": "FINAL"
        },
        "reviewStatus": "completed",
        "createdAtMs": "2020-02-21 13:23:19.001"
    }'

  exec_admin_graphql 'customer' "$variables"

  level=$(graphql_output '.data.customer.level')
  status=$(graphql_output '.data.customer.status')

  echo "After rejection webhook - level: $level, status: $status"
  # After rejection, level should remain BASIC but status should become INACTIVE
  [[ "$level" == "BASIC" ]] || exit 1
  [[ "$status" == "INACTIVE" ]] || exit 1
}

@test "sumsub: sandbox mode with random customer ID should return 200" {
  random_customer_id=$(uuidgen)

  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "random_applicant_id",
      "inspectionId": "random_inspection_id",
      "correlationId": "random_correlation_id",
      "levelName": "basic-kyc-level",
      "externalUserId": "'"$random_customer_id"'",
      "type": "applicantCreated",
      "sandboxMode": true,
      "reviewStatus": "init",
      "createdAtMs": "2024-10-05 13:23:19.002",
      "clientId": "testClientId"
    }')

  echo "Status code: $status_code"
  [[ "$status_code" -eq 200 ]] || exit 1
}

@test "sumsub: non-sandbox mode with random customer ID should return 500" {
  random_customer_id=$(uuidgen)

  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "random_applicant_id",
      "inspectionId": "random_inspection_id",
      "correlationId": "random_correlation_id",
      "levelName": "basic-kyc-level",
      "externalUserId": "'"$random_customer_id"'",
      "type": "applicantCreated",
      "sandboxMode": false,
      "reviewStatus": "init",
      "createdAtMs": "2024-10-05 13:23:19.002",
      "clientId": "testClientId"
    }')

  echo "Status code: $status_code"
  [[ "$status_code" -eq 500 ]] || exit 1
}


