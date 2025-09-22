{{ config(materialized='table') }}

with disbursal_initiated as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        event_type,
        cast(json_value(event, '$.amount') as numeric) as amount,
        cast(json_value(event, '$.idx') as integer) as idx
    from {{ ref('stg_credit_facility_events') }} as cfe
    where cfe.event_type = 'disbursal_initiated'

),

disbursal_concluded as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        cast(
            format_date(
                '%Y%m%d',
                parse_timestamp(
                    '%Y-%m-%dT%H:%M:%E*SZ',
                    json_value(event, '$.recorded_at'),
                    'UTC'
                )
            ) as int64
        ) as event_recorded_at_date_key,
        cast(json_value(event, '$.idx') as integer) as idx,
        parse_timestamp(
            '%Y-%m-%dT%H:%M:%E*SZ', json_value(event, '$.recorded_at'), 'UTC'
        ) as event_recorded_at
    from {{ ref('stg_credit_facility_events') }} as cfe
    where
        cfe.event_type = 'disbursal_concluded'
        and json_value(event, '$.tx_id') is not null

)

select
    di.* except (amount),

    dc.event_recorded_at as disbursal_concluded_event_recorded_at,
    di.amount,

    coalesce(dc.event_recorded_at_date_key, 19000101)
        as disbursal_concluded_event_recorded_at_date_key
from disbursal_initiated as di
left join
    disbursal_concluded as dc
    on di.event_id = dc.event_id and di.idx = dc.idx
