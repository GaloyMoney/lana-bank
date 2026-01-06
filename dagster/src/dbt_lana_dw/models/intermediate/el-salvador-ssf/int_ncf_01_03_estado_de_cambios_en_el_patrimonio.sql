with

    config as (select * from {{ ref("static_ncf_01_03_account_config") }}),

    chart as (select * from {{ ref("int_core_chart_of_account_with_balances") }}),

    joined as (
        select order_by, title, column_title, sum(coalesce(balance, 0)) as balance
        from config
        left join chart on code in unnest(source_account_codes)
        group by order_by, title, column_title
    ),

    final as (
        select *
        from
            (select order_by, title, column_title, balance from joined) pivot (
                sum(balance)
                for column_title in (
                    'Capital Social',
                    'Reservas de Capital',
                    'Otras Reservas',
                    'Resultados por Aplicar',
                    'Utilidades no Distribuibles',
                    'Donaciones',
                    'Otro Resultado Integral Ejercicios Anteriores',
                    'Otro Resultado Integral del Ejercicio',
                    'Participaciones accionistas no controladores',
                    'Patrimonio Total'
                )
            )
    )

select * except (order_by)
from final
order by order_by
