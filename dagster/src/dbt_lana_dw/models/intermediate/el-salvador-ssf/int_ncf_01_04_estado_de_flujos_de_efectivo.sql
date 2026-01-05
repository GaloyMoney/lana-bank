with

    config as (select * from {{ ref("static_ncf_01_04_account_config") }}),

    chart as (select * from {{ ref("int_core_chart_of_account_with_balances") }}),

    final as (
        select order_by, title, sum(coalesce(balance, 0)) as balance
        from config
        left join chart on code in unnest(source_account_codes)
        group by order_by, title
    )

select title, balance
from final
order by order_by
