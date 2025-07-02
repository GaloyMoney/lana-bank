with interest_accrual_cycle as (
    select
        id as interest_accrual_cycle_id,
        credit_facility_id,
        obligation_id,
        cast(facility_matures_at as timestamp) as facility_matures_at,
        idx,
        cast(effective as timestamp) as effective,
        cast(accrued_at as timestamp) as accrued_at,

        cast(json_value(period, "$.start") as timestamp) as period_start_at,
        cast(json_value(period, "$.end") as timestamp) as period_end_at,
        json_value(period, "$.interval.type") as period_interval_type,


        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(total as numeric) / {{ var('cents_per_usd') }} as total_usd,
        is_interest_accruals_posted,

        cast(json_value(terms, "$.annual_rate") as numeric) as annual_rate,
        cast(json_value(terms, "$.one_time_fee_rate") as numeric) as one_time_fee_rate,
        cast(json_value(terms, "$.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(terms, "$.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(terms, "$.margin_call_cvl") as numeric) as margin_call_cvl,
        cast(json_value(terms, "$.duration.value") as integer) as duration_value,
        json_value(terms, "$.duration.type") as duration_type,
        json_value(terms, "$.accrual_interval.type") as accrual_interval,
        json_value(terms, "$.accrual_cycle_interval.type") as accrual_cycle_interval,
        cast(json_value(terms, "$.interest_due_duration_from_accrual.value") as integer) as interest_due_duration_from_accrual_value,
        json_value(terms, "$.interest_due_duration_from_accrual.type") as interest_due_duration_from_accrual_type,
        cast(json_value(terms, "$.obligation_overdue_duration_from_due.value") as integer) as obligation_overdue_duration_from_due_value,
        json_value(terms, "$.obligation_overdue_duration_from_due.type") as obligation_overdue_duration_from_due_type,
        cast(json_value(terms, "$.obligation_liquidation_duration_from_due.value") as integer) as obligation_liquidation_duration_from_due_value,
        json_value(terms, "$.obligation_liquidation_duration_from_due.type") as obligation_liquidation_duration_from_due_type,
        created_at as interest_accrual_cycle_created_at,
        modified_at as interest_accrual_cycle_modified_at,

        * except(
            id,
            credit_facility_id,
            obligation_id,
            facility_matures_at,
            idx,
            period,
            terms,
            accrued_at,
            amount,
            effective,
            total,
            is_interest_accruals_posted,
            created_at,
            modified_at,

            last_sequence,
            _sdc_received_at,
            _sdc_batched_at,
            _sdc_extracted_at,
            _sdc_deleted_at,
            _sdc_sequence,
            _sdc_table_version
        )
    from {{ ref('stg_core_interest_accrual_cycle_events_rollup') }}
)


select * from interest_accrual_cycle
