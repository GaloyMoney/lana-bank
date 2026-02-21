select
    {{ ident('tipo_deposito') }},
    {{ ident('cod_banco') }},
    left({{ ident('identificacion_garantia') }}, 20) as {{ ident('identificacion_garantia') }},
    left(replace(nit_depositante, '-', ''), 14) as {{ ident('nit_depositante') }},
    format_date('%Y-%m-%d', cast({{ ident('fecha_deposito') }} as date)) as {{ ident('fecha_deposito') }},
    format_date('%Y-%m-%d', cast({{ ident('fecha_vencimiento') }} as date)) as {{ ident('fecha_vencimiento') }},
    format('%.2f', round({{ ident('valor_deposito') }}, 2)) as {{ ident('valor_deposito') }}

from {{ ref("int_nrp_41_07_garantia_pignorada") }}
