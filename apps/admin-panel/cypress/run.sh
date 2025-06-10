#!/usr/bin/env bash

set -eux

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
  
  echo "--- Starting superadmin login process ---"
  echo "Admin URL: $ADMIN_URL"
  echo "Email: $email"

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

  echo "--- Getting login flow ---"
  local loginFlow=$(curl -s -X GET "$ADMIN_URL/self-service/login/browser" "${common_headers[@]}")
  echo "Login flow response: $loginFlow"
  local flowId=$(echo $loginFlow | jq -r '.id')
  local csrfToken=$(echo $loginFlow | jq -r '.ui.nodes[] | select(.attributes.name == "csrf_token") | .attributes.value')
  echo "Flow ID: $flowId"
  echo "CSRF Token: $csrfToken"

  variables=$(jq -n \
    --arg email "$email" \
    --arg csrfToken "$csrfToken" \
    '{ identifier: $email, method: "code", csrf_token: $csrfToken }' \
  )
  echo "--- Sending login request ---"
  echo "Variables: $variables"
  local loginResponse=$(curl -s -X POST -H "content-type: application/json" -d "$variables" "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}")
  echo "Login response: $loginResponse"

  sleep 2

  echo "--- Fetching verification code from database ---"
  local query="SELECT body FROM courier_messages WHERE recipient='${email}' ORDER BY created_at DESC LIMIT 1;"
  echo "Query: $query"
  local result=$(podman exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -t -c "${query}")
  echo "DB result: $result"

  if [[ -z "$result" ]]; then
    echo "No message for email ${email}" >&2
    echo "--- Checking all messages in database ---"
    podman exec lana-bank-kratos-admin-pg-1 psql -U dbuser -d default -c "SELECT recipient, created_at, body FROM courier_messages ORDER BY created_at DESC LIMIT 10;"
    exit 1
  fi

  local code=$(echo "$result" | grep -Eo '[0-9]{6}' | head -n1)
  echo "Verification code: $code"

  echo "--- Completing login with verification code ---"
  local loginFlow=$(curl -s -X GET "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}")
  local csrfToken=$(echo $loginFlow | jq -r '.ui.nodes[] | select(.attributes.name == "csrf_token") | .attributes.value')
  echo "New CSRF Token: $csrfToken"

  variables=$(jq -n \
    --arg email "$email" \
    --arg code "$code" \
    --arg csrfToken "$csrfToken" \
    '{ identifier: $email, method: "code", csrf_token: $csrfToken, code: $code }' \
  )
  echo "Final login variables: $variables"
  local finalResponse=$(curl -s -X POST -H "content-type: application/json" -d "$variables" "$ADMIN_URL/self-service/login?flow=$flowId" "${common_headers[@]}")
  echo "Final login response: $finalResponse"

  cookies=$(cat $(cookie_jar 'admin') | tail -n 2)
  echo -n $cookies > $(cookie_jar 'admin')
  echo "--- Login process completed ---"
  echo "Cookie file contents: $(cat $(cookie_jar 'admin'))"
}

login_superadmin

echo "--- Processing cookies for Cypress ---"
COOKIE1_NAME=$(cat $(cookie_jar 'admin') | cut -d" " -f6)
COOKIE1_VALUE=$(cat $(cookie_jar 'admin') | cut -d" " -f7)
COOKIE2_NAME=$(cat $(cookie_jar 'admin') | cut -d" " -f13)
COOKIE2_VALUE=$(cat $(cookie_jar 'admin') | cut -d" " -f14)

echo "Cookie 1: $COOKIE1_NAME=$COOKIE1_VALUE"
echo "Cookie 2: $COOKIE2_NAME=$COOKIE2_VALUE"

export COOKIES=$(jq -n \
  --arg cookie1_name "$COOKIE1_NAME" \
  --arg cookie1_value "$COOKIE1_VALUE" \
  --arg cookie2_name "$COOKIE2_NAME" \
  --arg cookie2_value "$COOKIE2_VALUE" \
'{ cookie1_name: $cookie1_name, cookie1_value: $cookie1_value, cookie2_name: $cookie2_name, cookie2_value: $cookie2_value }' | base64 -w 0)

echo "Encoded cookies: $COOKIES"

# This is a workaround to work with cypress and the bundler module resolution
cp tsconfig.json tsconfig.json.bak
trap '[ -f tsconfig.json.bak ] && mv tsconfig.json.bak tsconfig.json' EXIT
sed -i 's/"moduleResolution": *"bundler"/"moduleResolution": "node"/' tsconfig.json

if [[ ${CI:-} == "true" ]]; then
  echo "Installing Cypress binary if missing..."
  pnpm exec cypress install
fi

echo "==================== Running cypress ===================="
echo "Execution mode: $EXECUTION_MODE"
echo "Current working directory: $(pwd)"
echo "Environment variables:"
echo "  CI: ${CI:-not set}"
echo "  COOKIES: ${COOKIES:0:50}..." # Show first 50 chars of cookies

echo "--- Cypress environment check ---"
echo "Cypress config file exists: $(test -f cypress.config.ts && echo 'YES' || echo 'NO')"
echo "Cypress binary location: $(which cypress 2>/dev/null || echo 'NOT FOUND')"
echo "Cypress version: $(pnpm exec cypress --version 2>/dev/null || echo 'NOT AVAILABLE')"
echo "Node.js version: $(node --version 2>/dev/null || echo 'NOT AVAILABLE')"
echo "Pnpm version: $(pnpm --version 2>/dev/null || echo 'NOT AVAILABLE')"
echo "--- Environment variables ---"
echo "  CI: ${CI:-not set}"
echo "  DISPLAY: ${DISPLAY:-not set}"
echo "  HOME: ${HOME:-not set}"
echo "  PATH: ${PATH:0:100}..." # Show first 100 chars of PATH

# Ensure cypress directories exist
mkdir -p cypress/results cypress/videos cypress/screenshots cypress/manuals/screenshots
echo "Created cypress output directories"

if [[ $EXECUTION_MODE == "ui" ]]; then
  echo "Running cypress in UI mode..."
  if ! nix develop -c pnpm run cypress:run-local; then
    echo "ERROR: Cypress UI mode failed with exit code $?"
    exit 1
  fi
elif [[ $EXECUTION_MODE == "headless" ]]; then
  echo "Running cypress in headless mode..."
  echo "About to execute: nix develop -c pnpm run cypress:run-headless"
  if ! timeout 600 nix develop -c pnpm run cypress:run-headless; then
    local exit_code=$?
    echo "ERROR: Cypress headless mode failed with exit code $exit_code"
    echo "--- Debugging information ---"
    echo "Process list:"
    ps aux | grep -E "(cypress|chrome|electron)" || echo "No cypress/chrome/electron processes found"
    echo "Network connections:"
    netstat -tulpn | grep -E "(3001|4455|5253)" || echo "No relevant network connections found"
    echo "Disk space:"
    df -h
    echo "Memory usage:"
    free -h 2>/dev/null || echo "free command not available"
    exit $exit_code
  fi
elif [[ $EXECUTION_MODE == "browserstack" ]]; then
  echo "Running cypress in browserstack mode..."
  if ! nix develop -c pnpm run cypress:run-browserstack; then
    echo "ERROR: Cypress browserstack mode failed with exit code $?"
    exit 1
  fi
  mv $(find build_artifacts -type d -name "screenshots") cypress/manuals
  rm -rf build_artifacts
fi

echo "--- Cypress execution completed successfully ---"
