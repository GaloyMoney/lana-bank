#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "domain-config: notification email config can update" {
  from_email="notifications@example.com"
  from_name="Notifier"

  variables=$(
    jq -n \
      --arg fromEmail "$from_email" \
      --arg fromName "$from_name" \
    '{
      input: {
        fromEmail: $fromEmail,
        fromName: $fromName
      }
    }'
  )

  exec_admin_graphql 'notification-email-config-update' "$variables"

  updated_email=$(graphql_output '.data.notificationEmailConfigUpdate.notificationEmailConfig.fromEmail')
  updated_name=$(graphql_output '.data.notificationEmailConfigUpdate.notificationEmailConfig.fromName')
  [[ "$updated_email" == "$from_email" ]] || exit 1
  [[ "$updated_name" == "$from_name" ]] || exit 1

  exec_admin_graphql 'notification-email-config'
  current_email=$(graphql_output '.data.notificationEmailConfig.fromEmail')
  current_name=$(graphql_output '.data.notificationEmailConfig.fromName')
  [[ "$current_email" == "$from_email" ]] || exit 1
  [[ "$current_name" == "$from_name" ]] || exit 1
}
