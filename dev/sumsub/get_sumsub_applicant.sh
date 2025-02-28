#!/bin/bash

# SumSub Applicant Data Fetcher using curl
# Usage: ./get_sumsub_applicant.sh <external_user_id>

# Check if external_user_id is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <external_user_id>"
  exit 1
fi

# Get the external user ID from command line argument
EXTERNAL_USER_ID="$1"

# Check if required environment variables are set
if [ -z "$SUMSUB_KEY" ] || [ -z "$SUMSUB_SECRET" ]; then
  echo "Error: SUMSUB_KEY or SUMSUB_SECRET environment variables not set"
  exit 1
fi

# API endpoint
URL_PATH="/resources/applicants/-;externalUserId=$EXTERNAL_USER_ID/one"
FULL_URL="https://api.sumsub.com$URL_PATH"

# Current timestamp in seconds
TS=$(date +%s)

# Create signature (timestamp + HTTP method + URL path + body)
# For GET requests, the body is empty
METHOD="GET"
SIGNATURE=$(echo -n "$TS$METHOD$URL_PATH" | \
  openssl dgst -sha256 -hmac "$SUMSUB_SECRET" -binary | \
  xxd -p | tr -d '\n')

echo "Fetching SumSub data for externalUserId: $EXTERNAL_USER_ID"

# Make the request
curl -s -X GET "$FULL_URL" \
  -H "Accept: application/json" \
  -H "X-App-Token: $SUMSUB_KEY" \
  -H "X-App-Access-Ts: $TS" \
  -H "X-App-Access-Sig: $SIGNATURE" | \
  jq . 2>/dev/null || echo "Error: Failed to parse JSON response. Install jq for pretty printing."

# Note: This script uses jq for pretty-printing the JSON response.
# If jq is not installed, it will output the raw response.