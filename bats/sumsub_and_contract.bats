#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "sumsub: integrate with gql" {
  if [[ -z "${LANA_DOMAIN_CONFIG_SUMSUB_API_KEY}" || -z "${LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET}" ]]; then
    skip "Skipping test because LANA_DOMAIN_CONFIG_SUMSUB_API_KEY or LANA_DOMAIN_CONFIG_SUMSUB_API_SECRET is not defined"
  fi

  customer_email=$(generate_email)
  telegramHandle=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    --arg telegramHandle "$telegramHandle" \
    --arg customerType "$customer_type" \
    '{
      input: {
        email: $email,
        telegramHandle: $telegramHandle,
        customerType: $customerType
      }
    }'
  )

  exec_admin_graphql 'prospect-create' "$variables"
  prospect_id=$(graphql_output .data.prospectCreate.prospect.prospectId)

  echo "prospect_id: $prospect_id"
  [[ "$prospect_id" != "null" ]] || exit 1

  # Create KYC link (for reference and fallback testing)

  variables=$(
    jq -n \
    --arg prospectId "$prospect_id" \
    '{
      input: {
        prospectId: $prospectId
      }
    }'
  )

  exec_admin_graphql 'prospect-kyc-link-create' "$variables"
  url=$(graphql_output .data.prospectKycLinkCreate.url)
  [[ "$url" != "null" ]] || exit 1
  echo "Created KYC link: $url"

  # Use a synthetic applicant ID for webhook simulation
  test_applicant_id="test-applicant-$(uuidgen)"
  echo "Using synthetic test applicant_id: $test_applicant_id"

  # Simulate Sumsub webhook callbacks since Sumsub can't reach our local server
  echo "Simulating applicantCreated webhook..."
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$test_applicant_id"'",
      "inspectionId": "test-inspection-id",
      "correlationId": "'"$(uuidgen)"'",
      "levelName": "basic-kyc-level",
      "externalUserId": "'"$prospect_id"'",
      "type": "applicantCreated",
      "sandboxMode": true,
      "reviewStatus": "init",
      "createdAtMs": "2024-10-05 13:23:19.002",
      "clientId": "testClientId"
    }'

  echo "Simulating applicantReviewed (GREEN) webhook..."
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$test_applicant_id"'",
      "inspectionId": "test-inspection-id",
      "correlationId": "'"$(uuidgen)"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantReviewed",
      "reviewResult": {
        "reviewAnswer": "GREEN"
      },
      "reviewStatus": "completed",
      "createdAtMs": "2024-10-05 13:23:19.003",
      "sandboxMode": true
    }'

  # Customer is created asynchronously via webhook inbox processing.
  # Poll until the customer exists.
  customer_id="$prospect_id"
  for i in {1..30}; do
    variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
    exec_admin_graphql 'customer' "$variables"
    fetched_id=$(graphql_output .data.customer.customerId)
    [[ "$fetched_id" != "null" ]] && break
    sleep 1
  done
  [[ "$fetched_id" != "null" ]] || exit 1

  # Verify the customer kyc verification after the complete KYC flow
  variables=$(jq -n --arg customerId "$customer_id" '{ id: $customerId }')

  exec_admin_graphql 'customer' "$variables"
  level=$(graphql_output '.data.customer.level')
  final_applicant_id=$(graphql_output '.data.customer.applicantId')

  # After kyc verification check
  echo "After test applicant creation - level: $level, applicant_id: $final_applicant_id"

  # The complete test applicant should result in BASIC level
  [[ "$level" == "BASIC" ]] || exit 1
  [[ "$final_applicant_id" == "$test_applicant_id" ]] || exit 1

  # Test webhook callback integration (original functionality)
  echo "Testing webhook callback functionality..."
  
  # Test intermediate webhook calls should not return 500
  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/webhook/sumsub \
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

  status_code=$(curl -s -o /dev/null -w "%{http_code}" http://localhost:5253/webhook/sumsub \
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
  curl -s -X POST http://localhost:5253/webhook/sumsub \
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

  variables=$(jq -n --arg customerId "$customer_id" '{ id: $customerId }')
  exec_admin_graphql 'customer' "$variables"

  level=$(graphql_output '.data.customer.level')
  status=$(graphql_output '.data.customer.status')

  echo "After rejection webhook - level: $level, status: $status"
  # After rejection, level should remain BASIC but customer should be FROZEN
  [[ "$level" == "BASIC" ]] || exit 1
  [[ "$status" == "FROZEN" ]] || exit 1
}
