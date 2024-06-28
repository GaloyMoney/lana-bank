REPO_ROOT=$(git rev-parse --show-toplevel)
COMPOSE_PROJECT_NAME="${COMPOSE_PROJECT_NAME:-${REPO_ROOT##*/}}"

CACHE_DIR=${BATS_TMPDIR:-tmp/bats}/galoy-bats-cache
mkdir -p "$CACHE_DIR"

KRATOS_PUBLIC_ENDPOINT="http://localhost:4455"
GQL_PUBLIC_ENDPOINT="http://localhost:4455/graphql"
GQL_ADMIN_ENDPOINT="http://localhost:5253/graphql"
GQL_CALA_ENDPOINT="http://localhost:2252/graphql"

LAVA_HOME="${LAVA_HOME:-.lava}"
export LAVA_CONFIG="${REPO_ROOT}/bats/lava.yml"
SERVER_PID_FILE="${LAVA_HOME}/server-pid"

reset_pg() {
  docker exec "${COMPOSE_PROJECT_NAME}-core-pg-1" psql $PG_CON -c "DROP SCHEMA public CASCADE"
  docker exec "${COMPOSE_PROJECT_NAME}-core-pg-1" psql $PG_CON -c "CREATE SCHEMA public"
  docker exec "${COMPOSE_PROJECT_NAME}-cala-pg-1" psql $PG_CON -c "DROP SCHEMA public CASCADE"
  docker exec "${COMPOSE_PROJECT_NAME}-cala-pg-1" psql $PG_CON -c "CREATE SCHEMA public"
}

server_cmd() {
  server_location="${REPO_ROOT}/target/debug/lava-core"
  if [[ ! -z ${CARGO_TARGET_DIR} ]] ; then
    server_location="${CARGO_TARGET_DIR}/debug/lava-core"
  fi

  bash -c ${server_location} $@
}

start_server() {
  # Check for running server
  if [ -n "$BASH_VERSION" ]; then
    server_process_and_status=$(ps a | grep 'target/debug/lava-core' | grep -v grep; echo ${PIPESTATUS[2]})
  elif [ -n "$ZSH_VERSION" ]; then
    server_process_and_status=$(ps a | grep 'target/debug/lava-core' | grep -v grep; echo ${pipestatus[3]})
  else
    echo "Unsupported shell."
    exit 1
  fi
  exit_status=$(echo "$server_process_and_status" | tail -n 1)
  if [ "$exit_status" -eq 0 ]; then
    rm -f "$SERVER_PID_FILE"
    return 0
  fi

  # Start server if not already running
  background server_cmd > .e2e-logs 2>&1
  for i in {1..20}
  do
    if head .e2e-logs | grep -q 'Starting graphql server on port'; then
      break
    elif head .e2e-logs | grep -q 'Connection reset by peer'; then
      stop_server
      sleep 1
      background server_cmd > .e2e-logs 2>&1
    else
      sleep 1
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
  echo "${REPO_ROOT}/bats/gql/$1.gql"
}


gql_admin_query() {
  cat "$(gql_admin_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_admin_file() {
  echo "${REPO_ROOT}/bats/admin-gql/$1.gql"
}

gql_cala_query() {
  cat "$(gql_cala_file $1)" | tr '\n' ' ' | sed 's/"/\\"/g'
}

gql_cala_file() {
  echo "${REPO_ROOT}/bats/cala-gql/$1.gql"
}

graphql_output() {
  echo $output | jq -r "$@"
}

exec_graphql() {
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
    "${GQL_PUBLIC_ENDPOINT}"
}

exec_admin_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}

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

exec_cala_graphql() {
  local query_name=$1
  local variables=${2:-"{}"}

  if [[ "${BATS_TEST_DIRNAME}" != "" ]]; then
    run_cmd="run"
  else
    run_cmd=""
  fi

  ${run_cmd} curl -s \
    -X POST \
    ${AUTH_HEADER:+ -H "$AUTH_HEADER"} \
    -H "Content-Type: application/json" \
    -d "{\"query\": \"$(gql_cala_query $query_name)\", \"variables\": $variables}" \
    "${GQL_CALA_ENDPOINT}"
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

  for ((i=0; i < attempts; i++)); do
    run "$@"
    if [[ "$status" -eq 0 ]] ; then
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

KRATOS_PG_CON="postgres://dbuser:secret@localhost:5434/default?sslmode=disable"

getEmailCode() {
  local email="$1"
  local query="SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"

  local result=$(psql $KRATOS_PG_CON -t -c "${query}")

  if [[ -z "$result" ]]; then
    echo "No message for email ${email}" >&2
    exit 1
  fi

  local code=$(echo "$result" | grep -Eo '[0-9]{6}' | head -n1)

  echo "$code"
}

create_user() {
  email="user$(date +%s%N)@example.com"

  flowId=$(curl -s -X GET \
      -H "Accept: application/json" \
      "$KRATOS_PUBLIC_SELFSERVICE_ENDPOINT/registration/api" | jq -r '.id')

  response=$(curl -s -X POST "$KRATOS_PUBLIC_SELFSERVICE_ENDPOINT/registration?flow=$flowId" \
  -H "Content-Type: application/json" \
  -d '{
    "method": "code",
    "traits": {
      "email": "'"$email"'"
    }
  }')

  code=$(getEmailCode "$email")

  verification_response=$(curl -s -X POST "$KRATOS_PUBLIC_SELFSERVICE_ENDPOINT/registration?flow=$flowId" \
  -H "Content-Type: application/json" \
  -d '{
    "code": "'"$code"'",
    "method": "code",
    "traits": {
      "email": "'"$email"'"
    }
  }')

  # Extract and print the session token if needed
  token=$(echo $verification_response | jq -r '.session_token')
  echo $token
}

add() {
  sum=0
  for num in "$@"
  do
    sum=$(echo "scale=2; $sum + $num" | bc)
  done
  echo $sum
}

assert_assets_liabilities_equity() {
  exec_cala_graphql 'assets-liabilities-equity'
  echo $(graphql_output)

  assets_usdt=$(graphql_output '.data.balanceSheet.byJournalId.assets.usdtBalance.settled.normalBalance.units')
  assets_usd=$(graphql_output '.data.balanceSheet.byJournalId.assets.usdBalance.settled.normalBalance.units')
  assets=$( add $assets_usdt $assets_usd )

  liabilities_usdt=$(graphql_output '.data.balanceSheet.byJournalId.liabilities.usdtBalance.settled.normalBalance.units')
  liabilities_usd=$(graphql_output '.data.balanceSheet.byJournalId.liabilities.usdBalance.settled.normalBalance.units')
  liabilities=$(add "$liabilities_usdt" "$liabilities_usd")

  equity_usdt=$(graphql_output '.data.balanceSheet.byJournalId.equity.usdtBalance.settled.normalBalance.units')
  equity_usd=$(graphql_output '.data.balanceSheet.byJournalId.equity.usdBalance.settled.normalBalance.units')
  equity=$(add "$equity_usdt" "$equity_usd")

  liabilities_and_equity=$( add "$liabilities" "$equity" )

  [[ "$assets" == "$liabilities_and_equity" ]] || exit 1
}
