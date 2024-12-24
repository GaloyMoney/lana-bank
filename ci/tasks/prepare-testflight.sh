#!/bin/bash

set -eu

echo "Preparing testflight"

cp -r pipeline-tasks/ci/testflight/${CHART} testflight/tf
cp -r repo/chart/${CHART} testflight/tf/chart

cat <<EOF > testflight/tf/terraform.tfvars
testflight_namespace = "${CHART}-testflight-$(cat repo/.git/short_ref)"
EOF

cat <<EOF > testflight/env_name
${CHART}-testflight-$(cat repo/.git/short_ref)
EOF

if [[ ${ADDITIONAL_CHART} != "" ]]; then
  cp -r repo/chart/${ADDITIONAL_CHART} testflight/tf/${ADDITIONAL_CHART}
fi
