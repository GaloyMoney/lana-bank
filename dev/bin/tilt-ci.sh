#!/usr/bin/env bash

make reset-tf-state || true
make dev-down || true

cd dev
nohup tilt up > tilt-up.log 2>&1 < /dev/null &
sleep 5

echo "sending requests to tilt-apiserver now..."

for i in {1..60}; do
    if tilt get uiresource core -o json | jq -e '.status.runtimeStatus == "error"' > /dev/null; then
        echo "uiresource/core is in error state. retrying..."
        tilt trigger core
        break
    fi
    sleep 1
done

tilt wait --for=condition=Ready --timeout=600s uiresource/core
tilt wait --for=condition=Ready --timeout=1200s uiresource/admin-panel
