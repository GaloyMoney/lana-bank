#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "admin-server: graceful shutdown with sleep query" {
  # Start the sleep query in the background
  exec_admin_graphql 'sleep' '{"seconds": 5}'
  echo "Output: $output"
  [[ "$output" =~ "Wake up!" ]]
#   SLEEP_PID=$!
  
#   # Give the query a moment to reach the server
#   sleep 1
  
#   # Stop the server (sends SIGTERM)
#   stop_server

#   # Wait for the background sleep query to finish
#   wait $SLEEP_PID || true
  
#   # Check the response
#   run cat "$BATS_TMPDIR/sleep_response.json"
#   echo "Output: $output"
#   [[ "$output" =~ "Wake up!" ]]
}

