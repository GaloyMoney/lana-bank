{{
    config(
        unique_key=["interest_accrual_cycle_id", "version"],
    )
}}


with
    source as (
        select s.* from {{ ref("stg_core_interest_accrual_cycle_events_rollup") }} as s
    ),

    transformed as (
        select
            interest_accrual_cycle_id,
            credit_facility_id,
            cast(facility_maturity_date as timestamp) as facility_maturity_date,

            idx,
            parse_timestamp(
                '%Y-%m-%dT%H:%M:%E*SZ', json_value(period, '$.start')
            ) as period_start_at,
            parse_timestamp(
                '%Y-%m-%dT%H:%M:%E*SZ', json_value(period, '$.end')
            ) as period_end_at,
            json_value(period, '$.interval.type') as period_interval_type,

            cast(json_value(terms, '$.annual_rate') as numeric) as annual_rate,
            cast(
                json_value(terms, '$.one_time_fee_rate') as numeric
            ) as one_time_fee_rate,
            cast(json_value(terms, '$.initial_cvl') as numeric) as initial_cvl,
            cast(json_value(terms, '$.liquidation_cvl') as numeric) as liquidation_cvl,
            cast(json_value(terms, '$.margin_call_cvl') as numeric) as margin_call_cvl,
            cast(json_value(terms, '$.duration.value') as integer) as duration_value,
            json_value(terms, '$.duration.type') as duration_type,
            json_value(terms, '$.accrual_interval.type') as accrual_interval,
            json_value(
                terms, '$.accrual_cycle_interval.type'
            ) as accrual_cycle_interval,
            cast(
                json_value(
                    terms, '$.interest_due_duration_from_accrual.value'
                ) as integer
            ) as interest_due_duration_from_accrual_value,
            json_value(
                terms, '$.interest_due_duration_from_accrual.type'
            ) as interest_due_duration_from_accrual_type,
            cast(
                json_value(
                    terms, '$.obligation_overdue_duration_from_due.value'
                ) as integer
            ) as obligation_overdue_duration_from_due_value,
            json_value(
                terms, '$.obligation_overdue_duration_from_due.type'
            ) as obligation_overdue_duration_from_due_type,
            cast(
                json_value(
                    terms, '$.obligation_liquidation_duration_from_due.value'
                ) as integer
            ) as obligation_liquidation_duration_from_due_value,
            json_value(
                terms, '$.obligation_liquidation_duration_from_due.type'
            ) as obligation_liquidation_duration_from_due_type,

            cast(accrued_at as timestamp) as last_accrued_at,
            cast(amount as numeric)
            / {{ var("cents_per_usd") }} as last_accrued_interest_amount_usd,

            cast(effective as timestamp) as posted_effective,
            cast(total as numeric)
            / {{ var("cents_per_usd") }} as posted_total_interest_usd,
            obligation_id as posted_obligation_id,
            is_interest_accruals_posted,

            created_at as interest_accrual_cycle_created_at,
            modified_at as interest_accrual_cycle_modified_at,

            * except (
                interest_accrual_cycle_id,
                credit_facility_id,
                obligation_id,
                facility_maturity_date,
                idx,
                period,
                terms,
                accrued_at,
                amount,
                effective,
                total,
                is_interest_accruals_posted,
                created_at,
                modified_at
            )
        from source
    )

select *
from transformed
