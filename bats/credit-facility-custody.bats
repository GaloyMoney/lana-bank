#!/usr/bin/env bats

load "helpers"

PERSISTED_LOG_FILE="credit-facility-custody.e2e-logs"
RUN_LOG_FILE="credit-facility-custody.run.e2e-logs"

setup_file() {
  export LANA_DOMAIN_CONFIG_REQUIRE_VERIFIED_CUSTOMER_FOR_ACCOUNT=false
  start_server
  login_superadmin
  login_lanacli
  reset_log_files "$PERSISTED_LOG_FILE" "$RUN_LOG_FILE"
}

teardown_file() {
  stop_server
  cp "$LOG_FILE" "$PERSISTED_LOG_FILE"
}

wait_for_approval() {
  variables=$(
    jq -n \
      --arg creditFacilityProposalId "$1" \
    '{ id: $creditFacilityProposalId }'
  )
  exec_admin_graphql 'find-credit-facility-proposal' "$variables"
  echo "withdrawal | $i. $(graphql_output)" >> $RUN_LOG_FILE
  status=$(graphql_output '.data.creditFacilityProposal.status')
  [[ "$status" == "APPROVED" ]] || return 1
}

wait_for_collateral() {
  pending_credit_facility_id=$1

  variables=$(
    jq -n \
      --arg pendingCreditFacilityId "$pending_credit_facility_id" \
    '{ id: $pendingCreditFacilityId }'
  )
  exec_admin_graphql 'find-pending-credit-facility' "$variables"
  echo $(graphql_output) | jq .
  collateral=$(graphql_output '.data.pendingCreditFacility.collateral.btcBalance')
  [[ "$collateral" -eq 1000 ]] || exit 1
}


@test "credit-facility-custody: can create with mock custodian" {
  # Setup prerequisites
  customer_id=$(create_customer)

  deposit_account_id=$(create_deposit_account_for_customer "$customer_id")

  facility=100000
  variables=$(
    jq -n \
    --arg customerId "$customer_id" \
    --argjson facility "$facility" \
    '{
      input: {
        customerId: $customerId,
        facility: $facility,
        custodianId: "00000000-0000-0000-0000-000000000000",
        terms: {
          annualRate: "12",
          accrualCycleInterval: "END_OF_MONTH",
          accrualInterval: "END_OF_DAY",
          disbursalPolicy: "SINGLE_DISBURSAL",
          oneTimeFeeRate: "5",
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

  variables=$(
     jq -n \
      --arg creditFacilityProposalId "$credit_facility_proposal_id" \
    '{
      input: {
        creditFacilityProposalId: $creditFacilityProposalId,
        approved: true
      }
    }'
  )

  exec_admin_graphql 'credit-facility-proposal-customer-approval-conclude' "$variables"

  retry 30 2 wait_for_approval "$credit_facility_proposal_id"

  variables=$(
    jq -n \
      --arg pendingCreditFacilityId "$credit_facility_proposal_id" \
    '{ id: $pendingCreditFacilityId }'
  )

  exec_admin_graphql 'find-pending-credit-facility' "$variables"
  echo $(graphql_output) | jq .

  address=$(graphql_output '.data.pendingCreditFacility.wallet.address')
  [[ "$address" == "bt1qaddressmock" ]] || exit 1
}

@test "credit-facility-custody: cannot update manually collateral with a custodian" {
  pending_credit_facility_id=$(read_value 'credit_facility_proposal_id')

  # Get collateral_id from pending credit facility
  variables=$(jq -n --arg id "$pending_credit_facility_id" '{ id: $id }')
  exec_admin_graphql 'find-pending-credit-facility' "$variables"
  collateral_id=$(graphql_output '.data.pendingCreditFacility.collateralId')
  [[ "$collateral_id" != "null" ]] || exit 1

  variables=$(
    jq -n \
      --arg collateral_id "$collateral_id" \
      --arg effective "$(naive_now)" \
    '{
      input: {
        collateralId: $collateral_id,
        collateral: 50000000,
        effective: $effective,
      }
    }'
  )
  exec_admin_graphql 'collateral-update' "$variables"
  errors=$(graphql_output '.errors')
  [[ "$errors" =~ "ManualUpdateError" ]] || exit 1
}

@test "credit-facility-custody: can update collateral by a custodian" {
  pending_credit_facility_id=$(read_value 'credit_facility_proposal_id')

  variables=$(
    jq -n \
      --arg pending_credit_facility_id "$pending_credit_facility_id" \
      '{ id: $pending_credit_facility_id }'
  )
  exec_admin_graphql 'find-credit-facility' "$variables"
  collateral=$(graphql_output '.data.creditFacility.balance.collateral.btcBalance')
  [[ "$collateral" -eq 0 ]] || exit 1

  # external wallet ID 123 is hard coded in mock custodian
  curl -s -X POST --json '{"wallet": "123", "balance": 1000}' http://localhost:5253/webhook/custodian/mock

  retry 30 2 wait_for_collateral "$pending_credit_facility_id"
}
