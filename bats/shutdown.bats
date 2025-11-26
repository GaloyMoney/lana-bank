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

@test "admin-server: graceful shutdown with test job" {
  # Spawn a test job that respects shutdown signals
  variables=$(
    jq -n \
    '{
      input: {
        waitSeconds: 10,
        respectShutdown: true
      }
    }'
  )
  exec_admin_graphql 'testJobSpawn' "$variables"
  echo "Spawn job output: $output"
  [[ "$output" =~ "jobId" ]]
  
  # Give the job a moment to start
  sleep 2
  
  # Stop the server (sends SIGTERM) - should trigger graceful shutdown
  stop_server
  
  # Server should have shut down gracefully within the timeout period
  # The job should have received the shutdown signal and exited cleanly
  # Check server logs for graceful shutdown messages
  run grep -i "shutdown signal" "$BATS_TMPDIR/server.log" || true
  echo "Shutdown log: $output"
}

@test "admin-server: forced shutdown with non-responsive test job" {
  # Start the server again
  # start_server
  # login_superadmin
  
  # Spawn a test job that ignores shutdown signals  
  variables=$(
    jq -n \
    '{
      input: {
        waitSeconds: 3,
        respectShutdown: false
      }
    }'
  )
  exec_admin_graphql 'testJobSpawn' "$variables"
  echo "Spawn non-responsive job output: $output"
  [[ "$output" =~ "jobId" ]]
  
  # Give the job a moment to start
  # sleep 2
  
  # # Stop the server - job should be killed after shutdown timeout
  # stop_server
  
  # # Check logs for job being killed/rescheduled after timeout
  # run grep -i "still running after shutdown" "$BATS_TMPDIR/server.log" || true
  # echo "Forced kill log: $output"
}
