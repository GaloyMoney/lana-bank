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

@test "superuser: can create bank manager" {
  bank_manager_email=$(generate_email)

  # First get the bank-manager role ID
  local cli_output
  cli_output=$("$LANACLI" --json user roles-list)
  role_id=$(echo "$cli_output" | jq -r '.[] | select(.name == "bank-manager").roleId')
  [[ "$role_id" != "null" ]] || exit 1

  # Create user with email and roleId
  cli_output=$("$LANACLI" --json user create --email "$bank_manager_email" --role-id "$role_id")
  user_id=$(echo "$cli_output" | jq -r '.userId')
  [[ "$user_id" != "null" ]] || exit 1

  # Verify the user was created with the correct role
  role=$(echo "$cli_output" | jq -r '.role.name')
  [[ "$role" = "bank-manager" ]] || exit 1
}


@test "superuser: can create prospect and approve KYC" {
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
}
