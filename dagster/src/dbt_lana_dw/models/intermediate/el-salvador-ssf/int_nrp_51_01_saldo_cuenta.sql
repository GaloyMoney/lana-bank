with

    latest_available_as_of as (
        select max(as_of_date) as as_of_date
        from {{ ref("int_nrp_51_01_saldo_cuenta_daily") }}
        where as_of_date <= current_date("UTC")
    ),

    daily as (
        select *
        from {{ ref("int_nrp_51_01_saldo_cuenta_daily") }}
        where as_of_date = (select as_of_date from latest_available_as_of)
    ),

    final as (select id_codigo_cuenta, nom_cuenta, valor from daily)

select *
from final
order by id_codigo_cuenta
