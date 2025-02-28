#!/bin/zsh

# Check if required environment variables are set
if [[ -z "$SUMSUB_KEY" || -z "$SUMSUB_SECRET" ]]; then
    echo "Error: SUMSUB_KEY or SUMSUB_SECRET environment variables not found"
    exit 1
fi

# Generate a random UUID v4 for the customer ID using macOS's uuidgen
CUSTOMER_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')

# Save customer ID to a file for later use
echo "$CUSTOMER_ID" > .sumsub_customer_id
echo "Customer ID saved to .sumsub_customer_id"

LEVEL_NAME="basic-kyc-level"

# API endpoint
URL_PATH="/resources/sdkIntegrations/levels/$LEVEL_NAME/websdkLink?&externalUserId=$CUSTOMER_ID"
FULL_URL="https://api.sumsub.com$URL_PATH"

# Empty JSON body
BODY="{}"

# Current timestamp in seconds (macOS compatible)
TS=$(date +%s)

# Create signature
METHOD="POST"
SIGNATURE=$(echo -n "$TS$METHOD$URL_PATH$BODY" | \
  openssl dgst -sha256 -hmac "$SUMSUB_SECRET" -binary | \
  xxd -p | tr -d '\n')

echo "Generated customer ID: $CUSTOMER_ID"
echo "Making request to SumSub API..."

# Make the request
curl -X POST "$FULL_URL" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -H "X-App-Token: $SUMSUB_KEY" \
  -H "X-App-Access-Ts: $TS" \
  -H "X-App-Access-Sig: $SIGNATURE" \
  -d "$BODY"

# Add a newline after the response for better readability
echo ""
echo ""
echo "To get applicant data later, run:"
echo "./get_sumsub_applicant.sh \$(cat .sumsub_customer_id)"