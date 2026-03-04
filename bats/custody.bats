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

@test "custody: can create custodian" {
  name="test-komainu-$(date +%s)"
  api_key="test-api-key-$(date +%s)"
  api_secret="test-api-secret-$(date +%s)"
  secret_key="test-secret-key-$(date +%s)"
  webhook_secret="test-webhook-secret-$(date +%s)"

  input_json=$(
    jq -n \
    --arg name "$name" \
    --arg apiKey "$api_key" \
    --arg apiSecret "$api_secret" \
    --arg secretKey "$secret_key" \
    --arg webhookSecret "$webhook_secret" \
    '{
      komainu: {
        name: $name,
        apiKey: $apiKey,
        apiSecret: $apiSecret,
        testingInstance: true,
        secretKey: $secretKey,
        webhookSecret: $webhookSecret
      }
    }'
  )

  local cli_output
  cli_output=$("$LANACLI" --json custodian create --input-json "$input_json")
  custodian_id=$(echo "$cli_output" | jq -r '.custodianId')
  [[ "$custodian_id" != "null" ]] || exit 1

  cache_value "custodian_id" "$custodian_id"
}

@test "custody: can update custodian config" {
  custodian_id=$(read_value "custodian_id")

  name="test-komainu-$(date +%s)"
  new_api_key="updated-api-key-$(date +%s)"
  new_api_secret="updated-api-secret-$(date +%s)"
  new_secret_key="updated-secret-key-$(date +%s)"
  new_webhook_secret="updated-webhook-secret-$(date +%s)"

  config_json=$(
    jq -n \
    --arg name "$name" \
    --arg apiKey "$new_api_key" \
    --arg apiSecret "$new_api_secret" \
    --arg secretKey "$new_secret_key" \
    --arg webhookSecret "$new_webhook_secret" \
    '{
      komainu: {
        name: $name,
        apiKey: $apiKey,
        apiSecret: $apiSecret,
        testingInstance: false,
        secretKey: $secretKey,
        webhookSecret: $webhookSecret
      }
    }'
  )

  local cli_output
  cli_output=$("$LANACLI" --json custodian config-update \
    --custodian-id "$custodian_id" \
    --config-json "$config_json")
  updated_id=$(echo "$cli_output" | jq -r '.custodianId')
  [[ "$updated_id" != "null" ]] || exit 1

}
