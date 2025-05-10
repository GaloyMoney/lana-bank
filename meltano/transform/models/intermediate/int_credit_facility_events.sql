with initialized as (
    select
        id as credit_facility_id,
        recorded_at as initialized_recorded_at,

        json_value(event, "$.customer_id") as customer_id,

        cast(json_value(event, '$.amount') as numeric) as facility_amount,

        cast(json_value(event, "$.terms.annual_rate") as numeric) as annual_rate,

        cast(json_value(event, "$.terms.initial_cvl") as numeric) as initial_cvl,
        cast(json_value(event, "$.terms.liquidation_cvl") as numeric) as liquidation_cvl,
        cast(json_value(event, "$.terms.margin_call_cvl") as numeric) as margin_call_cvl,

        cast(json_value(event, "$.terms.one_time_fee_rate") as numeric) as one_time_fee_rate,

        json_value(event, "$.terms.duration.type") as duration_type,
        cast(json_value(event, "$.terms.duration.value") as integer) as duration_value,

        json_value(event, "$.terms.accrual_interval.type") as accrual_interval,
        json_value(event, "$.terms.accrual_cycle_interval.type") as accrual_cycle_interval,

        json_value(event, "$.terms.interest_due_duration.type") as interest_due_duration_type,
        cast(json_value(event, "$.terms.interest_due_duration.value") as integer) as interest_due_duration_value,

        json_value(event, "$.terms.interest_overdue_duration.type") as interest_overdue_duration_type,
        cast(json_value(event, "$.terms.interest_overdue_duration.value") as integer) as interest_overdue_duration_value,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = 'initialized'
)

, concluded as (
    select
        id as credit_facility_id,
        recorded_at as concluded_recorded_at,
        cast(json_value(event, '$.approved') as boolean) as approved,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "approval_process_concluded"
)

, collateral_state_changed as (
    select
        id as credit_facility_id,
        max(recorded_at) as collateral_state_changed_recorded_at,
        cast(json_value(any_value(event having max recorded_at), '$.collateral') as numeric) as collateral,
        cast(json_value(any_value(event having max recorded_at), '$.price') as numeric) as price,
        json_value(any_value(event having max recorded_at), '$.state') as state,
        cast(json_value(any_value(event having max recorded_at), '$.outstanding.interest') as numeric) as outstanding_interest,
        cast(json_value(any_value(event having max recorded_at), '$.outstanding.disbursed') as numeric) as outstanding_disbursed,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = 'collateralization_state_changed'
    group by credit_facility_id
)

, activated as (
    select
        id as credit_facility_id,
        recorded_at as activated_recorded_at,
        recorded_at as activated_at,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "activated"
)

, accrual_cycle_started as (
    select
        id as credit_facility_id,
        recorded_at as accrual_cycle_started_recorded_at,
        cast(json_value(event, '$.idx') as integer) as idx,
        json_value(event, '$.interest_accrual_id') as interest_accrual_id,
        cast(json_value(event, '$.period.start') as timestamp) as period_start,
        cast(json_value(event, '$.period.end') as timestamp) as period_end,
        json_value(event, '$.period.interval.type') as period_interval_type,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "interest_accrual_cycle_started"
)

, accrual_cycle_concluded as (
    select
        id as credit_facility_id,
        recorded_at as accrual_cycle_concluded_recorded_at,
        cast(json_value(event, '$.idx') as integer) as idx,

    from {{ ref('stg_credit_facility_events') }}
    where event_type = "interest_accrual_cycle_concluded"
)

, accrual_cycles as (
    select
        credit_facility_id,
        array_agg(struct(period_start as period_start, period_end as period_end, period_interval_type as period_interval_type, accrual_cycle_concluded_recorded_at is null as concluded)) as accrual_cycles
    from accrual_cycle_started
    left join accrual_cycle_concluded using (credit_facility_id, idx)
    group by credit_facility_id
)

, final as (
    select
        *,
        case when duration_type = 'months' then timestamp_add(date(activated_at), interval duration_value month) end as maturity_at,
    from initialized
    left join concluded using (credit_facility_id)
    left join collateral_state_changed using (credit_facility_id)
    left join activated using (credit_facility_id)
    left join accrual_cycles using (credit_facility_id)
)


select * from final
