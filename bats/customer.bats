#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "customer: unauthorized" {
  cache_value "alice" "invalid-token"
  exec_graphql 'alice' 'me'
  error_code=$(graphql_output '.error.code')
  [[ "$error_code" == 401 ]] || exit 1

  error_status=$(graphql_output '.error.status')
  [[ "$error_status" == "Unauthorized" ]] || exit 1
}

@test "customer: can create a customer" {
  customer_email=$(generate_email)

  variables=$(
    jq -n \
    --arg email "$customer_email" \
    '{
      input: {
        email: $email
        }
      }'
  )

  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1

  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
  exec_admin_graphql 'customer-audit-log' "$variables"
  echo $(graphql_output) | jq .

  audit_entries=$(graphql_output '.data.customer.audit')
  [[ "$audit_entries" != "null" ]] || exit 1

  action=$(graphql_output '.data.customer.audit[0].action')
  [[ "$action" == "customer-create" ]] || exit 1

  authorized=$(graphql_output '.data.customer.audit[0].authorized')
  [[ "$authorized" == "true" ]] || exit 1
}

@test "customer: can deposit" {
  customer_id=$(create_customer)
  cache_value "customer_id" $customer_id

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"
  deposit_id=$(graphql_output '.data.depositRecord.deposit.depositId')
  [[ "$deposit_id" != "null" ]] || exit 1
  echo $(graphql_output) | jq .

  usd_balance=$(graphql_output '.data.depositRecord.deposit.customer.balance.checking.settled')
  [[ "$usd_balance" == "150000" ]] || exit 1
}

@test "customer: withdraw can be cancelled" {
  customer_id=$(read_value "customer_id")

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
        id: $customerId
    }'
  )

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalInitiate.withdrawal.status')
  [[ "$status" == "INITIATED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "150000" ]] || exit 1

  assert_accounts_balanced

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
    '{
      input: {
        withdrawalId: $withdrawalId
      }
    }'
  )
  exec_admin_graphql 'withdrawal-cancel' "$variables"
  echo $(graphql_output) | jq .

  withdrawal_id=$(graphql_output '.data.withdrawalCancel.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalCancel.withdrawal.status')
  [[ "$status" == "CANCELLED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "150000" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalCancel.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  assert_accounts_balanced
}

@test "customer: can withdraw" {
  customer_id=$(read_value "customer_id")

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        customerId: $customerId,
        amount: 150000,
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_admin_graphql 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalInitiate.withdrawal.status')
  [[ "$status" == "INITIATED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalInitiate.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "150000" ]] || exit 1

  assert_accounts_balanced

  variables=$(
    jq -n \
      --arg withdrawalId "$withdrawal_id" \
    '{
      input: {
        withdrawalId: $withdrawalId
      }
    }'
  )
  exec_admin_graphql 'confirm-withdrawal' "$variables"

  echo $(graphql_output)
  withdrawal_id=$(graphql_output '.data.withdrawalConfirm.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1
  status=$(graphql_output '.data.withdrawalConfirm.withdrawal.status')
  [[ "$status" == "CONFIRMED" ]] || exit 1
  settled_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.settled')
  [[ "$settled_usd_balance" == "0" ]] || exit 1
  pending_usd_balance=$(graphql_output '.data.withdrawalConfirm.withdrawal.customer.balance.checking.pending')
  [[ "$pending_usd_balance" == "0" ]] || exit 1

  assert_accounts_balanced
}

@test "customer: verify level 2" {
  skip
# TODO: mock this call
  exec_graphql 'alice' 'sumsub-token-create'
  token=$(echo "$output" | jq -r '.data.sumsubTokenCreate.token')
  # [[ "$token" != "null" ]] || exit 1

# TODO: mock this call
  exec_graphql 'alice' 'sumsub-permalink-create'
  echo "$output"

  exec_graphql 'alice' 'me'
  echo "$output"
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ZERO" ]] || exit 1

  user_id=$(graphql_output '.data.me.customerId')
  [[ "$user_id" != "null" ]] || exit 1

  status=$(graphql_output '.data.me.status')
  [[ "$status" == "INACTIVE" ]] || exit 1

  curl -v -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "5c9e177b0a975a6eeccf5960",
        "inspectionId": "5c9e177b0a975a6eeccf5961",
        "correlationId": "req-63f92830-4d68-4eee-98d5-875d53a12258",
        "levelName": "basic-kyc-level",
        "externalUserId": "'"$user_id"'",
        "type": "applicantCreated",
        "sandboxMode": "false",
        "reviewStatus": "init",
        "createdAtMs": "2020-02-21 13:23:19.002",
        "clientId": "coolClientId"
    }'

  exec_graphql 'alice' 'me'
  echo "$output"

  applicant_id=$(graphql_output '.data.me.applicantId')
  [[ "$applicant_id" != "null" ]] || exit 1

  applicant_id=$(graphql_output '.data.me.applicantId')
  [[ "$applicant_id" != "null" ]] || exit 1

  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ZERO" ]] || exit 1

    status=$(graphql_output '.data.me.status')
  [[ "$status" == "INACTIVE" ]] || exit 1

  # accepted
  curl -v -X POST http://localhost:5253/sumsub/callback \
      -H "Content-Type: application/json" \
      -d '{
          "applicantId": "5cb56e8e0a975a35f333cb83",
          "inspectionId": "5cb56e8e0a975a35f333cb84",
          "correlationId": "req-a260b669-4f14-4bb5-a4c5-ac0218acb9a4",
          "externalUserId": "'"$user_id"'",
          "levelName": "basic-kyc-level",
          "type": "applicantReviewed",
          "reviewResult": {
            "reviewAnswer": "GREEN"
          },
          "reviewStatus": "completed",
          "createdAtMs": "2020-02-21 13:23:19.321"
      }'

  exec_graphql 'alice' 'me'
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ONE" ]] || exit 1

    status=$(graphql_output '.data.me.status')
  [[ "$status" == "ACTIVE" ]] || exit 1

  # declined
  curl -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "5cb744200a975a67ed1798a4",
        "inspectionId": "5cb744200a975a67ed1798a5",
        "correlationId": "req-fa94263f-0b23-42d7-9393-ab10b28ef42d",
        "externalUserId": "'"$user_id"'",
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

  exec_graphql 'alice' 'me'
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ONE" ]] || exit 1

  status=$(graphql_output '.data.me.status')
  [[ "$status" == "INACTIVE" ]] || exit 1

}
