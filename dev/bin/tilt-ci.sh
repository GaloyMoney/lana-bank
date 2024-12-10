#!/bin/bash

REPO_ROOT=$(git rev-parse --show-toplevel)

[ -f tmp.env.ci ] && source tmp.env.ci || true

cd "${REPO_ROOT}"
tilt ci --file dev/Tiltfile | tee tilt.log 
status=${PIPESTATUS[0]}

if [[ $status -eq 0 ]]; then
  echo "Tilt CI passed"
fi

exit "$status"
