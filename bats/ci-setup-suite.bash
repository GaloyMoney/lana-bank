#!/usr/bin/env bash

export REPO_ROOT=$(git rev-parse --show-toplevel)
source "${REPO_ROOT}/bats/helpers.bash"

setup_suite() {
  echo "Starting server" >> ./test.log
  start_server
  echo "Server started" >> ./test.log
}

teardown_suite() {
  stop_server

  lsof -i :5253 | tail -n 1 | cut -d" " -f2 | xargs -L 1 kill -9 || true
  lsof -i :5254 | tail -n 1 | cut -d" " -f2 | xargs -L 1 kill -9 || true
}
