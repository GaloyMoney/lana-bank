REPO_ROOT=$(git rev-parse --show-toplevel)
COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-${REPO_ROOT##*/}}"

CACHE_DIR=${BATS_TMPDIR:-tmp/bats}/galoy-bats-cache
mkdir -p "$CACHE_DIR"

OATHKEEPER_PROXY="http://localhost:4455"
MAILHOG_ENDPOINT="http://localhost:8025"

GQL_APP_ENDPOINT="${OATHKEEPER_PROXY}/app/graphql"
GQL_ADMIN_ENDPOINT="${OATHKEEPER_PROXY}/admin/graphql"

LANA_HOME="${LANA_HOME:-.lana}"
export LANA_CONFIG="${REPO_ROOT}/bats/lana-sim-time.yml"
SERVER_PID_FILE="${LANA_HOME}/server-pid"

LOG_FILE=".e2e-logs"

reset_pg() {
  docker exec "${COMPOSE_PROJECT_NAME}-core-pg-1" psql $PG_CON -c "DROP SCHEMA public CASCADE"
  docker exec "${COMPOSE_PROJECT_NAME}-core-pg-1" psql $PG_CON -c "CREATE SCHEMA public"
}

server_cmd() {
  server_location="${REPO_ROOT}/target/debug/lana-cli"
  if [[ ! -z ${CARGO_TARGET_DIR} ]]; then
    server_location="${CARGO_TARGET_DIR}/debug/lana-cli"
  fi

  bash -c ${server_location} $@
}

server_cmd_nix() {
  server_location="$(nix build . --print-out-paths)/bin/lana-cli"

  bash -c ${server_location} $@
}

start_server() {
    echo "--- Starting server make ---"

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

start_server_nix() {
  echo "--- Starting server nix ---"

  # Check for running server
  if pgrep -f '[l]ana-cli' >/dev/null; then
    rm -f "$SERVER_PID_FILE"
    return 0
  fi

  echo "LANA_CONFIG: $LANA_CONFIG"

  # Start server if not already running
  background server_cmd_nix > "$LOG_FILE" 2>&1
  echo "--- Server started ---"
  for i in {1..20}; do
    echo "--- Checking if server is running ${i} ---"
    if grep -q 'Starting' "$LOG_FILE"; then
      break
    elif grep -q 'Connection reset by peer' "$LOG_FILE"; then
      stop_server
      sleep 1
      background server_cmd_nix > "$LOG_FILE" 2>&1
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

  flowId=$(curl -s -X GET -H "Accept: application/json" "${OATHKEEPER_PROXY}/app/self-service/login/api" | jq -r '.id')
  variables=$(jq -n --arg email "$email" '{ identifier: $email, method: "code" }' )
  curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/app/self-service/login?flow=$flowId"
  sleep 1

  code=$(getEmailCode $email)
  variables=$(jq -n --arg email "$email" --arg code "$code" '{ identifier: $email, method: "code", code: $code }' )
  session=$(curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/app/self-service/login?flow=$flowId")
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

login_superadmin() {
  local email="admin@galoy.io"

  flowId=$(curl -s -X GET -H "Accept: application/json" "${OATHKEEPER_PROXY}/admin/self-service/login/api" | jq -r '.id')
  variables=$(jq -n --arg email "$email" '{ identifier: $email, method: "code" }' )
  curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/admin/self-service/login?flow=$flowId"
  sleep 1

  code=$(getEmailCode $email)
  variables=$(jq -n --arg email "$email" --arg code "$code" '{ identifier: $email, method: "code", code: $code }' )
  session=$(curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/admin/self-service/login?flow=$flowId")
  token=$(echo $session | jq -r '.session_token')
  cache_value "superadmin" $token
}

exec_admin_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}

  AUTH_HEADER="Authorization: Bearer $(read_value "superadmin")"

  if [[ "${BATS_TEST_DIRNAME}" != "" ]]; then
    run_cmd="run"
  else
    run_cmd=""
  fi

  ${run_cmd} curl -s \
    -X POST \
    ${AUTH_HEADER:+ -H "$AUTH_HEADER"} \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$(gql_admin_query $query_name)\", \"variables\": $variables}" \
    "${GQL_ADMIN_ENDPOINT}"
}

exec_admin_graphql_upload() {
  local query_name=$1
  local variables=$2
  local file_path=$3
  local file_var_name=${4:-"file"}

  AUTH_HEADER="Authorization: Bearer $(read_value "superadmin")"

  curl -s -X POST \
    ${AUTH_HEADER:+ -H "$AUTH_HEADER"} \
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

  local emails=$(curl -s -X GET "${MAILHOG_ENDPOINT}/api/v2/search?kind=to&query=${email}")
  if [[ $(echo "$emails" | jq '.total') -eq 0 ]]; then
    echo "No message for email ${email}"
    exit 1
  fi

  local email_content=$(echo "$emails" | jq '.items[0].MIME.Parts[0].Body' | tr -d '"')
  local code=$(echo "$email_content" | grep -Eo '[0-9]{6}' | head -n1)

  echo "$code"
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
