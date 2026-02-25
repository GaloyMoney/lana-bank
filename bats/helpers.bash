REPO_ROOT=$(git rev-parse --show-toplevel)
COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-${REPO_ROOT##*/}}"

CACHE_DIR=${BATS_TMPDIR:-tmp/bats}/galoy-bats-cache
mkdir -p "$CACHE_DIR"

OATHKEEPER_PROXY="http://localhost:4455"

GQL_APP_ENDPOINT="http://app.localhost:4455/graphql"
GQL_ADMIN_ENDPOINT="http://admin.localhost:4455/graphql"

LANA_HOME="${LANA_HOME:-.lana}"
SERVER_PID_FILE="${LANA_HOME}/server-pid"

LOG_FILE=".e2e-logs"

server_cmd() {
  if [[ -n "${LANA_BIN:-}" ]]; then
    export LANA_CONFIG="${REPO_ROOT}/bats/lana.yml"
    "${LANA_BIN}"
  else
    SQLX_OFFLINE=true make run-server
  fi
}
wait_for_keycloak_user_ready() {
  local email="admin@galoy.io"

  wait4x http http://localhost:8081/realms/master   --timeout 60s --interval 1s
  wait4x http http://localhost:8081/realms/internal --timeout 10s --interval 1s

  for i in {1..60}; do
    access_token=$(get_user_access_token "$email" 2>/dev/null || true)
    [[ -n "$access_token" && "$access_token" != "null" ]] && { echo "✅ User ready"; return 0; }
    sleep 1
  done

  echo "admin user not ready"; exit 1
}

start_server() {
  echo "--- Starting server ---"

  # Check for running server
  if pgrep -f '[l]ana-cli' >/dev/null; then
    rm -f "$SERVER_PID_FILE"
    return 0
  fi

  # Start server if not already running
  local server_started=false
  background server_cmd > "$LOG_FILE" 2>&1
  for i in {1..30}; do
    echo "--- Checking server ${i} ---"
    if grep -q 'Starting' "$LOG_FILE"; then
      server_started=true
      break
    elif grep -q 'Connection reset by peer' "$LOG_FILE"; then
      stop_server
      sleep 1
      background server_cmd > "$LOG_FILE" 2>&1
    else
      sleep 1
      echo "--- Server not running ---"
      cat "$LOG_FILE"
    fi
  done

  if [[ "$server_started" == "false" ]]; then
    echo "--- Server unable to start ---"
    return 1
  fi
}
stop_server() {
  if [[ -f "$SERVER_PID_FILE" ]]; then
    PID=$(cat "$SERVER_PID_FILE")

    if kill -TERM $PID 2>/dev/null; then
      echo "Sent SIGTERM to process $PID"

      local seconds=0
      while (( seconds < 30 )) && kill -0 $PID 2>/dev/null; do
        sleep 1
        ((seconds++))
      done

      if kill -0 $PID 2>/dev/null; then
        echo "Process didn't stop gracefully, sending SIGKILL"
        kill -9 $PID 2>/dev/null || true
      fi
    fi

    rm -f "$SERVER_PID_FILE"
  fi
}

gql_query() {
  cat "$(gql_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_file() {
  echo "${REPO_ROOT}/bats/customer-gql/$1.gql"
}

gql_operation_name() {
  local file=$1
  local operation_line

  operation_line=$(grep -E '^(query|mutation|subscription)' "$file" | head -n 1 || true)

  if [[ -z "$operation_line" ]]; then
    echo ""
    return 0
  fi

  if [[ "$operation_line" =~ ^(query|mutation|subscription)[[:space:]]+([A-Za-z0-9_]+) ]]; then
    echo "${BASH_REMATCH[2]}"
  else
    echo ""
  fi
}

gql_admin_query() {
  cat "$(gql_admin_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_admin_file() {
  echo "${REPO_ROOT}/bats/admin-gql/$1.gql"
}

gql_admin_operation_name() {
  gql_operation_name "$(gql_admin_file $1)"
}

gql_customer_operation_name() {
  gql_operation_name "$(gql_file $1)"
}

gql_dagster_query() {
  cat "$(gql_dagster_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_dagster_file() {
  echo "${REPO_ROOT}/bats/dagster-gql/$1.gql"
}

gql_dagster_operation_name() {
  gql_operation_name "$(gql_dagster_file $1)"
}

graphql_output() {
  echo $output | jq -r "$@"
}

graphql_payload() {
  local query=$1
  local variables_json=$2
  local operation_name=${3:-}

  if [[ -z "$variables_json" ]]; then
    variables_json="{}"
  fi

  if [[ -n "$operation_name" ]]; then
    jq -n --arg query "$query" --argjson variables "$variables_json" --arg operationName "$operation_name" '{query: $query, variables: $variables, operationName: $operationName}'
  else
    jq -n --arg query "$query" --argjson variables "$variables_json" '{query: $query, variables: $variables}'
  fi
}

login_customer() {
  local email=$1
  echo "--- Logging in customer: $email ---"
  
  wait_for_keycloak_user_ready
  local access_token=$(get_customer_access_token "$email") || { echo "Get token failed: $email" >&2; return 1; }
  
  cache_value "$email" $access_token
  echo "--- Customer login successful ---"
}

exec_customer_graphql() {
  local token_name=$1
  local query_name=$2
  local variables=${3:-"{}"}
  local run_cmd="${BATS_TEST_DIRNAME:+run}"
  local operation_name=$(gql_customer_operation_name "$query_name")
  local payload

  payload=$(graphql_payload "$(gql_query $query_name)" "$variables" "$operation_name")

  ${run_cmd} curl -s -X POST \
    -H "Authorization: Bearer $(read_value "$token_name")" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "${GQL_APP_ENDPOINT}"
}

get_user_access_token() {
  local email=$1
  
  local response=$(curl -s -X POST \
      "http://localhost:8081/realms/internal/protocol/openid-connect/token" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "client_id=admin-panel" \
      -d "username=${email}" \
      -d "grant_type=password" \
      -d "scope=openid profile email")
    
  local access_token=$(echo "$response" | jq -r '.access_token')
  
  if [[ "$access_token" == "null" || -z "$access_token" ]]; then
    echo "User token failed for $email: $response" >&2
    return 1
  fi
  echo "$access_token"
}
get_customer_access_token() {
  local email=$1
  
  local response=$(curl -s -X POST \
      "http://localhost:8081/realms/customer/protocol/openid-connect/token" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "client_id=customer-portal" \
      -d "username=${email}" \
      -d "grant_type=password" \
      -d "scope=openid profile email")
    
  local access_token=$(echo "$response" | jq -r '.access_token')
  
  if [[ "$access_token" == "null" || -z "$access_token" ]]; then
    echo "Customer token failed for $email: $response" >&2
    return 1
  fi
  echo "$access_token"
}

login_superadmin() {
  local email="admin@galoy.io"
  wait_for_keycloak_user_ready

  local access_token=$(get_user_access_token "$email") || { echo "Get token failed: $email" >&2; return 1; }

  cache_value "superadmin" $access_token
}

exec_admin_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}
  local run_cmd="${BATS_TEST_DIRNAME:+run}"
  local operation_name=$(gql_admin_operation_name "$query_name")
  local payload

  payload=$(graphql_payload "$(gql_admin_query $query_name)" "$variables" "$operation_name")

  ${run_cmd} curl -s -X POST \
    -H "Authorization: Bearer $(read_value "superadmin")" \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "${GQL_ADMIN_ENDPOINT}"
}

exec_admin_graphql_upload() {
  local query_name=$1
  local variables=$2
  local file_path=$3
  local file_var_name=${4:-"file"}
  local token=$(read_value "superadmin")
  local operation_name=$(gql_admin_operation_name "$query_name")
  local payload

  payload=$(graphql_payload "$(gql_admin_query $query_name)" "$variables" "$operation_name")

  curl -s -X POST \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: multipart/form-data" \
    -F "operations=${payload}" \
    -F "map={\"0\":[\"variables.$file_var_name\"]}" \
    -F "0=@$file_path" \
    "${GQL_ADMIN_ENDPOINT}"
}

exec_dagster_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}
  local run_cmd="${BATS_TEST_DIRNAME:+run}"
  local operation_name=$(gql_dagster_operation_name "$query_name")
  local payload

  payload=$(graphql_payload "$(gql_dagster_query $query_name)" "$variables" "$operation_name")

  ${run_cmd} curl -s -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "${DAGSTER_URL:-http://localhost:3000/graphql}"
}

exec_dagster_graphql_status() {
  local query_name=$1
  local variables=${2:-"{}"}
  local run_cmd="${BATS_TEST_DIRNAME:+run}"
  local operation_name=$(gql_dagster_operation_name "$query_name")
  local payload

  payload=$(graphql_payload "$(gql_dagster_query $query_name)" "$variables" "$operation_name")

  ${run_cmd} curl -s -o /dev/null -w "%{http_code}" -X POST \
    -H "Content-Type: application/json" \
    -d "$payload" \
    "${DAGSTER_URL:-http://localhost:3000/graphql}"
}

dagster_validate_json() {
  if ! echo "$output" | jq . >/dev/null 2>&1; then
    echo "Dagster GraphQL did not return valid JSON: $output"
    return 1
  fi
}

# Check if the launch_run GraphQL response (in global $output) contains errors
# Must be called immediately after exec_dagster_graphql "launch_run" to check the response
dagster_check_launch_run_errors() {
  dagster_validate_json || return 1

  local error_type=$(echo "$output" | jq -r '.data.launchRun.__typename // empty')
  if [ "$error_type" = "PythonError" ] || [ "$error_type" = "RunConfigValidationInvalid" ] || [ "$error_type" = "InvalidSubsetError" ]; then
    local error_msg=$(echo "$output" | jq -r '.data.launchRun.message // .data.launchRun.errors[0].message // "Unknown error"')
    echo "Failed to launch run: $error_type - $error_msg"
    echo "Full response: $output"
    return 1
  fi
}

dagster_poll_run_status() {
  local run_id=$1
  local attempts=${2:-90}
  local sleep_between=${3:-2}
  local run_status=""

  while [ $attempts -gt 0 ]; do
    local poll_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
    exec_dagster_graphql "run_status" "$poll_vars"
    
    dagster_validate_json || return 1
    
    run_status=$(echo "$output" | jq -r '.data.runOrError.status // empty')
    
    if [ "$run_status" = "SUCCESS" ]; then
      return 0
    fi
    if [ "$run_status" = "FAILURE" ] || [ "$run_status" = "CANCELED" ]; then
      echo "Run $run_status. Fetching event logs..."
      local event_vars=$(jq -n --arg runId "$run_id" '{ runId: $runId }')
      exec_dagster_graphql "run_events" "$event_vars"
      if echo "$output" | jq . >/dev/null 2>&1; then
        echo "=== Error/failure events ==="
        echo "$output" | jq -r '.data.logsForRun.events[]? | select(.message != null) | select(.eventType == "STEP_FAILURE" or (.message | test("error|Error|ERROR|FAILURE|fail"))) | "\(.eventType): \(.message)"' 2>/dev/null || true
        echo "=== Last 20 events ==="
        echo "$output" | jq -r '[.data.logsForRun.events[]? | select(.message != null)] | .[-20:][] | "\(.eventType): \(.message)"' 2>/dev/null || true
      fi
      return 1
    fi
    
    attempts=$((attempts-1))
    sleep $sleep_between
  done
  
  echo "Run last status: $run_status"
  return 1
}

# Run the given command in the background. Useful for starting a
# node and then moving on with commands that exercise it for the
# test.
#
# Ensures that BATS' handling of file handles is taken into account;
# see
# https://github.com/bats-core/bats-core#printing-to-the-terminal
# https://github.com/sstephenson/bats/issues/80#issuecomment-174101686
# for details.
background() {
  "$@" 3>- &
  echo $!
}

# Taken from https://github.com/docker/swarm/blob/master/test/integration/helpers.bash
# Retry a command $1 times until it succeeds. Wait $2 seconds between retries.
retry() {
  local attempts=$1
  shift
  local delay=$1
  shift
  local i

  for ((i = 0; i < attempts; i++)); do
    run "$@"
    if [[ "$status" -eq 0 ]]; then
      return 0
    fi
    sleep "$delay"
  done

  echo "Command \"$*\" failed $attempts times. Output: $output"
  false
}

random_uuid() {
  if [[ -e /proc/sys/kernel/random/uuid ]]; then
    cat /proc/sys/kernel/random/uuid
  else
    uuidgen
  fi
}

cache_value() {
  echo $2 >${CACHE_DIR}/$1
}

read_value() {
  cat ${CACHE_DIR}/$1
}

cat_logs() {
  cat "$LOG_FILE"
}

reset_log_files() {
  for file in "$@"; do
    rm "$file" &> /dev/null || true && touch "$file"
  done
}

generate_email() {
  echo "user$(date +%s%N)@example.com" | tr '[:upper:]' '[:lower:]'
}

create_customer() {
  customer_email=$(generate_email)
  telegramHandle=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
      --arg email "$customer_email" \
      --arg telegramHandle "$telegramHandle" \
      --arg customerType "$customer_type" \
      '{
      input: {
        email: $email,
        telegramHandle: $telegramHandle,
        customerType: $customerType
      }
    }'
  )

  exec_admin_graphql 'prospect-create' "$variables"
  prospect_id=$(graphql_output .data.prospectCreate.prospect.prospectId)
  [[ "$prospect_id" != "null" ]] || exit 1

  # Simulate KYC start via SumSub applicantCreated webhook
  local webhook_id="req-$(date +%s%N)"
  local applicant_id="test-applicant-$webhook_id"
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantCreated",
      "reviewStatus": "init",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  # Simulate KYC approval via SumSub webhook
  curl -s -X POST http://localhost:5253/webhook/sumsub \
    -H "Content-Type: application/json" \
    -d '{
      "applicantId": "'"$applicant_id"'",
      "inspectionId": "test-inspection-'"$webhook_id"'",
      "correlationId": "'"$webhook_id"'",
      "externalUserId": "'"$prospect_id"'",
      "levelName": "basic-kyc-level",
      "type": "applicantReviewed",
      "reviewResult": { "reviewAnswer": "GREEN" },
      "reviewStatus": "completed",
      "createdAtMs": "'"$(date -u +%Y-%m-%dT%H:%M:%S.000Z)"'",
      "sandboxMode": true
    }' > /dev/null

  # Customer is created asynchronously via webhook inbox processing.
  # Poll until the customer exists.
  customer_id="$prospect_id"
  for i in {1..30}; do
    variables=$(jq -n --arg id "$customer_id" '{ id: $id }')
    exec_admin_graphql 'customer' "$variables"
    fetched_id=$(graphql_output .data.customer.customerId)
    [[ "$fetched_id" != "null" ]] && break
    sleep 1
  done
  [[ "$fetched_id" != "null" ]] || exit 1

  echo $prospect_id
}

create_deposit_account_for_customer() {
  customer_id=$1

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      input: {
        customerId: $customerId
      }
    }'
  )

  exec_admin_graphql 'deposit-account-create' "$variables"
  deposit_account_id=$(graphql_output '.data.depositAccountCreate.account.depositAccountId')
  [[ "$deposit_account_id" != "null" ]] || exit 1
  echo "$deposit_account_id"
}

assert_balance_sheet_balanced() {
  variables=$(
    jq -n \
      --arg from "$(from_utc)" \
      '{ from: $from }'
  )
  exec_admin_graphql 'balance-sheet' "$variables"
  echo $(graphql_output)

  balance_usd=$(graphql_output '.data.balanceSheet.balance.usd.balancesByLayer.settled.netDebit')
  balance=${balance_usd}
  echo "Balance Sheet USD Balance (should be 0): $balance"
  [[ "$balance" == "0" ]] || exit 1

  debit_usd=$(graphql_output '.data.balanceSheet.balance.usd.balancesByLayer.settled.debit')
  debit=${debit_usd}
  echo "Balance Sheet USD Debit (should be >0): $debit"
  [[ "$debit" -gt "0" ]] || exit 1

  credit_usd=$(graphql_output '.data.balanceSheet.balance.usd.balancesByLayer.settled.credit')
  credit=${credit_usd}
  echo "Balance Sheet USD Credit (should be == debit): $credit"
  [[ "$credit" == "$debit" ]] || exit 1
}

assert_trial_balance() {
  variables=$(
    jq -n \
      --arg from "$(from_utc)" \
      '{ from: $from }'
  )
  exec_admin_graphql 'trial-balance' "$variables"
  echo $(graphql_output)

  all_btc=$(graphql_output '.data.trialBalance.total.btc.balancesByLayer.all.netDebit')
  echo "Trial Balance BTC (should be zero): $all_btc"
  [[ "$all_btc" == "0" ]] || exit 1

  all_usd=$(graphql_output '.data.trialBalance.total.usd.balancesByLayer.all.netDebit')
  echo "Trial Balance USD (should be zero): $all_usd"
  [[ "$all_usd" == "0" ]] || exit 1
}

assert_accounts_balanced() {
  assert_balance_sheet_balanced
  assert_trial_balance
}

net_usd_revenue() {
  variables=$(
    jq -n \
      --arg from "$(from_utc)" \
      '{ from: $from }'
  )
  exec_admin_graphql 'profit-and-loss' "$variables"

  revenue_usd=$(graphql_output '.data.profitAndLossStatement.net.usd.balancesByLayer.all.netCredit')
  echo $revenue_usd
}

from_utc() {
  date -u -d @0 +"%Y-%m-%dT%H:%M:%S.%3NZ"
}

naive_now() {
  date +"%Y-%m-%d"
}
