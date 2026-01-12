#!/bin/bash

set -eu

# Copy binaries into repo if they exist (RC flow)
if [[ -d binaries ]]; then
  cp -r binaries repo/
fi

if [[ -f version/version ]]; then
  echo "VERSION=$(cat version/version)" >> repo/.env
fi

echo "COMMITHASH=$(cd repo && git rev-parse HEAD)" >> repo/.env
echo "BUILDTIME=$(date -u '+%F-%T')" >> repo/.env

# Generate GH_TOKEN only if credentials are provided (release flow, not RC)
if [[ -n "${GH_APP_ID:-}" ]] && [[ -n "${GH_APP_PRIVATE_KEY:-}" ]]; then
  export GH_TOKEN="$(ghtoken generate -b "${GH_APP_PRIVATE_KEY}" -i "${GH_APP_ID}" | jq -r '.token')"
  echo "GH_TOKEN=$GH_TOKEN" >> repo/.env
fi
