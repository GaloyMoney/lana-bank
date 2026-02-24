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

@test "domain-config: notification email configs can update" {
  from_email="notifications@example.com"
  from_name="Notifier"

  exec_admin_graphql 'notification-email-config'
  email_config_id=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-email").domainConfigId')
  name_config_id=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-name").domainConfigId')

  variables=$(
    jq -n \
      --arg id "$email_config_id" \
      --arg value "$from_email" \
    '{
      input: {
        domainConfigId: $id,
        value: $value
      }
    }'
  )

  exec_admin_graphql 'notification-email-config-update' "$variables"

  updated_email=$(graphql_output '.data.domainConfigUpdate.domainConfig.value')
  [[ "$updated_email" == "$from_email" ]] || exit 1

  variables=$(
    jq -n \
      --arg id "$name_config_id" \
      --arg value "$from_name" \
    '{
      input: {
        domainConfigId: $id,
        value: $value
      }
    }'
  )

  exec_admin_graphql 'notification-email-config-update' "$variables"

  updated_name=$(graphql_output '.data.domainConfigUpdate.domainConfig.value')
  [[ "$updated_name" == "$from_name" ]] || exit 1

  exec_admin_graphql 'notification-email-config'
  current_email=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-email").value')
  current_name=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-name").value')
  [[ "$current_email" == "$from_email" ]] || exit 1
  [[ "$current_name" == "$from_name" ]] || exit 1
}
