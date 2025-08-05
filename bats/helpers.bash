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
  nix run .
}

wait_for_kratos_user_ready() {
  local email="admin@galoy.io"
  echo "--- Waiting for Kratos user to be created ---"
  
  # First, check if the UserOnboarding job has processed the UserCreated event
  # Look for specific log patterns indicating Kratos user creation
  echo "--- Checking logs for user onboarding completion ---"
  retry 30 1 grep -q "kratos_admin.*create_user\|user.*onboarding.*completed\|authentication.*id.*updated\|UserCreated.*processed" "$LOG_FILE" || true
  
  # Alternative method: Try to verify the admin user exists in Kratos
  echo "--- Verifying Kratos user exists ---"
  for i in {1..15}; do
    echo "--- Checking if Kratos user exists (attempt ${i}) ---"
    
    # Get a login flow
    flowId=$(curl -s -X GET -H "Accept: application/json" "http://admin.localhost:4455/self-service/login/api" 2>/dev/null | jq -r '.id' 2>/dev/null || echo "")
    
    if [[ -n "$flowId" && "$flowId" != "null" ]]; then
      # Try to initiate login for the admin user
      variables=$(jq -n --arg email "$email" '{ identifier: $email, method: "code" }' 2>/dev/null || echo "")
      response=$(curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" \
                 -d "$variables" "http://admin.localhost:4455/self-service/login?flow=$flowId" 2>/dev/null || echo "")
      
      # If we don't get "account does not exist" error, the user is ready
      if [[ -n "$response" ]] && ! echo "$response" | grep -q "This account does not exist"; then
        echo "--- Kratos user ready ---"
        return 0
      fi
    fi
    
    echo "--- Kratos user not ready yet, waiting... ---"
    sleep 1
  done
  
  echo "--- Kratos user may not be ready, but proceeding anyway ---"
  echo "--- Note: Login may require retries ---"
}

wait_for_keycloak_user_ready() {
  local email="admin@galoy.io"

  echo "--- Waiting for Keycloak service to be ready ---"
  for i in {1..10}; do
    echo "--- Checking if Keycloak is responding (attempt ${i}) ---"
    
    # Check if Keycloak realms are accessible (better indicator than health endpoints)
    if curl -s -f "http://localhost:8081/realms/master" >/dev/null 2>&1 && \
       curl -s -f "http://localhost:8081/realms/lana-admin" >/dev/null 2>&1; then
      echo "--- Keycloak service is ready ---"
      break
    fi
    
    if [[ $i -eq 10 ]]; then
      echo "--- Keycloak service not ready after 10 attempts, proceeding anyway ---"
    else
      echo "--- Keycloak service not ready yet, waiting... ---"
      sleep 2
    fi
  done
  
  # Verify the admin user exists in Keycloak
  echo "--- Verifying Keycloak admin user exists ---"
  for i in {1..20}; do
    echo "--- Checking if Keycloak admin user exists (attempt ${i}) ---"
    admin_token=$(get_keycloak_admin_token 2>/dev/null || echo "")
    if [[ -n "$admin_token" && "$admin_token" != "null" ]]; then
      user_id=$(find_user_by_email "$admin_token" "$email" 2>/dev/null || echo "")
      if [[ -n "$user_id" && "$user_id" != "null" ]]; then
        echo "--- Keycloak admin user found with ID: ${user_id} ---"
        access_token=$(get_user_access_token "$admin_token" "$user_id" "$email" 2>/dev/null || echo "")
        if [[ -n "$access_token" && "$access_token" != "null" ]]; then
          echo "--- Keycloak user is ready and can authenticate ---"
          return 0
        else
          echo "--- Keycloak user found but cannot authenticate yet ---"
        fi
      else
        echo "--- Keycloak admin user not found, may still be creating ---"
      fi
    else
      echo "--- Cannot get Keycloak admin token yet ---"
    fi
    
    echo "--- Keycloak user not ready yet, waiting... ---"
    sleep 2
  done
  
  echo "--- Keycloak user may not be ready, but proceeding anyway ---"
  echo "--- Note: Login may require retries ---"
}

start_server() {
  echo "--- Starting server ---"

  # Check for running server
  if pgrep -f '[l]ana-cli' >/dev/null; then
    rm -f "$SERVER_PID_FILE"
    return 0
  fi

  # Start server if not already running
  background server_cmd > "$LOG_FILE" 2>&1
  for i in {1..20}; do
    echo "--- Checking if server is running ${i} ---"
    if grep -q 'Starting' "$LOG_FILE"; then
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
}
stop_server() {
  if [[ -f "$SERVER_PID_FILE" ]]; then
    kill -9 $(cat "$SERVER_PID_FILE") || true
  fi
}

gql_query() {
  cat "$(gql_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_file() {
  echo "${REPO_ROOT}/bats/customer-gql/$1.gql"
}

gql_admin_query() {
  cat "$(gql_admin_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_admin_file() {
  echo "${REPO_ROOT}/bats/admin-gql/$1.gql"
}

graphql_output() {
  echo $output | jq -r "$@"
}

login_customer() {
  local email=$1

  flowId=$(curl -s -X GET -H "Accept: application/json" "http://app.localhost:4455/self-service/login/api" | jq -r '.id')
  variables=$(jq -n --arg email "$email" '{ identifier: $email, method: "code" }' )
  curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "http://app.localhost:4455/self-service/login?flow=$flowId"

  code=$(getEmailCode $email)
  variables=$(jq -n --arg email "$email" --arg code "$code" '{ identifier: $email, method: "code", code: $code }' )
  session=$(curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "http://app.localhost:4455/self-service/login?flow=$flowId")
  token=$(echo $session | jq -r '.session_token')
  cache_value "$email" $token
}

exec_customer_graphql() {
  local token_name=$1
  local query_name=$2
  local variables=${3:-"{}"}

  AUTH_HEADER="Authorization: Bearer $(read_value "$token_name")"

  if [[ "${BATS_TEST_DIRNAME}" != "" ]]; then
    run_cmd="run"
  else
    run_cmd=""
  fi

  ${run_cmd} curl -s \
    -X POST \
    ${AUTH_HEADER:+ -H "$AUTH_HEADER"} \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$(gql_query $query_name)\", \"variables\": $variables}" \
    "${GQL_APP_ENDPOINT}"
}


get_keycloak_admin_token() {
  local response=$(curl -s -X POST \
    "http://localhost:8081/realms/master/protocol/openid-connect/token" \
    -H "Content-Type: application/x-www-form-urlencoded" \
    -d "client_id=admin-cli" \
    -d "username=admin" \
    -d "password=admin" \
    -d "grant_type=password")
  
  local token=$(echo "$response" | jq -r '.access_token')
  if [[ "$token" == "null" || -z "$token" ]]; then
    echo "Failed to get Keycloak admin token: $response" >&2
    return 1
  fi
  echo "$token"
}

find_user_by_email() {
  local admin_token=$1
  local email=$2
  
  local response=$(curl -s -X GET \
    "http://localhost:8081/admin/realms/lana-admin/users?email=${email}&exact=true" \
    -H "Authorization: Bearer ${admin_token}" \
    -H "Content-Type: application/json")
  
  local user_id=$(echo "$response" | jq -r '.[0].id')
  if [[ "$user_id" == "null" || -z "$user_id" ]]; then
    echo "User not found: $email" >&2
    return 1
  fi
  echo "$user_id"
}


set_user_password() {
  local admin_token=$1
  local user_id=$2
  local password="admin"
  
  curl -s -X PUT \
    "http://localhost:8081/admin/realms/lana-admin/users/${user_id}/reset-password" \
    -H "Authorization: Bearer ${admin_token}" \
    -H "Content-Type: application/json" \
      -d "{\"type\":\"password\",\"value\":\"${password}\",\"temporary\":false}" >/dev/null
  
  echo "$password"
}

get_user_access_token() {
  local admin_token=$1
  local user_id=$2
  local email=$3
  
  local password=$(set_user_password "$admin_token" "$user_id")
  local response=$(curl -s -X POST \
      "http://localhost:8081/realms/lana-admin/protocol/openid-connect/token" \
      -H "Content-Type: application/x-www-form-urlencoded" \
      -d "client_id=lana-admin-panel" \
      -d "username=${email}" \
      -d "password=${password}" \
      -d "grant_type=password" \
      -d "scope=openid profile email")
    
  local access_token=$(echo "$response" | jq -r '.access_token')
  
  if [[ "$access_token" == "null" || -z "$access_token" ]]; then
    echo "Failed to get user access token for $email: $response" >&2
    return 1
  fi
  echo "$access_token"
}

login_superadmin() {
  local email="admin@galoy.io"
  wait_for_keycloak_user_ready
  local cached_token=$(read_value "superadmin" 2>/dev/null || echo "")
  if [[ -n "$cached_token" && "$cached_token" != "" ]]; then
    return 0
  fi
  
  local admin_token=$(get_keycloak_admin_token)
  if [[ $? -ne 0 ]]; then
    echo "Failed to get Keycloak admin token" >&2
    return 1
  fi
    
  local user_id=$(find_user_by_email "$admin_token" "$email")
  if [[ $? -ne 0 ]]; then
    echo "Failed to find user: $email" >&2
    return 1
  fi

  local access_token=$(get_user_access_token "$admin_token" "$user_id" "$email")
  if [[ $? -ne 0 ]]; then
    echo "Failed to get access token for: $email" >&2
    return 1
  fi

  cache_value "superadmin" $access_token
}

exec_admin_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}
  local token=$(read_value "superadmin")

  if [[ "${BATS_TEST_DIRNAME}" != "" ]]; then
    run_cmd="run"
  else
    run_cmd=""
  fi

  ${run_cmd} curl -s \
    -X POST \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$(gql_admin_query $query_name)\", \"variables\": $variables}" \
    "${GQL_ADMIN_ENDPOINT}"
}

exec_admin_graphql_upload() {
  local query_name=$1
  local variables=$2
  local file_path=$3
  local file_var_name=${4:-"file"}
  local token=$(read_value "superadmin")

  curl -s -X POST \
    -H "Authorization: Bearer ${token}" \
    -H "Content-Type: multipart/form-data" \
    -F "operations={\"query\": \"$(gql_admin_query $query_name)\", \"variables\": $variables}" \
    -F "map={\"0\":[\"variables.$file_var_name\"]}" \
    -F "0=@$file_path" \
    "${GQL_ADMIN_ENDPOINT}"
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

getEmailCode() {
  local email="$1"

  local container_name="${COMPOSE_PROJECT_NAME}-kratos-admin-pg-1"
  local query="SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"
  
  for i in {1..10}; do
    echo "--- Checking for email code for ${email} (attempt ${i}) ---" >&2
    
    local result=""
    local code=""
    
    # Try podman exec first (for containerized environments like GitHub Actions)
    if command -v podman >/dev/null 2>&1; then
      result=$(podman exec "${container_name}" psql -U dbuser -d default -t -c "${query}" 2>/dev/null || echo "")
    fi
    
    # If we got a result from container exec, extract the code
    if [[ -n "$result" ]]; then
      code=$(echo "$result" | grep -Eo '[0-9]{6}' | head -n1)
      if [[ -n "$code" ]]; then
        echo "--- Email code found: ${code} ---" >&2
        echo "$code"
        return 0
      fi
    fi
    
    # Fallback to direct connection (for development environments)
    local KRATOS_PG_CON="postgres://dbuser:secret@localhost:5434/default?sslmode=disable"
    result=$(psql $KRATOS_PG_CON -t -c "${query}" 2>/dev/null || echo "")

    if [[ -n "$result" ]]; then
      code=$(echo "$result" | grep -Eo '[0-9]{6}' | head -n1)
      if [[ -n "$code" ]]; then
        echo "--- Email code found: ${code} ---" >&2
        echo "$code"
        return 0
      fi
    fi
    
    echo "--- No email code found yet, waiting... ---" >&2
    sleep 1
  done

  echo "No message for email ${email} after 10 attempts" >&2
  exit 1
}

generate_email() {
  echo "user$(date +%s%N)@example.com" | tr '[:upper:]' '[:lower:]'
}

create_customer() {
  customer_email=$(generate_email)
  telegramId=$(generate_email)
  customer_type="INDIVIDUAL"

  variables=$(
    jq -n \
      --arg email "$customer_email" \
      --arg telegramId "$telegramId" \
      --arg customerType "$customer_type" \
      '{
      input: {
        email: $email,
        telegramId: $telegramId,
        customerType: $customerType
      }
    }'
  )

  exec_admin_graphql 'customer-create' "$variables"
  customer_id=$(graphql_output .data.customerCreate.customer.customerId)
  [[ "$customer_id" != "null" ]] || exit 1
  echo $customer_id
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

wait_for_checking_account() {
  customer_id=$1

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{ id: $customerId }'
  )
  exec_admin_graphql 'customer' "$variables"

  echo "checking | $i. $(graphql_output)" >> $RUN_LOG_FILE
  deposit_account_id=$(graphql_output '.data.customer.depositAccount.depositAccountId')
  [[ "$deposit_account_id" != "null" ]] || exit 1

}
