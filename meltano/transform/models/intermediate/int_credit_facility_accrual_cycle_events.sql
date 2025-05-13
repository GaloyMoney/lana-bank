with accrual_cycle_started as (
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

select * from accrual_cycles
