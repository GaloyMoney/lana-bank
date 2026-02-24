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

  variables=$(
    jq -n \
    --arg name "$name" \
    --arg apiKey "$api_key" \
    --arg apiSecret "$api_secret" \
    --arg secretKey "$secret_key" \
    --arg webhookSecret "$webhook_secret" \
    '{
      input: {
        komainu: {
          name: $name,
          apiKey: $apiKey,
          apiSecret: $apiSecret,
          testingInstance: true,
          secretKey: $secretKey,
          webhookSecret: $webhookSecret
        }
      }
    }'
  )
  
  exec_admin_graphql 'custodian-create' "$variables"
  custodian_id=$(graphql_output .data.custodianCreate.custodian.custodianId)
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
  
  variables=$(
    jq -n \
    --arg name "$name" \
    --arg custodianId "$custodian_id" \
    --arg apiKey "$new_api_key" \
    --arg apiSecret "$new_api_secret" \
    --arg secretKey "$new_secret_key" \
    --arg webhookSecret "$new_webhook_secret" \
    '{
      input: {
        custodianId: $custodianId,
        config: {
          komainu: {
            name: $name,
            apiKey: $apiKey,
            apiSecret: $apiSecret,
            testingInstance: false,
            secretKey: $secretKey,
            webhookSecret: $webhookSecret
          }
        }
      }
    }'
  )
  
  exec_admin_graphql 'custodian-config-update' "$variables"
  custodian_id=$(graphql_output .data.custodianConfigUpdate.custodian.custodianId)
  [[ "$custodian_id" != "null" ]] || exit 1
  
}
