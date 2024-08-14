#!/usr/bin/env bats

load "helpers"

setup_file() {
  start_server
}

teardown_file() {
  stop_server
}

loan_balance() {
  variables=$(
    jq -n \
    --arg loanId "$1" \
    '{ id: $loanId }'
  )
  exec_admin_graphql 'find-loan' "$variables"

  outstanding_balance=$(graphql_output '.data.loan.balance.outstanding.usdBalance')
  cache_value 'outstanding' "$outstanding_balance"
  collateral_balance_sats=$(graphql_output '.data.loan.balance.collateral.btcBalance')
  cache_value 'collateral_sats' "$collateral_balance_sats"
  interest_incurred=$(graphql_output '.data.loan.balance.interestIncurred.usdBalance')
  cache_value 'interest' "$interest_incurred"
}

wait_for_interest() {
  loan_balance $1
  interest_incurred=$(read_value 'interest')
  [[ "$interest_incurred" -gt "0" ]] || return 1
}

@test "loan: loan lifecycle" {
  customer_id=$(create_customer)

  revenue_before=$(net_usd_revenue)

  principal=10000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --argjson principal "$principal" \
    '{
      input: {
        customerId: $customerId,
        desiredPrincipal: $principal,
        loanTerms: {
          annualRate: "12",
          interval: "END_OF_MONTH",
          duration: { period: "MONTHS", units: 3 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140"
        }
      }
    }'
  )
  exec_admin_graphql 'loan-create' "$variables"
  loan_id=$(graphql_output '.data.loanCreate.loan.loanId')
  [[ "$loan_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg loanId "$loan_id" \
    '{
      input: {
        loanId: $loanId,
        collateral: 233334,
      }
    }'
  )
  exec_admin_graphql 'loan-approve' "$variables"
  loan_id=$(graphql_output '.data.loanApprove.loan.loanId')
  [[ "$loan_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  exec_admin_graphql 'customer' "$variables"
  usd_balance=$(graphql_output '.data.customer.balance.checking.settled.usdBalance')
  [[ "$usd_balance" == "$principal" ]] || exit 1

  retry 20 1 wait_for_interest "$loan_id"
  interest_before=$(read_value "interest")
  outstanding_before=$(read_value "outstanding")
  expected_outstanding=$(add $principal $interest_before)
  [[ "$outstanding_before" == "$expected_outstanding" ]] || exit 1

  collateral_sats=$(read_value 'collateral_sats')
  [[ "$collateral_sats" == "233334" ]] || exit 1


  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
      --argjson amount "$outstanding_before" \
    '{
      input: {
        customerId: $customerId,
        amount: $amount,
      }
    }'
  )
  exec_admin_graphql 'record-deposit' "$variables"

  variables=$(
    jq -n \
      --arg loanId "$loan_id" \
      --argjson amount "$outstanding_before" \
    '{
      input: {
        loanId: $loanId,
        amount: $amount,
      }
    }'
  )
  exec_admin_graphql 'loan-partial-payment' "$variables"

  loan_balance "$loan_id"
  outstanding_after=$(read_value "outstanding")
  [[ "$outstanding_after" == "0" ]] || exit 1
  collateral_sats=$(read_value 'collateral_sats')
  [[ "$collateral_sats" == "0" ]] || exit 1

  revenue_after=$(net_usd_revenue)
  [[ $revenue_after -gt $revenue_before ]] || exit 1

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  exec_admin_graphql 'customer' "$variables"
  usd_balance=$(graphql_output '.data.customer.balance.checking.settled.usdBalance')
  [[ "$usd_balance" == "$principal" ]] || exit 1
}

@test "loan: paginated listing" {
  customer_id=$(create_customer)

  # Create 2 loans
  for i in {1..2}; do
    principal=10000
    variables=$(
      jq -n \
      --arg customerId "$customer_id" \
      --argjson principal "$principal" \
      '{
        input: {
          customerId: $customerId,
          desiredPrincipal: $principal,
          loanTerms: {
            annualRate: "12",
            interval: "END_OF_MONTH",
            duration: { period: "MONTHS", units: 3 },
            liquidationCvl: "105",
            marginCallCvl: "125",
            initialCvl: "140"
          }
        }
      }'
    )
    exec_admin_graphql 'loan-create' "$variables"
    loan_id=$(graphql_output '.data.loanCreate.loan.loanId')
    [[ "$loan_id" != "null" ]] || exit 1

    variables=$(
      jq -n \
        --arg loanId "$loan_id" \
      '{
        input: {
          loanId: $loanId,
          collateral: 233334,
        }
      }'
    )
    exec_admin_graphql 'loan-approve' "$variables"
    loan_id=$(graphql_output '.data.loanApprove.loan.loanId')
    [[ "$loan_id" != "null" ]] || exit 1
  done

  variables=$(
    jq -n \
      --argjson first 1 \
    '{ first: $first }'
  )
  exec_admin_graphql 'loan-list' "$variables"
  loan_id=$(graphql_output '.data.loans.edges[0].node.loanId')
  [[ "$loan_id" != "null" ]] || exit 1
  [[ "$(graphql_output '.data.loans.pageInfo.hasNextPage')" == "true" ]] || exit 1
  cursor=$(graphql_output '.data.loans.pageInfo.endCursor')

  variables=$(
    jq -n \
      --argjson first 1 \
      --arg cursor "$cursor" \
    '{ first: $first, after: $cursor }'
  )
  exec_admin_graphql 'loan-list' "$variables"
  loan_id=$(graphql_output '.data.loans.edges[0].node.loanId')
  [[ "$loan_id" != "null" ]] || exit 1
}
