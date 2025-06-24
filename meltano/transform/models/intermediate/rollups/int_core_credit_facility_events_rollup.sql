with credit_facility as (
    select
        id as credit_facility_id,
        customer_id,

        amount as facility_amount_usd,
        cast(json_value(terms, "$.annual_rate") as numeric) as annual_rate,
        cast(json_value(terms, "$.one_time_fee_rate") as numeric) as one_time_fee_rate,

        cast(json_value(terms, "$.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(terms, "$.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(terms, "$.margin_call_cvl") as numeric) as margin_call_cvl,

        cast(json_value(terms, "$.duration.value") as integer) as duration_value,
        json_value(terms, "$.duration.type") as duration_type,

        json_value(terms, "$.accrual_interval.type") as accrual_interval,
        json_value(terms, "$.accrual_cycle_interval.type") as accrual_cycle_interval,

        collateral,
        price,
        collateralization_ratio,
        collateralization_state,

        approved,

        is_approval_process_concluded,
        is_activated,
        activated_at,
        is_completed,

        interest_accrual_cycle_idx,
        cast(json_value(interest_period, "$.start") as timestamp) as interest_period_start_at,
        cast(json_value(interest_period, "$.end") as timestamp) as interest_period_end_at,
        json_value(interest_period, "$.interval.type") as interest_period_interval_type,

        cast(json_value(outstanding, "$.interest") as numeric) as outstanding_interest,
        cast(json_value(outstanding, "$.disbursed") as numeric) as outstanding_disbursed,

        cast(json_value(terms, "$.interest_due_duration_from_accrual.value") as integer) as interest_due_duration_from_accrual_value,
        json_value(terms, "$.interest_due_duration_from_accrual.type") as interest_due_duration_from_accrual_type,

        cast(json_value(terms, "$.obligation_overdue_duration_from_due.value") as integer) as obligation_overdue_duration_from_due_value,
        json_value(terms, "$.obligation_overdue_duration_from_due.type") as obligation_overdue_duration_from_due_type,

        cast(json_value(terms, "$.obligation_liquidation_duration_from_due.value") as integer) as obligation_liquidation_duration_from_due_value,
        json_value(terms, "$.obligation_liquidation_duration_from_due.type") as obligation_liquidation_duration_from_due_type,

        * except(
            id,
            customer_id,
            amount,
            last_sequence,
            created_at,
            modified_at,
            ledger_tx_ids,
            account_ids,
            terms,
            collateral,
            price,
            collateralization_ratio,
            collateralization_state,
            approved,
            is_approval_process_concluded,
            is_activated,
            activated_at,
            is_completed,
            interest_accrual_cycle_idx,
            interest_period,
            outstanding,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_credit_facility_events_rollup') }}
    where approved
)


select * from credit_facility
