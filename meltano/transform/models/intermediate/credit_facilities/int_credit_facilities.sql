{{ config(materialized='table') }}

with initialized as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        event_type,
        cast(json_value(event, '$.terms.annual_rate') as numeric)
            as terms_annual_rate,
        cast(json_value(event, '$.terms.duration.value') as integer)
            as terms_duration_value,
        cast(json_value(event, '$.terms.initial_cvl') as numeric)
            as terms_initial_cvl,
        cast(json_value(event, '$.terms.liquidation_cvl') as numeric)
            as terms_liquidation_cvl,
        cast(json_value(event, '$.terms.margin_call_cvl') as numeric)
            as terms_margin_call_cvl,
        cast(json_value(event, '$.facility') as numeric) as facility,
        json_value(event, '$.customer_id') as customer_id,
        json_value(event, '$.terms.accrual_cycle_interval.type')
            as terms_accrual_cycle_interval_type,
        json_value(event, '$.terms.duration.type') as terms_duration_type,
        json_value(event, '$.terms.accrual_interval.type')
            as terms_accrual_interval_type
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'initialized'

),

approval_process_concluded as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        cast(json_value(event, '$.approved') as boolean) as approved
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'approval_process_concluded'

),

activated as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        cast(
            format_date(
                '%Y%m%d',
                parse_timestamp(
                    '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.activated_at')
                ),
                'UTC'
            ) as int64
        ) as activated_at_date_key,
        parse_timestamp(
            '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.activated_at'), 'UTC'
        ) as activated_at
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'activated'

),

completed as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        cast(
            format_date(
                '%Y%m%d',
                parse_timestamp(
                    '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.completed_at')
                ),
                'UTC'
            ) as int64
        ) as completed_at_date_key,
        parse_timestamp(
            '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.completed_at'), 'UTC'
        ) as completed_at
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'completed'

)

select
    i.* except (facility),

    apc.recorded_at as approval_process_concluded_recorded_at,

    a.recorded_at as activated_recorded_at,
    a.activated_at,
    c.recorded_at as completed_recorded_at,

    c.completed_at,
    i.facility,
    coalesce(apc.recorded_at_date_key, 19000101)
        as approval_process_concluded_recorded_at_date_key,

    coalesce(apc.approved, false) as approval_process_concluded_approved,
    coalesce(a.recorded_at_date_key, 19000101)
        as activated_recorded_at_date_key,
    coalesce(a.activated_at_date_key, 19000101) as activated_at_date_key,
    coalesce(c.recorded_at_date_key, 19000101)
        as completed_recorded_at_date_key,

    coalesce(c.completed_at_date_key, 19000101) as completed_at_date_key
from initialized as i
left join approval_process_concluded as apc on i.event_id = apc.event_id
left join activated as a on i.event_id = a.event_id
left join completed as c on i.event_id = c.event_id
