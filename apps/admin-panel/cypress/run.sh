#!/usr/bin/env bash

set -eu

EXECUTION_MODE="${1:-ui}"

ADMIN_URL="http://localhost:4455/admin-panel"
MAILHOG_URL="http://localhost:8025"
email="admin%40galoy.io"

CACHE_DIR=/tmp/lava-cache
rm -rf $CACHE_DIR || true
mkdir -p $CACHE_DIR

cookie_jar() {
  echo "$CACHE_DIR/$1.jar"
}

common_headers=(
  -b "$(cookie_jar 'admin')"
  -c "$(cookie_jar 'admin')"
  -H 'accept-language: en-GB,en-US;q=0.9,en;q=0.8'
  -H 'cache-control: no-cache'
  -H 'pragma: no-cache'
  -H 'sec-ch-ua: "Not)A;Brand";v="99", "Google Chrome";v="127", "Chromium";v="127"'
  -H 'sec-ch-ua-mobile: ?0'
  -H 'sec-ch-ua-platform: "macOS"'
  -H 'user-agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36'
)

curl -s "$ADMIN_URL" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: none' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null
curl -s "$ADMIN_URL/api/auth/signin?callbackUrl=%2F" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: none' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null
curl -s "$ADMIN_URL/api/auth/signin" -H 'accept: image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8' "${common_headers[@]}" -H "referer: $ADMIN_URL/api/auth/signin?callbackUrl=%2F" -H 'sec-fetch-dest: image' -H 'sec-fetch-mode: no-cors' -H 'sec-fetch-site: same-origin' >> /dev/null

csrfToken=$(cat "$(cookie_jar 'admin')" | grep "csrf-token" | sed 's/.*next-auth.csrf-token\s*\([^%]*\)%.*/\1/')
curl -s "$ADMIN_URL/api/auth/signin/email" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'content-type: application/x-www-form-urlencoded' -H "origin: $ADMIN_URL" -H "referer: $ADMIN_URL/api/auth/signin" \
  --data-raw "csrfToken=$csrfToken&email=$email" >> /dev/null
curl -s "$ADMIN_URL/api/auth/verify-request?provider=email&type=email" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H "referer: $ADMIN_URL/api/auth/signin" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: same-origin' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null

sleep 2

get_magiclink_local() { 
    curl -s http://localhost:8025/api/v2/messages | \
    jq -r '.items[0].MIME.Parts[0].Body' | \
    perl -MMIME::QuotedPrint -pe '$_=MIME::QuotedPrint::decode($_);' | \
    grep -o 'http://.*' | \
    sed 's/=3D/=/g; s/%3A/:/g; s/%2F/\//g; s/%3F/?/g; s/%3D/=/g; s/%26/\&/g; s/%40/@/g'
}

echo "==================== Fetching authentication link locally from mailhog ===================="
export MAGIC_LINK="$(get_magiclink_local)"
echo MAGIC_LINK: $MAGIC_LINK
if [[ $MAGIC_LINK == "" ]]; then
  echo "Error: Could not retrieve magic link"
  exit 1
fi

# This is a workaround to work with cypress and the bundler module resolution
cp tsconfig.json tsconfig.json.bak
trap '[ -f tsconfig.json.bak ] && mv tsconfig.json.bak tsconfig.json' EXIT
sed -i 's/"moduleResolution": *"bundler"/"moduleResolution": "node"/' tsconfig.json

echo "==================== Running cypress ===================="
if [[ $EXECUTION_MODE == "ui" ]]; then
  nix develop -c pnpm run cypress:run-local
elif [[ $EXECUTION_MODE == "headless" ]]; then
  nix develop -c pnpm run cypress:run-headless
elif [[ $EXECUTION_MODE == "browserstack" ]]; then
  nix develop -c pnpm run cypress:run-browserstack
fi
