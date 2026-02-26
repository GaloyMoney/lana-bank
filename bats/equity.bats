#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

@test "equity: can add usd equity" {
  skip "TODO: rewrite this broken test to use lanacli-only flow"
}
