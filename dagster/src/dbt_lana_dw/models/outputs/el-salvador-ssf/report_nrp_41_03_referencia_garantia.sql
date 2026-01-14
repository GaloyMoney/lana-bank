select
    left(`num_referencia`, 20) as `num_referencia`,
    left(`cod_cartera`, 2) as cod_cartera,
    left(`cod_activo`, 2) as cod_activo,
    left(`identificacion_garantia`, 20) as identificacion_garantia,
    left(`tipo_garantia`, 2) as tipo_garantia,
    format(
        '%.2f', round(`valor_garantia_proporcional`, 2)
    ) as valor_garantia_proporcional

from {{ ref("int_nrp_41_03_referencia_garantia") }}
