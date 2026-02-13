#!/usr/bin/env bats

load "helpers"

RUN_LOG_FILE="customer.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

wait_for_approval() {
  variables=$(
    jq -n \
      --arg withdrawId "$1" \
    '{ id: $withdrawId }'
  )
  exec_admin_graphql 'find-withdraw' "$variables"
  echo "withdrawal | $i. $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.withdrawal.status')
  [[ "$status" == "PENDING_CONFIRMATION" ]] || return 1
}

@test "customer: can create a customer via prospect flow" {
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
  [[ "$prospect_id" != "null" ]] || exit 1

  # Verify customerType in prospect response
  response_customer_type=$(graphql_output .data.prospectCreate.prospect.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1

  # Simulate KYC start via SumSub applicantCreated webhook
  webhook_id="req-$(date +%s%N)"
  applicant_id="test-applicant-$webhook_id"
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantCreated",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  # Simulate KYC approval via SumSub webhook
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantReviewed",
      "reviewResult": { "reviewAnswer": "GREEN" },
      "reviewStatus": "completed",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  # Customer has the same ID as the prospect
  customer_id="$prospect_id"
  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer' "$variables"
  fetched_id=$(graphql_output .data.customer.customerId)
  [[ "$fetched_id" != "null" ]] || exit 1

  exec_admin_graphql 'customer-audit-log' "$variables"
  audit_nodes_count=$(graphql_output '.data.audit.nodes | length')
  [[ "$audit_nodes_count" -gt 0 ]] || exit 1
}

@test "customer: can login" {
  skip # does not work on concourse

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
  [[ "$prospect_id" != "null" ]] || exit 1

  # Simulate KYC start via SumSub applicantCreated webhook
  webhook_id="req-$(date +%s%N)"
  applicant_id="test-applicant-$webhook_id"
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantCreated",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  # Simulate KYC approval via SumSub webhook
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantReviewed",
      "reviewResult": { "reviewAnswer": "GREEN" },
      "reviewStatus": "completed",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  customer_id="$prospect_id"

  login_customer $customer_email
  exec_customer_graphql $customer_email 'me'
  echo $(graphql_output) | jq .
  [[ "$(graphql_output .data.me.customer.customerId)" == "$customer_id" ]] || exit 1

  response_customer_type=$(graphql_output .data.me.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1
}
