{{ config(materialized='table') }}

with payment_recorded as (

    select
        id as event_id,
        cast(format_date('%Y%m%d', recorded_at) as int64)
            as recorded_at_date_key,
        recorded_at,
        event_type,
        cast(json_value(event, '$.disbursal_amount') as numeric)
            as disbursal_amount,
        cast(json_value(event, '$.interest_amount') as numeric)
            as interest_amount,
        coalesce(cast(
            format_date(
                '%Y%m%d',
                parse_timestamp(
                    '%Y-%m-%dT%H:%M:%E*SZ',
                    json_value(event, '$.recorded_in_ledger_at'),
                    'UTC'
                )
            ) as int64
        ), 19000101) as recorded_in_ledger_at_date_key,
        parse_timestamp(
            '%Y-%m-%dT%H:%M:%E*SZ',
            json_value(event, '$.recorded_in_ledger_at'),
            'UTC'
        ) as recorded_in_ledger_at
    from {{ ref('stg_credit_facility_events') }} as cfe
    where
        cfe.event_type = 'payment_recorded'
        and json_value(event, '$.tx_id') is not null

)

select *
from payment_recorded
