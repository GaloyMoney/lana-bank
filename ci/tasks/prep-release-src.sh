#!/bin/bash

set -eu

pushd repo

export prev_version=$(cog get-version -f 0.0.0)
echo $prev_version
export new_version=$(cat ../version/version)

if [[ $prev_version == "0.0.0" ]]; then
  git cliff --config ../pipeline-tasks/ci/vendor/config/git-cliff.toml > ../artifacts/gh-release-notes.md
else
  git cliff --config ../pipeline-tasks/ci/vendor/config/git-cliff.toml --ignore-tags ".*rc.*" $prev_version.. --tag $new_version > ../artifacts/gh-release-notes.md
fi

popd

echo "CHANGELOG:"
echo "-------------------------------"
cat artifacts/gh-release-notes.md
echo "-------------------------------"

echo -n "Release Version: "
echo $new_version
echo ""

echo $new_version > artifacts/gh-release-tag
echo "v$new_version Release" > artifacts/gh-release-name
