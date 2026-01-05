with

    approved_withdrawals as (
        select
            withdrawal_id,
            amount_usd,
            deposit_account_id,
            withdrawal_modified_at as recorded_at
        from {{ ref("int_core_withdrawal_events_rollup") }}
        where approved
    )

select *
from approved_withdrawals
