#!/usr/bin/env bats

load "helpers"

RUN_LOG_FILE="customer.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

wait_for_approval() {
  local cli_output
  cli_output=$("$LANACLI" --json withdrawal find --id "$1")
  echo "withdrawal | $i. $cli_output" >> $RUN_LOG_FILE
  status=$(echo "$cli_output" | jq -r '.status')
  [[ "$status" == "PENDING_CONFIRMATION" ]] || return 1
}

@test "customer: can create a customer via prospect flow" {
  customer_email=$(generate_email)
  telegramHandle=$(generate_email)
  customer_type="INDIVIDUAL"

  local cli_output
  cli_output=$("$LANACLI" --json prospect create \
    --email "$customer_email" \
    --telegram-handle "$telegramHandle" \
    --customer-type "$customer_type")
  prospect_id=$(echo "$cli_output" | jq -r '.prospectId')
  [[ "$prospect_id" != "null" && -n "$prospect_id" ]] || exit 1

  # Verify customerType in prospect response
  response_customer_type=$(echo "$cli_output" | jq -r '.customerType')
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
      "reviewStatus": "init",
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

  # Customer is created asynchronously via webhook inbox processing.
  # Poll until the customer exists.
  customer_id="$prospect_id"
  for i in {1..30}; do
    cli_output=$("$LANACLI" --json customer get --id "$customer_id" 2>/dev/null || echo '{}')
    fetched_id=$(echo "$cli_output" | jq -r '.customerId // empty')
    [[ -n "$fetched_id" ]] && break
    sleep 1
  done
  [[ -n "$fetched_id" ]] || exit 1

  local audit_output
  audit_output=$("$LANACLI" --json audit customer --id "$customer_id")
  audit_nodes_count=$(echo "$audit_output" | jq '. | length')
  [[ "$audit_nodes_count" -gt 0 ]] || exit 1
}

@test "customer: can login" {
  skip # does not work on concourse

  customer_email=$(generate_email)
  telegramHandle=$(generate_email)
  customer_type="INDIVIDUAL"

  local cli_output
  cli_output=$("$LANACLI" --json prospect create \
    --email "$customer_email" \
    --telegram-handle "$telegramHandle" \
    --customer-type "$customer_type")
  prospect_id=$(echo "$cli_output" | jq -r '.prospectId')
  [[ "$prospect_id" != "null" && -n "$prospect_id" ]] || exit 1

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
      "reviewStatus": "init",
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

  # Customer is created asynchronously via webhook inbox processing.
  # Poll until the customer exists.
  customer_id="$prospect_id"
  for i in {1..30}; do
    cli_output=$("$LANACLI" --json customer get --id "$customer_id" 2>/dev/null || echo '{}')
    fetched_id=$(echo "$cli_output" | jq -r '.customerId // empty')
    [[ -n "$fetched_id" ]] && break
    sleep 1
  done
  [[ -n "$fetched_id" ]] || exit 1

  login_customer $customer_email
  exec_customer_graphql $customer_email 'me'
  echo $(graphql_output) | jq .
  [[ "$(graphql_output .data.me.customer.customerId)" == "$customer_id" ]] || exit 1

  response_customer_type=$(graphql_output .data.me.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1
}
