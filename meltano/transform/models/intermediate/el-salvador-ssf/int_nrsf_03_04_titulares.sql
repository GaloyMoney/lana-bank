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
        {{ ref('int_deposit_accounts') }}
)
,

customers as (
    select *
    from
        {{ ref('int_customers') }}
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
    left(replace(upper(deposit_account_id), '-', ''), 20) as `Número de cuenta`
from
    final
