select
    left({{ ident('num_referencia') }}, 20) as {{ ident('num_referencia') }},
    left({{ ident('cod_cartera') }}, 2) as cod_cartera,
    left({{ ident('cod_activo') }}, 2) as cod_activo,
    left({{ ident('identificacion_garantia') }}, 20) as identificacion_garantia,
    left({{ ident('tipo_garantia') }}, 2) as tipo_garantia,
    format(
        '%.2f', round({{ ident('valor_garantia_proporcional') }}, 2)
    ) as valor_garantia_proporcional

from {{ ref("int_nrp_41_03_referencia_garantia") }}
