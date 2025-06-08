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

# TODO: mock this call
# this is a end user api call. ignoring for now.
# exec_graphql 'alice' 'sumsub-token-create'
# token=$(echo "$output" | jq -r '.data.sumsubTokenCreate.token')
# [[ "$token" != "null" ]] || exit 1

# TODO: mock this call

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

  curl -v http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "5c9e177b0a975a6eeccf5960",
        "inspectionId": "5c9e177b0a975a6eeccf5961",
        "correlationId": "req-63f92830-4d68-4eee-98d5-875d53a12258",
        "levelName": "basic-kyc-level",
        "externalUserId": "'"$customer_id"'",
        "type": "applicantCreated",
        "sandboxMode": false,
        "reviewStatus": "init",
        "createdAtMs": "2020-02-21 13:23:19.002",
        "clientId": "coolClientId"
    }'


  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  exec_admin_graphql 'customer' "$variables"
  applicant_id=$(graphql_output '.data.customer.applicantId')
  [[ "$applicant_id" != "null" ]] || exit 1

  level=$(graphql_output '.data.customer.level')
  [[ "$level" == "NOT_KYCED" ]] || exit 1

    status=$(graphql_output '.data.customer.status')
  [[ "$status" == "INACTIVE" ]] || exit 1

  # should ignore intermadiary call without returning 500
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

  # accepted
  curl -v -X POST http://localhost:5253/sumsub/callback \
      -H "Content-Type: application/json" \
      -d '{
          "applicantId": "5cb56e8e0a975a35f333cb83",
          "inspectionId": "5cb56e8e0a975a35f333cb84",
          "correlationId": "req-a260b669-4f14-4bb5-a4c5-ac0218acb9a4",
          "externalUserId": "'"$customer_id"'",
          "levelName": "basic-kyc-level",
          "type": "applicantReviewed",
          "reviewResult": {
            "reviewAnswer": "GREEN"
          },
          "reviewStatus": "completed",
          "createdAtMs": "2020-02-21 13:23:19.321"
      }'

  exec_admin_graphql 'customer' "$variables"

  level=$(graphql_output '.data.customer.level')
  [[ "$level" == "BASIC" ]] || exit 1

    status=$(graphql_output '.data.customer.status')
  [[ "$status" == "ACTIVE" ]] || exit 1

  # declined
  curl -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "5cb744200a975a67ed1798a4",
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
  [[ "$level" == "BASIC" ]] || exit 1

  status=$(graphql_output '.data.customer.status')
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
