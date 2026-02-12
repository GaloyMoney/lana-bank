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


@test "superuser: can create customer" {
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
}
