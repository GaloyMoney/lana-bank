with source as (
    select
        s.*
    from {{ ref('stg_core_deposit_events_rollup') }} as s
)


, transformed as (
    select
        deposit_id,
        deposit_account_id,

        cast(amount as numeric) / 100 as amount_usd,
        created_at as deposit_created_at,
        modified_at as deposit_modified_at,

        * except(
            deposit_id,
            deposit_account_id,
            amount,
            created_at,
            modified_at
        )
    from source
)


select * from transformed
