{{ config(tags=["asof"]) }}

with
    chart as (select * from {{ ref("int_core_chart_of_accounts") }}),

    balances as (
        select * from {{ ref("int_account_sets_expanded_with_balances_asof") }}
    ),

    final as (
        select
            code,
            dotted_code,
            spaced_code,
            node_name,
            account_set_id,
            coalesce(sum(balance), 0) as balance
        from chart
        left join balances using (account_set_id)
        group by code, dotted_code, spaced_code, node_name, account_set_id
    )

select *
from final
