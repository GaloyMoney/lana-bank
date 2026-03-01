load "helpers"

setup_file() {
  start_server
  login_superadmin
}

teardown_file() {
  stop_server
}

@test "terms-template: can create" {
  template_name="Test Template $(date +%s)"

  variables=$(
    jq -n \
    --arg name "$template_name" \
    '{
      input: {
        name: $name,
        annualRate: 5.5,
        accrualCycleInterval: "END_OF_MONTH",
        accrualInterval: "END_OF_DAY",
        oneTimeFeeRate: "5",
        disbursalPolicy: "SINGLE_DISBURSAL",
        duration: {
          period: "MONTHS",
          units: 12
        },
        interestDueDurationFromAccrual: { period: "DAYS", units: 0 },
        obligationOverdueDurationFromDue: { period: "DAYS", units: 50 },
        obligationLiquidationDurationFromDue: { period: "DAYS", units: 60 },
        liquidationCvl: 80,
        marginCallCvl: 90,
        initialCvl: 100
      }
    }'
  )

  exec_admin_graphql 'terms-template-create' "$variables"

  terms_template_id=$(graphql_output '.data.termsTemplateCreate.termsTemplate.termsTemplateId')
  [[ "$terms_template_id" != "null" ]] || exit 1

  cache_value 'terms_template_id' "$terms_template_id"
}

@test "terms-template: can update" {
  terms_template_id=$(read_value 'terms_template_id')

  variables=$(
    jq -n \
    --arg termsTemplateId "$terms_template_id" \
    '{
      input: {
        termsTemplateId: $termsTemplateId,
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

  updated_id=$(graphql_output '.data.termsTemplateUpdate.termsTemplate.termsTemplateId')
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

  retrieved_id=$(graphql_output '.data.termsTemplate.termsTemplateId')
  [[ "$retrieved_id" == "$terms_template_id" ]] || exit 1

  annual_rate=$(graphql_output '.data.termsTemplate.values.annualRate')
  [[ "$annual_rate" == "6.5" ]] || exit 1

  duration_units=$(graphql_output '.data.termsTemplate.values.duration.units')
  [[ "$duration_units" == "24" ]] || exit 1
}
