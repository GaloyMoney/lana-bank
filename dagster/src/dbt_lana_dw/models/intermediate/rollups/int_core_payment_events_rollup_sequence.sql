{{ config(
    unique_key = ['payment_id', 'version'],
) }}


with source as (
    select s.*
    from {{ ref('stg_core_payment_events_rollup') }} as s
),

transformed as (
    select
        payment_id,
        credit_facility_id,
        cast(amount as numeric) / {{ var('cents_per_usd') }} as amount_usd,
        cast(0 as numeric) / {{ var('cents_per_usd') }} as interest_usd,
        cast(0 as numeric) / {{ var('cents_per_usd') }} as disbursal_usd,
        created_at as payment_created_at,
        modified_at as payment_modified_at,

        * except (
            payment_id,
            credit_facility_id,
            amount,
            created_at,
            modified_at
        )
    from source
)

select * from transformed
