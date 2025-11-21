with source as (
    select
        s.*
    from {{ ref('stg_core_withdrawal_events_rollup') }} as s
)


, transformed as (
    select
        withdrawal_id,
        deposit_account_id,

        cast(amount as numeric) / 100 as amount_usd,
        approved,
        is_approval_process_concluded,
        is_confirmed,
        is_cancelled,
        created_at as withdrawal_created_at,
        modified_at as withdrawal_modified_at,

        * except(
            withdrawal_id,
            deposit_account_id,
            amount,
            approved,
            is_approval_process_concluded,
            is_confirmed,
            is_cancelled,
            created_at,
            modified_at
        )
    from source
)


select * from transformed
