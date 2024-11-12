#!/usr/bin/env bash

set -eu

ADMIN_URL="https://admin.staging.lava.galoy.io"

CACHE_DIR=/tmp/lava-cache
mkdir -p $CACHE_DIR

cookie_jar() {
  echo "$CACHE_DIR/$1.jar"
}

pushd cypress/mailinator-fetch-inbox
nix develop -c pnpm i
popd

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

echo "==================== Fetching authentication link ===================="

curl -s "$ADMIN_URL" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: none' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null
curl -s "$ADMIN_URL/api/auth/signin?callbackUrl=%2F" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: none' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null
curl -s "$ADMIN_URL/api/auth/signin" -H 'accept: image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8' "${common_headers[@]}" -H "referer: $ADMIN_URL/api/auth/signin?callbackUrl=%2F" -H 'sec-fetch-dest: image' -H 'sec-fetch-mode: no-cors' -H 'sec-fetch-site: same-origin' >> /dev/null

csrfToken=$(cat "$(cookie_jar 'admin')" | grep "csrf-token" | sed 's/.*next-auth.csrf-token\s*\([^%]*\)%.*/\1/')
curl -s "$ADMIN_URL/api/auth/signin/email" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H 'content-type: application/x-www-form-urlencoded' -H "origin: $ADMIN_URL" -H "referer: $ADMIN_URL/api/auth/signin" \
  --data-raw "csrfToken=$csrfToken&email=galoysuperuser%40mailinator.com" >> /dev/null
curl -s "$ADMIN_URL/api/auth/verify-request?provider=email&type=email" -H 'accept: text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7' "${common_headers[@]}" -H "referer: $ADMIN_URL/api/auth/signin" -H 'sec-fetch-dest: document' -H 'sec-fetch-mode: navigate' -H 'sec-fetch-site: same-origin' -H 'sec-fetch-user: ?1' -H 'upgrade-insecure-requests: 1' >> /dev/null

pushd cypress/mailinator-fetch-inbox
  LINK=$(nix develop -c node index.js galoysuperuser admin@lava.galoy.io | jq -r '.clickablelinks[].link')
popd

echo "==================== Running cypress on browserstack ===================="

export MAGIC_LINK="$LINK"
nix develop -c pnpm run browserstack:run
