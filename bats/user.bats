#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

@test "user: unauthorized" {
  "skip"

  cache_value "alice" "invalid-token"
  exec_graphql 'alice' 'me'
  error_code=$(graphql_output '.error.code')
  [[ "$error_code" == 401 ]] || exit 1

  error_status=$(graphql_output '.error.status')
  [[ "$error_status" == "Unauthorized" ]] || exit 1
}

@test "user: can create a user" {
  token=$(create_user)
  cache_value "alice" "$token"

  exec_graphql 'alice' 'me'
  echo "$output"

  user_id=$(graphql_output '.data.me.userId')
  [[ "$user_id" != "null" ]] || exit 1

  btc_address=$(graphql_output '.data.me.btcDepositAddress')
  cache_value 'user.btc' "$btc_address"

  ust_address=$(graphql_output '.data.me.ustDepositAddress')
  cache_value 'user.ust' "$ust_address"
}

@test "user: can deposit" {
  "skip"

  ust_address=$(read_value 'user.ust')
  btc_address=$(read_value 'user.btc')

  variables=$(
    jq -n \
      --arg address "$ust_address" \
    '{
       address: $address,
       amount: "10000",
       currency: "UST"
    }'
  )
  exec_cala_graphql 'simulate-deposit' "$variables"

  exec_graphql 'alice' 'me'
  usd_balance=$(graphql_output '.data.me.balance.checking.settled.usdBalance')
  [[ "$usd_balance" == 1000000 ]] || exit 1

  assert_assets_liabilities_equity

  btc_address=$(read_value 'user.btc')

  variables=$(
    jq -n \
      --arg address "$btc_address" \
    '{
       address: $address,
       amount: "1",
       currency: "BTC"
    }'
  )
  exec_cala_graphql 'simulate-deposit' "$variables"

  exec_graphql 'alice' 'me'
  btc_balance=$(graphql_output '.data.me.balance.unallocatedCollateral.settled.btcBalance')
  [[ "$btc_balance" == 100000000 ]] || exit 1

  assert_assets_liabilities_equity
}

@test "user: can withdraw" {
  "skip"

  variables=$(
    jq -n \
    --arg date "$(date +%s%N)" \
    '{
      input: {
        amount: 150000,
        destination: "tron-address",
        reference: ("withdrawal-ref-" + $date)
      }
    }'
  )
  exec_graphql 'alice' 'initiate-withdrawal' "$variables"

  withdrawal_id=$(graphql_output '.data.withdrawalInitiate.withdrawal.withdrawalId')
  [[ "$withdrawal_id" != "null" ]] || exit 1

  exec_graphql 'alice' 'me'
  checking_balance=$(graphql_output '.data.me.balance.checking.settled.usdBalance')
  [[ "$checking_balance" == 850000 ]] || exit 1

  assert_assets_liabilities_equity
}

@test "user: verify level 2" {
# TODO: mock this call
  exec_graphql 'alice' 'sumsub-token-create'
  token=$(echo "$output" | jq -r '.data.sumsubTokenCreate.token')
  [[ "$token" != "null" ]] || exit 1

# TODO: mock this call
  exec_graphql 'alice' 'sumsub-permalink-create'
  echo "$output"

  exec_graphql 'alice' 'me'
  echo "$output"
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ZERO" ]] || exit 1

  user_id=$(graphql_output '.data.me.userId')
  [[ "$user_id" != "null" ]] || exit 1

  # # accepted
  curl -v -X POST http://localhost:5253/sumsub/callback \
      -H "Content-Type: application/json" \
      -d '{
          "applicantId": "5cb56e8e0a975a35f333cb83",
          "inspectionId": "5cb56e8e0a975a35f333cb84",
          "correlationId": "req-a260b669-4f14-4bb5-a4c5-ac0218acb9a4",
          "externalUserId": "'"$user_id"'",
          "levelName": "basic-kyc-level",
          "type": "applicantReviewed",
          "reviewResult": {
            "reviewAnswer": "GREEN"
          },
          "reviewStatus": "completed",
          "createdAtMs": "2020-02-21 13:23:19.321"
      }'

  exec_graphql 'alice' 'me'
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ONE" ]] || exit 1

  # declined
  curl -X POST http://localhost:5253/sumsub/callback \
    -H "Content-Type: application/json" \
    -d '{
        "applicantId": "5cb744200a975a67ed1798a4",
        "inspectionId": "5cb744200a975a67ed1798a5",
        "correlationId": "req-fa94263f-0b23-42d7-9393-ab10b28ef42d",
        "externalUserId": "'"$user_id"'",
        "levelName": "basic-kyc-level",
        "type": "applicantReviewed",
        "reviewResult": {
            "moderationComment": "We could not verify your profile. If you have any questions, please contact the Company where you try to verify your profile ${clientSupportEmail}",
            "clientComment": "Suspected fraudulent account.",
            "reviewAnswer": "RED",
            "rejectLabels": ["UNSATISFACTORY_PHOTOS", "GRAPHIC_EDITOR", "FORGERY"],
            "reviewRejectType": "FINAL"
        },
        "reviewStatus": "completed",
        "createdAtMs": "2020-02-21 13:23:19.001"
    }'

  exec_graphql 'alice' 'me'
  level=$(graphql_output '.data.me.level')
  [[ "$level" == "ONE" ]] || exit 1

}
