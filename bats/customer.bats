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

@test "customer: can create a customer" {
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
  
  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1
  
  # Verify customerType in response
  response_customer_type=$(graphql_output .data.customerCreate.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1

  variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
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

  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1

  login_customer $customer_email
  exec_customer_graphql $customer_email 'me'
  echo $(graphql_output) | jq .
  [[ "$(graphql_output .data.me.customer.customerId)" == "$customer_id" ]] || exit 1
  
  response_customer_type=$(graphql_output .data.me.customer.customerType)
  [[ "$response_customer_type" == "$customer_type" ]] || exit 1
}
