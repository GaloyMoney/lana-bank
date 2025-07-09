#!/bin/bash

# Submit finance transaction for non-existing applicant API example
# Usage: ./submit_finance_transaction.sh

# Check if required environment variables are set
if [ -z "$SUMSUB_KEY" ] || [ -z "$SUMSUB_SECRET" ]; then
  echo "Error: SUMSUB_KEY or SUMSUB_SECRET environment variables not set"
  exit 1
fi

# Generate a random transaction ID
TXN_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
# Generate a random external user ID
EXTERNAL_USER_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
# Generate a random counterparty ID
COUNTERPARTY_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')

# API endpoint
URL_PATH="/resources/applicants/-/kyt/txns/-/data"
FULL_URL="https://api.sumsub.com$URL_PATH"

# Current timestamp in seconds
TS=$(date +%s)
# Format timestamp for txnDate
DATE_FORMAT=$(date -u +"%Y-%m-%d %H:%M:%S+0000")

# Request body based on the example from the website
BODY='{
  "txnId": "'$TXN_ID'",
  "txnType": "finance",
  "txnDirection": "in",
  "externalTxnId": "'$TXN_ID'",
  "txnStatus": "pending",
  "txnTimestamp": '$(date +%s)',
  "txnDate": "'$DATE_FORMAT'",
  "info": {
    "type": "CustomTxnType",
    "direction": "in",
    "amount": 1000,
    "currencyCode": "USD",
    "currencyType": "fiat",
    "amountInDefaultCurrency": 1000,
    "defaultCurrencyCode": "USD",
    "paymentDetails": "Test Transaction"
  },
  "applicant": {
    "type": "individual",
    "externalUserId": "'$EXTERNAL_USER_ID'",
    "fullName": "John Doe",
    "dob": "1980-01-01",
    "placeOfBirth": "New York",
    "address": {
      "buildingName": "Apartment Building",
      "flatNumber": "101",
      "street": "Main Street",
      "state": "NY",
      "buildingNumber": "123",
      "town": "New York",
      "postCode": "10001",
      "country": "USA",
      "formattedAddress": "123 Main Street, New York, NY 10001"
    },
    "paymentMethod": {
      "type": "bank account",
      "accountId": "123456789",
      "issuingCountry": "USA"
    },
    "device": {
      "ipInfo": {
        "ip": "192.168.1.1",
        "countryCode3": "USA"
      }
    }
  },
  "counterparty": {
    "type": "individual",
    "externalUserId": "'$COUNTERPARTY_ID'",
    "fullName": "Jane Smith",
    "paymentMethod": {
      "type": "bank account",
      "accountId": "987654321",
      "issuingCountry": "USA"
    }
  },
  "levelName": "basic-kyc-level"
}'

# Create signature
METHOD="POST"
SIGNATURE=$(echo -n "$TS$METHOD$URL_PATH$BODY" | \
  openssl dgst -sha256 -hmac "$SUMSUB_SECRET" -binary | \
  xxd -p | tr -d '\n')

echo "Generated transaction ID: $TXN_ID"
echo "Generated external user ID: $EXTERNAL_USER_ID"
echo "Making request to SumSub API..."

# Make the request and store the response
RESPONSE=$(curl -s -X POST "$FULL_URL" \
  -H "Accept: application/json" \
  -H "Content-Type: application/json" \
  -H "X-App-Token: $SUMSUB_KEY" \
  -H "X-App-Access-Ts: $TS" \
  -H "X-App-Access-Sig: $SIGNATURE" \
  -d "$BODY")

echo "Response: $RESPONSE"
