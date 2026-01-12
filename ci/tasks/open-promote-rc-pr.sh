#!/bin/bash

set -eu

VERSION=$(cat version/version)
echo "Promoting to version: $VERSION"

pushd rc-version-repo
cat version

cat <<EOF >> ../body.md
# Promote RC to ${VERSION}

This PR promotes the RC version to final release version ${VERSION}.

Once merged, the main release pipeline will pick up this version.
EOF

export GH_TOKEN="$(ghtoken generate -b "${GH_APP_PRIVATE_KEY}" -i "${GH_APP_ID}" | jq -r '.token')"

gh pr close ${BOT_BRANCH} || true
gh pr create \
  --title "chore: promote RC to ${VERSION}" \
  --body-file ../body.md \
  --base ${BRANCH} \
  --head ${BOT_BRANCH} \
  --label galoybot

