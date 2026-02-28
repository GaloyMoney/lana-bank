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

  local cli_output
  cli_output=$("$LANACLI" --json terms-template update \
    --id "$terms_template_id" \
    --annual-rate 6.5 \
    --accrual-interval END_OF_DAY \
    --accrual-cycle-interval END_OF_MONTH \
    --one-time-fee-rate 5 \
    --disbursal-policy SINGLE_DISBURSAL \
    --duration-months 24 \
    --initial-cvl 95 \
    --margin-call-cvl 85 \
    --liquidation-cvl 75 \
    --interest-due-days 0 \
    --overdue-days 50 \
    --liquidation-days 60)

  updated_id=$(echo "$cli_output" | jq -r '.termsId')
  [[ "$updated_id" == "$terms_template_id" ]] || exit 1

  annual_rate=$(echo "$cli_output" | jq -r '.values.annualRate')
  [[ "$annual_rate" == "6.5" ]] || exit 1
}

@test "terms-template: can retrieve" {
  terms_template_id=$(read_value 'terms_template_id')

  local cli_output
  cli_output=$("$LANACLI" --json terms-template get --id "$terms_template_id")

  retrieved_id=$(echo "$cli_output" | jq -r '.termsId')
  [[ "$retrieved_id" == "$terms_template_id" ]] || exit 1

  annual_rate=$(echo "$cli_output" | jq -r '.values.annualRate')
  [[ "$annual_rate" == "6.5" ]] || exit 1

  duration_units=$(echo "$cli_output" | jq -r '.values.duration.units')
  [[ "$duration_units" == "24" ]] || exit 1
}
