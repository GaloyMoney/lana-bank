#!/bin/bash
set -eu

VERSION=$(cat version/version)
echo "Promoting to version: $VERSION"

pushd rc-version-repo

cat <<EOF >> ../body.md
# Promote RC to ${VERSION}

This PR promotes the RC version to final release version ${VERSION}.

Once merged, the main release pipeline will pick up this version.
EOF

if [[ -z $(git config --global user.email) ]]; then
  git config --global user.email "bot@galoy.io"
fi
if [[ -z $(git config --global user.name) ]]; then
  git config --global user.name "CI Bot"
fi

echo "$VERSION" > version
git add version
git commit -m "chore: promote RC to ${VERSION}"
