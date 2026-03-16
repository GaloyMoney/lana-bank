#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "domain-config: notification email configs can update" {
  from_email="notifications@example.com"
  from_name="Notifier"

  exec_admin_graphql 'domain-configs'
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

  exec_admin_graphql 'domain-config-update' "$variables"

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

  exec_admin_graphql 'domain-config-update' "$variables"

  updated_name=$(graphql_output '.data.domainConfigUpdate.domainConfig.value')
  [[ "$updated_name" == "$from_name" ]] || exit 1

  exec_admin_graphql 'domain-configs'
  current_email=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-email").value')
  current_name=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "notification-email-from-name").value')
  [[ "$current_email" == "$from_email" ]] || exit 1
  [[ "$current_name" == "$from_name" ]] || exit 1
}

@test "domain-config: time can advance to next end of day" {
  exec_admin_graphql 'domain-configs'
  timezone_config_id=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "timezone").domainConfigId')
  closing_time_config_id=$(graphql_output '.data.domainConfigs.edges[].node | select(.key == "closing-time").domainConfigId')

  timezone_variables=$(
    jq -n \
      --arg id "$timezone_config_id" \
      --arg value "UTC" \
      '{
        input: {
          domainConfigId: $id,
          value: $value
        }
      }'
  )
  exec_admin_graphql 'domain-config-update' "$timezone_variables"

  closing_time_variables=$(
    jq -n \
      --arg id "$closing_time_config_id" \
      --arg value "00:00:00" \
      '{
        input: {
          domainConfigId: $id,
          value: $value
        }
      }'
  )
  exec_admin_graphql 'domain-config-update' "$closing_time_variables"

  exec_admin_graphql 'time'
  initial_date=$(graphql_output '.data.time.currentDate')
  initial_current_time=$(graphql_output '.data.time.currentTime')
  initial_next_eod=$(graphql_output '.data.time.nextEndOfDayAt')
  can_advance=$(graphql_output '.data.time.canAdvanceToNextEndOfDay')

  [[ "$initial_date" == "2024-01-01" ]] || exit 1
  [[ "$initial_current_time" == "2024-01-01T00:00:00Z" ]] || exit 1
  [[ "$initial_next_eod" == "2024-01-02T00:00:00Z" ]] || exit 1
  [[ "$can_advance" == "true" ]] || exit 1

  exec_admin_graphql 'time-advance-to-next-end-of-day'
  advanced_date=$(graphql_output '.data.timeAdvanceToNextEndOfDay.time.currentDate')
  advanced_current_time=$(graphql_output '.data.timeAdvanceToNextEndOfDay.time.currentTime')
  advanced_next_eod=$(graphql_output '.data.timeAdvanceToNextEndOfDay.time.nextEndOfDayAt')

  [[ "$advanced_date" == "2024-01-02" ]] || exit 1
  [[ "$advanced_current_time" == "2024-01-02T00:00:00Z" ]] || exit 1
  [[ "$advanced_next_eod" == "2024-01-03T00:00:00Z" ]] || exit 1
}
