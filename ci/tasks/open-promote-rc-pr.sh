#!/bin/bash

set -eu

VERSION=$(cat version/version)
echo "Promoting to version: $VERSION"

pushd repo

cat <<EOF > ../body.md
# Promote RC to ${VERSION}

This PR promotes the RC to the final version: ${VERSION}
EOF

export GH_TOKEN="$(ghtoken generate -b "${GH_APP_PRIVATE_KEY}" -i "${GH_APP_ID}" | jq -r '.token')"

gh pr close ${BOT_BRANCH} || true
gh pr create \
  --title "ci: promote RC to ${VERSION}" \
  --base ${BRANCH} \
  --body-file ../body.md \
  --head ${BOT_BRANCH} \
  --label promote-rc \
  --label galoybot \
  --draft
