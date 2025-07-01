with

deposit_balances as (
    select *
    from
        {{ ref('int_deposit_balances') }}
)
,

deposit_accounts as (
    select *
    from
        {{ ref('int_core_deposit_account_events_rollup') }}
)
,

customers as (
    select *
    from
        {{ ref('int_core_customer_events_rollup') }}
)
,

final as (

    select *
    from deposit_balances
    left join deposit_accounts using (deposit_account_id)
    left join customers using (customer_id)
)


select
    left(replace(customer_id, '-', ''), 14) as `NIU`,
    left(replace(upper(deposit_account_id), '-', ''), 20) as `Número de cuenta`,
from
    final
