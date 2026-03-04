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

  local cli_output
  cli_output=$("$LANACLI" --json domain-config list)
  email_config_id=$(echo "$cli_output" | jq -r '[.[].node | select(.key == "notification-email-from-email").domainConfigId] | first')
  name_config_id=$(echo "$cli_output" | jq -r '[.[].node | select(.key == "notification-email-from-name").domainConfigId] | first')

  cli_output=$("$LANACLI" --json domain-config update \
    --domain-config-id "$email_config_id" \
    --value-json "\"$from_email\"")

  updated_email=$(echo "$cli_output" | jq -r '.value')
  [[ "$updated_email" == "$from_email" ]] || exit 1

  cli_output=$("$LANACLI" --json domain-config update \
    --domain-config-id "$name_config_id" \
    --value-json "\"$from_name\"")

  updated_name=$(echo "$cli_output" | jq -r '.value')
  [[ "$updated_name" == "$from_name" ]] || exit 1

  cli_output=$("$LANACLI" --json domain-config list)
  current_email=$(echo "$cli_output" | jq -r '[.[].node | select(.key == "notification-email-from-email").value] | first')
  current_name=$(echo "$cli_output" | jq -r '[.[].node | select(.key == "notification-email-from-name").value] | first')
  [[ "$current_email" == "$from_email" ]] || exit 1
  [[ "$current_name" == "$from_name" ]] || exit 1
}
