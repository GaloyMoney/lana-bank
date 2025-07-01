with

deposits as (
    select
        {# deposit_id, #}
        deposit_account_id,
        amount,
        recorded_at
    from {{ ref('int_deposits') }}
)
,

approved_withdrawals as (
    select
        {# withdrawal_id, #}
        deposit_account_id,
        recorded_at,
        -amount_usd as amount
    from {{ ref('int_approved_withdrawals') }}
)
,

unioned as (

    select
        deposit_account_id,
        amount,
        recorded_at
    from deposits

    union all

    select
        deposit_account_id,
        amount,
        recorded_at
    from approved_withdrawals

)
,

final as (

    select
        deposit_account_id,
        sum(amount) as deposit_account_balance,
        min(recorded_at) as earliest_recorded_at,
        max(recorded_at) as latest_recorded_at
    from unioned
    group by deposit_account_id

)

select *
from final
