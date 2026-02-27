{{
    config(
        materialized="table",
        partition_by={"field": "as_of_date", "data_type": "date"},
        cluster_by=["id_codigo_cuenta"],
    )
}}

with

    chart as (select * from {{ ref("int_core_chart_of_account_with_balances_daily") }}),

    final as (
        select
            as_of_date,
            code as id_codigo_cuenta,
            node_name as nom_cuenta,
            balance as valor
        from chart
    )

select *
from final
