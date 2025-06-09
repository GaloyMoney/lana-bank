#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server_nix
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "custody: can create custodian" {
  name="test-komainu-$(date +%s)"
  api_key="test-api-key-$(date +%s)"
  api_secret="test-api-secret-$(date +%s)"
  secret_key="test-secret-key-$(date +%s)"
  

  variables=$(
    jq -n \
    --arg name "$name" \
    --arg apiKey "$api_key" \
    --arg apiSecret "$api_secret" \
    --arg secretKey "$secret_key" \
    '{
      input: {
        komainu: {
          name: $name,
          apiKey: $apiKey,
          apiSecret: $apiSecret,
          testingInstance: true,
          secretKey: $secretKey
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
  
  variables=$(
    jq -n \
    --arg name "$name" \
    --arg custodianId "$custodian_id" \
    --arg apiKey "$new_api_key" \
    --arg apiSecret "$new_api_secret" \
    --arg secretKey "$new_secret_key" \
    '{
      input: {
        custodianId: $custodianId,
        config: {
          komainu: {
            name: $name,
            apiKey: $apiKey,
            apiSecret: $apiSecret,
            testingInstance: false,
            secretKey: $secretKey
          }
        }
      }
    }'
  )
  
  exec_admin_graphql 'custodian-config-update' "$variables"
  custodian_config=$(graphql_output .data.custodianConfigUpdate.custodian.custodianConfig)
  [[ "$custodian_config" != "null" ]] || exit 1
  
}
