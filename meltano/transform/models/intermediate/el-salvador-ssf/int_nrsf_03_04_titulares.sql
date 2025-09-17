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
    customer.id as `NIU`,
    deposit_account.id as `Número de cuenta`,
from
    final
left join
    {{ ref('stg_core_public_ids') }} as customer on customer_id = customer.target_id
left join
    {{ ref('stg_core_public_ids') }} as deposit_account on deposit_account_id = deposit_account.target_id
