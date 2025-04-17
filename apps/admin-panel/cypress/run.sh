#!/usr/bin/env bash

set -eu

EXECUTION_MODE="${1:-ui}"

CACHE_DIR=/tmp/lana-cache
rm -rf $CACHE_DIR || true
mkdir -p $CACHE_DIR

cookie_jar() {
  echo "$CACHE_DIR/$1.jar"
}

login_superadmin() {
  ADMIN_URL="http://localhost:4455/admin"
  email="admin@galoy.io"

  common_headers=(
    -b "$(cookie_jar 'admin')"
    -c "$(cookie_jar 'admin')"
    -H 'accept: application/json, text/plain, */*'
    -H 'accept-language: en-GB,en-US;q=0.9,en;q=0.8'
    -H 'cache-control: no-cache'
    -H 'pragma: no-cache'
    -H 'sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"'
    -H 'sec-ch-ua-mobile: ?0'
    -H 'sec-ch-ua-platform: "macOS"'
    -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36'
  )

  local loginFlow=$(curl -s -X GET "$ADMIN_URL/self-service/login/browser" "${common_headers[@]}")
  local flowId=$(echo $loginFlow | jq -r '.id')
  local csrfToken=$(echo $loginFlow | jq -r '.ui.nodes[] | select(.attributes.name == "csrf_token") | .attributes.value')

  variables=$(jq -n \
    --arg email "$email" \
    --arg csrfToken "$csrfToken" \
    '{ identifier: $email, method: "code", csrf_token: $csrfToken }' \
  )
  curl -s -X POST -H "content-type: application/json" -d "$variables" "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}" >> /dev/null

  sleep 2

  KRATOS_PG_CON="postgres://dbuser:secret@localhost:5434/default?sslmode=disable"

  local query="SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"
  local result=$(psql $KRATOS_PG_CON -t -c "${query}")

  if [[ -z "$result" ]]; then
    echo "No message for email ${email}" >&2
    exit 1
  fi

  local code=$(echo "$result" | grep -Eo '[0-9]{6}' | head -n1)

  local loginFlow=$(curl -s -X GET "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}")
  local csrfToken=$(echo $loginFlow | jq -r '.ui.nodes[] | select(.attributes.name == "csrf_token") | .attributes.value')

  variables=$(jq -n \
    --arg email "$email" \
    --arg code "$code" \
    --arg csrfToken "$csrfToken" \
    '{ identifier: $email, method: "code", csrf_token: $csrfToken, code: $code }' \
  )
  curl -s -X POST -H "content-type: application/json" -d "$variables" "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}" >> /dev/null

  cookies=$(cat $(cookie_jar 'admin') | tail -n 2)
  echo -n $cookies > $(cookie_jar 'admin')
}

login_superadmin

COOKIE1_NAME=$(cat $(cookie_jar 'admin') | cut -d" " -f6)
COOKIE1_VALUE=$(cat $(cookie_jar 'admin') | cut -d" " -f7)
COOKIE2_NAME=$(cat $(cookie_jar 'admin') | cut -d" " -f13)
COOKIE2_VALUE=$(cat $(cookie_jar 'admin') | cut -d" " -f14)

export COOKIES=$(jq -n \
  --arg cookie1_name "$COOKIE1_NAME" \
  --arg cookie1_value "$COOKIE1_VALUE" \
  --arg cookie2_name "$COOKIE2_NAME" \
  --arg cookie2_value "$COOKIE2_VALUE" \
'{ cookie1_name: $cookie1_name, cookie1_value: $cookie1_value, cookie2_name: $cookie2_name, cookie2_value: $cookie2_value }' | base64 -w 0)

# This is a workaround to work with cypress and the bundler module resolution
cp tsconfig.json tsconfig.json.bak
trap '[ -f tsconfig.json.bak ] && mv tsconfig.json.bak tsconfig.json' EXIT
sed -i 's/"moduleResolution": *"bundler"/"moduleResolution": "node"/' tsconfig.json

if [[ ${CI:-} == "true" ]]; then
  echo "Installing Cypress binary if missing..."
  pnpm exec cypress install
fi

echo "==================== Running cypress ===================="
if [[ $EXECUTION_MODE == "ui" ]]; then
  nix develop -c pnpm run cypress:run-local
elif [[ $EXECUTION_MODE == "headless" ]]; then
  nix develop -c pnpm run cypress:run-headless
elif [[ $EXECUTION_MODE == "browserstack" ]]; then
  nix develop -c pnpm run cypress:run-browserstack
  mv $(find build_artifacts -type d -name "screenshots") cypress/manuals
  rm -rf build_artifacts
fi
