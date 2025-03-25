#!/usr/bin/env bash

# Script to get an admin token for the MCP plugin
# Based on the login_superadmin function in helpers.bash

# Configuration
OATHKEEPER_PROXY="http://localhost:4455"
ADMIN_EMAIL="admin@galoy.io"
MAILHOG_ENDPOINT="http://localhost:8025"
GQL_ADMIN_ENDPOINT="${OATHKEEPER_PROXY}/admin/graphql"

# Temporary directory for cache
CACHE_DIR="/tmp/lana-mcp-cache"
mkdir -p "$CACHE_DIR"

# Function to cache a value
cache_value() {
  echo $2 > ${CACHE_DIR}/$1
}

# Function to read a cached value
read_value() {
  cat ${CACHE_DIR}/$1
}

# Function to get email verification code
getEmailCode() {
  local email="$1"

  local emails=$(curl -s -X GET "${MAILHOG_ENDPOINT}/api/v2/search?kind=to&query=${email}")
  
  if [[ $(echo "$emails" | jq '.total') -eq 0 ]]; then
    echo "No message for email ${email}" >&2
    exit 1
  fi

  local email_content=$(echo "$emails" | jq '.items[0].MIME.Parts[0].Body' | tr -d '"')
  local code=$(echo "$email_content" | grep -Eo '[0-9]{6}' | head -n1)

  echo "$code"
}

# Main function to get admin token
get_admin_token() {
  echo "Requesting login flow ID..." >&2
  flowId=$(curl -s -X GET -H "Accept: application/json" "${OATHKEEPER_PROXY}/admin/self-service/login/api" | jq -r '.id')
  
  echo "Initiating login for $ADMIN_EMAIL..." >&2
  variables=$(jq -n --arg email "$ADMIN_EMAIL" '{ identifier: $email, method: "code" }')
  curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/admin/self-service/login?flow=$flowId"
  sleep 1

  echo "Getting verification code from email..." >&2
  code=$(getEmailCode $ADMIN_EMAIL)
  echo "Verification code: $code" >&2
  
  echo "Completing login with verification code..." >&2
  variables=$(jq -n --arg email "$ADMIN_EMAIL" --arg code "$code" '{ identifier: $email, method: "code", code: $code }')
  session=$(curl -s -X POST -H "Accept: application/json" -H "Content-Type: application/json" -d "$variables" "${OATHKEEPER_PROXY}/admin/self-service/login?flow=$flowId")
  
  token=$(echo $session | jq -r '.session_token')
  cache_value "superadmin" $token
  
  echo "Admin token: $token" >&2
  echo $token
}

# Execute the function
get_admin_token 