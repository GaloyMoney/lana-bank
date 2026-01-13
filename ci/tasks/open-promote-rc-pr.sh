#!/bin/bash

set -eu

VERSION=$(cat version/version)
echo "Promoting to version: $VERSION"

pushd repo

export GH_TOKEN="$(ghtoken generate -b "${GH_APP_PRIVATE_KEY}" -i "${GH_APP_ID}" | jq -r '.token')"

gh pr close ${BOT_BRANCH} || true
gh pr create \
  --title "chore: promote RC to ${VERSION}" \
  --base ${BRANCH} \
  --head ${BOT_BRANCH} \
  --label galoybot
