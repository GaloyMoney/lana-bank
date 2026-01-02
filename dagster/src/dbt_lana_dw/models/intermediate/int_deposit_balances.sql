with

    deposits as (
        select
            {# deposit_id, #}
            deposit_account_id, amount_usd, deposit_modified_at as recorded_at
        from {{ ref("int_core_deposit_events_rollup") }}
    ),

    approved_withdrawals as (
        select
            {# withdrawal_id, #}
            deposit_account_id, - amount_usd as amount_usd, recorded_at
        from {{ ref("int_approved_withdrawals") }}
    ),

    unioned as (

        select deposit_account_id, amount_usd, recorded_at
        from deposits

        union all

        select deposit_account_id, amount_usd, recorded_at
        from approved_withdrawals

    ),

    final as (

        select
            deposit_account_id,
            sum(amount_usd) as deposit_account_balance_usd,
            min(recorded_at) as earliest_recorded_at,
            max(recorded_at) as latest_recorded_at
        from unioned
        group by deposit_account_id

    )

select *
from final
