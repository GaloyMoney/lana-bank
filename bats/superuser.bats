#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "superuser: can create bank manager" {
  bank_manager_email=$(generate_email)

  # First get the bank-manager role ID
  exec_admin_graphql 'list-roles'
  role_id=$(graphql_output ".data.roles.nodes[] | select(.name == \"bank-manager\").roleId")
  [[ "$role_id" != "null" ]] || exit 1

  # Create user with email and roleId
  variables=$(
    jq -n \
    --arg email "$bank_manager_email" --arg roleId "$role_id" \
    '{
      input: {
        email: $email,
        roleId: $roleId
        }
      }'
  )

  exec_admin_graphql 'user-create' "$variables"
  user_id=$(graphql_output .data.userCreate.user.userId)
  [[ "$user_id" != "null" ]] || exit 1

  # Verify the user was created with the correct role
  role=$(graphql_output .data.userCreate.user.role.name)
  [[ "$role" = "bank-manager" ]] || exit 1
}


@test "superuser: can create prospect and approve KYC" {
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

  # Customer has the same ID as the prospect
  customer_id="$prospect_id"
  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer' "$variables"
  fetched_id=$(graphql_output .data.customer.customerId)
  [[ "$fetched_id" != "null" ]] || exit 1
}
