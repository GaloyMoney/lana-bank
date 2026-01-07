with

    columns as (
        select
            100 as column_order_by,
            'Capital Social' as column_title,
            'Share Capital' as eng_column_title
        union all
        select 200, 'Reservas de Capital', 'Capital Reserves'
        union all
        select 300, 'Otras Reservas', 'Other Reserves'
        union all
        select 400, 'Resultados por Aplicar', 'Results by Apply'
        union all
        select 500, 'Utilidades no Distribuibles', 'Non-Distributable Profits'
        union all
        select 600, 'Donaciones', 'Donations'
        union all
        select
            700,
            'Otro Resultado Integral Ejercicios Anteriores',
            'Other Comprehensive Income from Previous Financial Years'
        union all
        select
            800,
            'Otro Resultado Integral del Ejercicio',
            'Other Comprehensive Income for the Year'
        union all
        select
            900,
            'Participaciones accionistas no controladores',
            "Non-controlling shareholders' shares"
        union all
        select 1000, 'Patrimonio Total', 'Total Equity'
    )

select *
from columns
order by column_order_by
