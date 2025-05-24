with initialized as (
    select
        id as payment_id,
        recorded_at as initialized_recorded_at,

        json_value(event, "$.credit_facility_id") as credit_facility_id,

        cast(json_value(event, '$.amount') as numeric) / {{ var('cents_per_usd') }} as payment_amount_usd,


    from {{ ref('stg_payment_events') }}
    where event_type = 'initialized'
)

, payment_allocated as (
    select
        id as payment_id,
        recorded_at as payment_allocated_at,
        coalesce(cast(json_value(event, '$.interest') as numeric), 0) / {{ var('cents_per_usd') }} as interest_amount_usd,
        coalesce(cast(json_value(event, '$.disbursal') as numeric), 0) / {{ var('cents_per_usd') }} as disbursal_amount_usd,

    from {{ ref('stg_payment_events') }}
    where event_type = "payment_allocated"
)

, final as (
    select
        *
    from initialized
    left join payment_allocated using (payment_id)
)


select * from final
