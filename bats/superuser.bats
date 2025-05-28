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

  exec_admin_graphql 'list-roles'
  role_id=$(graphql_output ".data.roles.nodes[] | select(.name == \"bank-manager\").roleId")
  [[ "$role_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
    --arg userId "$user_id" --arg roleId "$role_id" \
    '{
      input: {
        id: $userId,
        roleId: $roleId
        }
      }'
  )

  exec_admin_graphql 'user-update-role' "$variables"
  role=$(graphql_output .data.userUpdateRole.user.role.name)
  [[ "$role" = "bank-manager" ]] || exit 1
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
