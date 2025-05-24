with initialized as (
    select
        id as interest_accrual_cycle_id,
        recorded_at as initialized_recorded_at,

        json_value(event, "$.facility_id") as credit_facility_id,
        cast(json_value(event, "$.facility_matures_at") as timestamp) as facility_matures_at,

        cast(json_value(event, "$.idx") as integer) as idx,

        cast(json_value(event, "$.terms.annual_rate") as numeric) as annual_rate,
        cast(json_value(event, "$.terms.one_time_fee_rate") as numeric) as one_time_fee_rate,

        cast(json_value(event, "$.terms.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(event, "$.terms.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(event, "$.terms.margin_call_cvl") as numeric) as margin_call_cvl,

        cast(json_value(event, "$.terms.duration.value") as integer) as duration_value,
        json_value(event, "$.terms.duration.type") as duration_type,

        json_value(event, "$.terms.accrual_interval.type") as accrual_interval,
        json_value(event, "$.terms.accrual_cycle_interval.type") as accrual_cycle_interval,

        cast(json_value(event, "$.terms.interest_due_duration.value") as integer) as interest_due_duration_value,
        json_value(event, "$.terms.interest_due_duration.type") as interest_due_duration_type,

        cast(json_value(event, "$.terms.interest_overdue_duration.value") as integer) as interest_overdue_duration_value,
        json_value(event, "$.terms.interest_overdue_duration.type") as interest_overdue_duration_type,

        cast(json_value(event, "$.period.start") as timestamp) as period_start_at,
        cast(json_value(event, "$.period.end") as timestamp) as period_end_at,
        json_value(event, "$.period.interval.type") as period_interval_type,

        json_value(event, "$.account_ids.interest_income_account_id") as interest_income_account_id,
        json_value(event, "$.account_ids.interest_defaulted_account_id") as interest_defaulted_account_id,
        json_value(event, "$.account_ids.interest_receivable_due_account_id") as interest_receivable_due_account_id,
        json_value(event, "$.account_ids.interest_receivable_overdue_account_id") as interest_receivable_overdue_account_id,
        json_value(event, "$.account_ids.interest_receivable_not_yet_due_account_id") as interest_receivable_not_yet_due_account_id,

    from {{ ref('stg_interest_accrual_cycle_events') }}
    where event_type = 'initialized'
)

, interest_accrued as (
    select
        id as interest_accrual_cycle_id,
        recorded_at as recorded_at,
        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as accrued_interest_amount_usd,
        cast(json_value(event, '$.accrued_at') as timestamp) as accrued_at,
        json_value(event, '$.tx_id') as tx_id,

    from {{ ref('stg_interest_accrual_cycle_events') }}
    where event_type = "interest_accrued"
)

, interest_accrued_agg as (
    select
        interest_accrual_cycle_id,
        array_agg(struct(
            accrued_at,
            accrued_interest_amount_usd,
            recorded_at,
            tx_id as tx_id
        ) order by accrued_at ) as interest_accrued,

    from interest_accrued
    group by interest_accrual_cycle_id
)

, interest_accruals_posted as (
    select
        id as interest_accrual_cycle_id,
        recorded_at as interest_accruals_posted_recorded_at,
        cast(json_value(event, '$.total') as numeric) / {{ var('cents_per_usd') }} as total_interest_posted_usd,
        json_value(event, '$.tx_id') as posted_tx_id,
        json_value(event, '$.obligation_id') as posted_obligation_id,

    from {{ ref('stg_interest_accrual_cycle_events') }}
    where event_type = "interest_accruals_posted"

)

, final as (
    select
        *
    from initialized
    left join interest_accruals_posted using (interest_accrual_cycle_id)
    left join interest_accrued_agg using (interest_accrual_cycle_id)
)


select * from final
