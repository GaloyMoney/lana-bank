#!/usr/bin/env bats

load "helpers"

# setup_file() {
#   start_server
# }

# teardown_file() {
#   stop_server
# }

@test "fixed-term-loan: can create a loan" {

  variables=$(
    jq -n \
    '{
      input: {
        bitfinexUserName: "bitfinexUserName",
      }
    }'
  )
  exec_graphql 'fixed-term-loan-create' "$variables"
  id=$(graphql_output '.data.fixedTermLoanCreate.loan.loanId')
  [[ "$id" != null ]] || exit 1;

  balance=$(graphql_output '.data.fixedTermLoanCreate.loan.balance.units')
  [[ "$balance" == "0" ]] || exit 1;
}
