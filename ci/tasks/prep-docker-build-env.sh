#!/bin/bash

if [[ -f version/version ]]; then
  echo "VERSION=$(cat version/version)" >> repo/.env
fi

echo "COMMITHASH=$(git rev-parse HEAD)" >> repo/.env
echo "BUILDTIME=$(date -u '+%F-%T')" >> repo/.env

export GH_TOKEN="$(ghtoken generate -b "${GH_APP_PRIVATE_KEY}" -i "${GH_APP_ID}" | jq -r '.token')"
echo "GH_TOKEN=$GH_TOKEN" >> repo/.env
