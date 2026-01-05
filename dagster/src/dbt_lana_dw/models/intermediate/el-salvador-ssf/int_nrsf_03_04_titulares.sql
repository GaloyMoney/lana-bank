with

    deposit_balances as (select * from {{ ref("int_deposit_balances") }}),

    deposit_accounts as (
        select * from {{ ref("int_core_deposit_account_events_rollup") }}
    ),

    customers as (select * from {{ ref("int_core_customer_events_rollup") }}),

    final as (

        select *
        from deposit_balances
        left join deposit_accounts using (deposit_account_id)
        left join customers using (customer_id)
    ),
    ids as (
        select
            final.customer_id,
            final.deposit_account_id,
            customer_public_ids.id as `NIU`,
            deposit_account_public_ids.id as `Número de cuenta`
        from final
        left join
            {{ ref("stg_core_public_ids") }} as customer_public_ids
            on final.customer_id = customer_public_ids.target_id
        left join
            {{ ref("stg_core_public_ids") }} as deposit_account_public_ids
            on final.deposit_account_id = deposit_account_public_ids.target_id
    )

select `NIU`, `Número de cuenta`
from ids
