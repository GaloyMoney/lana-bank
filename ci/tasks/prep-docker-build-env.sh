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
