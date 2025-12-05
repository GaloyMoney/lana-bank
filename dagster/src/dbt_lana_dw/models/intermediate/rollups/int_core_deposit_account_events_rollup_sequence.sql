with source as (
    select
        s.*
    from {{ ref('stg_core_deposit_account_events_rollup') }} as s
)


, transformed as (
    select
        deposit_account_id,
        account_holder_id as customer_id,
        created_at as deposit_account_created_at,
        modified_at as deposit_account_modified_at,

        * except(
            deposit_account_id,
            account_holder_id,
            created_at,
            modified_at
        )
    from source
)


select * from transformed
