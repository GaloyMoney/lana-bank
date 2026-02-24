load "helpers"

setup_file() {
  start_server
  login_superadmin
  login_lanacli
}

teardown_file() {
  stop_server
}

@test "terms-template: can create" {
  template_name="Test Template $(date +%s)"

  local cli_output
  cli_output=$("$LANACLI" --json terms-template create \
    --name "$template_name" \
    --annual-rate 5.5 \
    --accrual-interval END_OF_DAY \
    --accrual-cycle-interval END_OF_MONTH \
    --one-time-fee-rate 5 \
    --disbursal-policy SINGLE_DISBURSAL \
    --duration-months 12 \
    --initial-cvl 100 \
    --margin-call-cvl 90 \
    --liquidation-cvl 80 \
    --interest-due-days 0 \
    --overdue-days 50 \
    --liquidation-days 60)

  terms_template_id=$(echo "$cli_output" | jq -r '.termsId')
  [[ "$terms_template_id" != "null" && -n "$terms_template_id" ]] || exit 1

  cache_value 'terms_template_id' "$terms_template_id"
}

@test "terms-template: can update" {
  terms_template_id=$(read_value 'terms_template_id')

  variables=$(
    jq -n \
    --arg id "$terms_template_id" \
    '{
      input: {
        id: $id,
        annualRate: 6.5,
        accrualCycleInterval: "END_OF_MONTH",
        accrualInterval: "END_OF_DAY",
        oneTimeFeeRate: "5",
        disbursalPolicy: "SINGLE_DISBURSAL",
        duration: {
          period: "MONTHS",
          units: 24
        },
        interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
        obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
        obligationLiquidationDurationFromDue: { period: "DAYS", units: 60 },
        liquidationCvl: 75,
        marginCallCvl: 85,
        initialCvl: 95
      }
    }'
  )

  exec_admin_graphql 'terms-template-update' "$variables"

  updated_id=$(graphql_output '.data.termsTemplateUpdate.termsTemplate.termsId')
  [[ "$updated_id" == "$terms_template_id" ]] || exit 1

  annual_rate=$(graphql_output '.data.termsTemplateUpdate.termsTemplate.values.annualRate')
  [[ "$annual_rate" == "6.5" ]] || exit 1
}

@test "terms-template: can retrieve" {
  terms_template_id=$(read_value 'terms_template_id')

  variables=$(
    jq -n \
    --arg id "$terms_template_id" \
    '{
      id: $id
    }'
  )

  exec_admin_graphql 'terms-template-get' "$variables"

  retrieved_id=$(graphql_output '.data.termsTemplate.termsId')
  [[ "$retrieved_id" == "$terms_template_id" ]] || exit 1

  annual_rate=$(graphql_output '.data.termsTemplate.values.annualRate')
  [[ "$annual_rate" == "6.5" ]] || exit 1

  duration_units=$(graphql_output '.data.termsTemplate.values.duration.units')
  [[ "$duration_units" == "24" ]] || exit 1
}
