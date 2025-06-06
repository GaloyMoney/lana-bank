pg_event() {
  echo $event | jq -r "$@"
}

@test "credit-facility-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_credit_facility_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".customer_id"
      ".amount"
      ".terms.annual_rate"
      ".terms.initial_cvl"
      ".terms.liquidation_cvl"
      ".terms.margin_call_cvl"
      ".terms.one_time_fee_rate"
      ".terms.duration.type"
      ".terms.duration.value"
      ".terms.accrual_interval.type"
      ".terms.accrual_cycle_interval.type"
      ".terms.interest_due_from_accrual_after_days.type"
      ".terms.interest_due_from_accrual_after_days.value"
      # ".terms.interest_overdue_duration.type"
      # ".terms.interest_overdue_duration.value"
      ".account_ids.facility_account_id"
      ".account_ids.collateral_account_id"
      ".account_ids.fee_income_account_id"
      ".account_ids.interest_income_account_id"
      ".account_ids.interest_defaulted_account_id"
      ".account_ids.disbursed_defaulted_account_id"
      ".account_ids.interest_receivable_due_account_id"
      ".account_ids.disbursed_receivable_due_account_id"
      ".account_ids.interest_receivable_overdue_account_id"
      ".account_ids.disbursed_receivable_overdue_account_id"
      ".account_ids.interest_receivable_not_yet_due_account_id"
      ".account_ids.disbursed_receivable_not_yet_due_account_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # approval_process_concluded, expecting:
    #
    event_type="approval_process_concluded"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".approved"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # activated, expecting:
    #
    event_type="activated"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".activated_at"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # collateralization_state_changed, expecting:
    #
    event_type="collateralization_state_changed"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".collateral"
      ".price"
      ".state"
      ".outstanding.interest"
      ".outstanding.disbursed"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # collateralization_ratio_changed, expecting:
    #
    event_type="collateralization_ratio_changed"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # interest_accrual_cycle_started, expecting:
    #
    event_type="interest_accrual_cycle_started"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".idx"
      ".interest_accrual_id"
      ".period.start"
      ".period.end"
      ".period.interval.type"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # interest_accrual_cycle_concluded, expecting:
    #
    event_type="interest_accrual_cycle_concluded"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".idx"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "collateral-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_collateral_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".credit_facility_id"
      ".account_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done


    #
    # updated, expecting:
    #
    event_type="updated"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".action"
      ".abs_diff"
      ".new_value"
      ".ledger_tx_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "disbursal-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_disbursal_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".facility_id"
      ".amount"
      ".account_ids.facility_account_id"
      ".account_ids.collateral_account_id"
      ".account_ids.fee_income_account_id"
      ".account_ids.interest_income_account_id"
      ".account_ids.interest_defaulted_account_id"
      ".account_ids.disbursed_defaulted_account_id"
      ".account_ids.interest_receivable_due_account_id"
      ".account_ids.disbursed_receivable_due_account_id"
      ".account_ids.interest_receivable_overdue_account_id"
      ".account_ids.disbursed_receivable_overdue_account_id"
      ".account_ids.interest_receivable_not_yet_due_account_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # approval_process_concluded, expecting:
    #
    event_type="approval_process_concluded"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".approved"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # settled, expecting:
    #
    event_type="settled"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".amount"
      ".ledger_tx_id"
      ".obligation_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "interest-accrual-cycle-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_interest_accrual_cycle_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".facility_id"
      ".facility_matures_at"
      ".idx"
      ".terms.annual_rate"
      ".terms.initial_cvl"
      ".terms.liquidation_cvl"
      ".terms.margin_call_cvl"
      ".terms.one_time_fee_rate"
      ".terms.duration.type"
      ".terms.duration.value"
      ".terms.accrual_interval.type"
      ".terms.accrual_cycle_interval.type"
      ".terms.interest_due_from_accrual_after_days.type"
      ".terms.interest_due_from_accrual_after_days.value"
      # ".terms.interest_overdue_duration.type"
      # ".terms.interest_overdue_duration.value"
      ".period.start"
      ".period.end"
      ".period.interval.type"
      ".account_ids.interest_income_account_id"
      ".account_ids.interest_defaulted_account_id"
      ".account_ids.interest_receivable_due_account_id"
      ".account_ids.interest_receivable_overdue_account_id"
      ".account_ids.interest_receivable_not_yet_due_account_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # interest_accruals_posted, expecting:
    #
    event_type="interest_accruals_posted"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".total"
      ".tx_id"
      ".obligation_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # interest_accrued, expecting:
    #
    event_type="interest_accrued"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".amount"
      ".accrued_at"
      ".tx_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "payment-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_payment_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".credit_facility_id"
      ".amount"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # payment_allocated, expecting:
    #
    event_type="payment_allocated"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".interest"
      ".disbursal"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "obligation-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_obligation_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".credit_facility_id"
      ".obligation_type"
      ".amount"
      ".due_date"
      ".overdue_date"
      # ".defaulted_date"
      ".tx_id"
      ".defaulted_account_id"
      ".due_accounts.receivable_account_id"
      ".due_accounts.account_to_be_credited_id"
      ".overdue_accounts.receivable_account_id"
      ".overdue_accounts.account_to_be_credited_id"
      ".not_yet_due_accounts.receivable_account_id"
      ".not_yet_due_accounts.account_to_be_credited_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # due_recorded, expecting:
    #
    event_type="due_recorded"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".amount"
      ".tx_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    # #
    # # overdue_recorded, expecting:
    # #
    # event_type="overdue_recorded"
    # event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    # [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    # declare -a fields=(
    #   ".amount"
    #   ".tx_id"
    # )
    # for field in "${fields[@]}"
    # do
    #   event_field=$(pg_event ${field})
    #   [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
    #   echo $event_field
    # done

    #
    # payment_allocated, expecting:
    #
    event_type="payment_allocated"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".amount"
      ".tx_id"
      ".payment_id"
      ".payment_allocation_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # completed, expecting:
    #
    event_type="completed"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "deposit-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_deposit_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".id"
      ".deposit_account_id"
      ".ledger_transaction_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}


@test "withdrawal-events: all events/fields exist" {

    #
    # initialized, expecting:
    #
    table="core_withdrawal_events"
    event_type="initialized"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
      ".id"
      ".amount"
      ".reference"
      ".deposit_account_id"
      ".approval_process_id"
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # confirmed, expecting:
    #
    event_type="confirmed"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # approval_process_concluded, expecting:
    #
    event_type="approval_process_concluded"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done

    #
    # cancelled, expecting:
    #
    event_type="cancelled"
    event=$(psql $DATABASE_URL -P format=unaligned -P tuples_only -c "select event from ${table} where event_type = '${event_type}' order by id, sequence limit 1")
    [[ "$event" != "" ]] || (echo "Missing ${table} -> ${event_type}" && exit 1)
    declare -a fields=(
    )
    for field in "${fields[@]}"
    do
      event_field=$(pg_event ${field})
      [[ "$event_field" != "null" ]] || (echo "Missing ${table} -> ${event_type} -> ${field}" && exit 1)
      echo $event_field
    done
}
