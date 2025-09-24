#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-custody.e2e-logs"
RUN_LOG_FILE="credit-facility-custody.run.e2e-logs"

setup_file() {
  start_server
  login_superadmin
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_collateral() {
  credit_facility_proposal_id=$1

  variables=$(
    jq -n \
      --arg creditFacilityProposalId "$credit_facility_proposal_id" \
    '{ id: $creditFacilityProposalId }'
  )
  exec_admin_graphql 'find-credit-facility-proposal' "$variables"
  collateral=$(graphql_output '.data.creditFacilityProposal.collateral.btcBalance')
  [[ "$collateral" -eq 1000 ]] || exit 1
}

@test "credit-facility-custody: can create with mock custodian" {
  # Setup prerequisites
  customer_id=$(create_customer)

  retry 80 1 wait_for_checking_account "$customer_id"

  variables=$(
    jq -n \
      --arg customerId "$customer_id" \
    '{
      id: $customerId
    }'
  )
  exec_admin_graphql 'customer' "$variables"

  deposit_account_id=$(graphql_output '.data.customer.depositAccount.depositAccountId')
  [[ "$deposit_account_id" != "null" ]] || exit 1

  facility=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --arg disbursal_credit_account_id "$deposit_account_id" \
    --argjson facility "$facility" \
    '{
      input: {
        customerId: $customerId,
        facility: $facility,
        disbursalCreditAccountId: $disbursal_credit_account_id,
        custodianId: "00000000-0000-0000-0000-000000000000",
        terms: {
          annualRate: "12",
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
          oneTimeFeeRate: "5",
          disbursalPolicy: "MULTIPLE",
          duration: { period: "MONTHS", units: 3 },
          interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
          obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
          obligationLiquidationDurationFromDue: { period: "DAYS", units: 60 },
          liquidationCvl: "105",
          marginCallCvl: "125",
          initialCvl: "140"
        }
      }
    }'
  )

  exec_admin_graphql 'credit-facility-proposal-create' "$variables"

  credit_facility_proposal_id=$(graphql_output '.data.creditFacilityProposalCreate.creditFacilityProposal.creditFacilityProposalId')
  [[ "$credit_facility_proposal_id" != "null" ]] || exit 1

  cache_value 'credit_facility_proposal_id' "$credit_facility_proposal_id"

  address=$(graphql_output '.data.creditFacilityProposalCreate.creditFacilityProposal.wallet.address')
  [[ "$address" == "bt1qaddressmock" ]] || exit 1
}

@test "credit-facility-custody: cannot update manually collateral with a custodian" {
  credit_facility_proposal_id=$(read_value 'credit_facility_proposal_id')

  variables=$(
    jq -n \
      --arg credit_facility_proposal_id "$credit_facility_proposal_id" \
      --arg effective "$(naive_now)" \
    '{
      input: {
        creditFacilityProposalId: $credit_facility_proposal_id,
        collateral: 50000000,
        effective: $effective,
      }
    }'
  )
  exec_admin_graphql 'credit-facility-proposal-collateral-update' "$variables"
  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "ManualUpdateError" ]] || exit 1
}

@test "credit-facility-custody: can update collateral by a custodian" {
  credit_facility_proposal_id=$(read_value 'credit_facility_proposal_id')

  variables=$(
    jq -n \
      --arg credit_facility_proposal_id "$credit_facility_proposal_id" \
    '{ id: $credit_facility_proposal_id }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  collateral=$(graphql_output '.data.creditFacility.balance.collateral.btcBalance')
  [[ "$collateral" -eq 0 ]] || exit 1

  # external wallet ID 123 is hard coded in mock custodian
  curl -s -X POST --json '{"wallet": "123", "balance": 1000}' http://localhost:5253/webhook/custodian/mock

  retry 10 1 wait_for_collateral "$credit_facility_proposal_id"
}
