#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "domain-config: notification email exposed configs can update" {
  from_email="notifications@example.com"
  from_name="Notifier"

  variables=$(
    jq -n \
      --arg key "notification-email-from-email" \
      --arg value "$from_email" \
    '{
      input: {
        key: $key,
        value: $value
      }
    }'
  )

  exec_admin_graphql 'notification-email-config-update' "$variables"

  updated_email=$(graphql_output '.data.updateExposedConfig.exposedConfig.value')
  [[ "$updated_email" == "$from_email" ]] || exit 1

  variables=$(
    jq -n \
      --arg key "notification-email-from-name" \
      --arg value "$from_name" \
    '{
      input: {
        key: $key,
        value: $value
      }
    }'
  )

  exec_admin_graphql 'notification-email-config-update' "$variables"

  updated_name=$(graphql_output '.data.updateExposedConfig.exposedConfig.value')
  [[ "$updated_name" == "$from_name" ]] || exit 1

  exec_admin_graphql 'notification-email-config'
  current_email=$(graphql_output '.data.listExposedConfigs[] | select(.key == "notification-email-from-email").value')
  current_name=$(graphql_output '.data.listExposedConfigs[] | select(.key == "notification-email-from-name").value')
  [[ "$current_email" == "$from_email" ]] || exit 1
  [[ "$current_name" == "$from_name" ]] || exit 1
}
