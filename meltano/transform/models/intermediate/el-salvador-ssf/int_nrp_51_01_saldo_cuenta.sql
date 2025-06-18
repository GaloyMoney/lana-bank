with

chart as (
    select *
    from {{ ref('int_core_chart_of_account_with_balances') }}
)

, final as (
    select
        dotted_code as id_codigo_cuenta,
        name as nom_cuenta,
        balance as valor

    from chart
)

select * from final
