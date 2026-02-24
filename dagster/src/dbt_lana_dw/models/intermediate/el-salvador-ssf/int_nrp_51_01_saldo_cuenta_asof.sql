{{ config(tags=["asof"]) }}

with

    chart as (select * from {{ ref("int_core_chart_of_account_with_balances_asof") }}),

    final as (
        select code as id_codigo_cuenta, node_name as nom_cuenta, balance as valor

        from chart
    )

select *
from final
order by id_codigo_cuenta
