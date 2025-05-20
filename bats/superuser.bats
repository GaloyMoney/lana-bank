#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server_nix
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "superuser: can create bank manager" {
  bank_manager_email=$(generate_email)

  variables=$(
    jq -n \
    --arg email "$bank_manager_email" \
    '{
      input: {
        email: $email
        }
      }'
  )

  exec_admin_graphql 'user-create' "$variables"
  user_id=$(graphql_output .data.userCreate.user.userId)
  [[ "$user_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
    --arg userId "$user_id" \
    '{
      input: {
        id: $userId,
        role: "BANK_MANAGER"
        }
      }'
  )

  exec_admin_graphql 'user-assign-role' "$variables" 
  role=$(graphql_output .data.userAssignRole.user.roles[0])
  [[ "$role" = "BANK_MANAGER" ]] || exit 1
}


@test "superuser: can create customer" {
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
  [[ "$customer_id" != "null" ]] || exit 1
}
