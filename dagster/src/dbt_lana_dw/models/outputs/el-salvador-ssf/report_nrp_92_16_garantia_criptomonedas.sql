select
    left(`num_referencia`, 20) as `num_referencia`,
    left(`cod_cartera`, 2) as `cod_cartera`,
    left(`cod_activo`, 2) as `cod_activo`,
    left(`identificacion_garantia`, 20) as `identificacion_garantia`,
    format('%.8f', `num_criptomonedas`) as `num_criptomonedas`,
    format('%.2f', round(`valor_contractual`, 2)) as `valor_contractual`,
    format('%.2f', round(`valor_mercado`, 2)) as `valor_mercado`,
    format_date(
        '%Y-%m-%d', cast(`fecha_valuacion_mercado` as date)
    ) as `fecha_valuacion_mercado`,
    `hora_valuacion_mercado`,
    format('%.2f', round(`tasas_conversion`, 2)) as `tasas_conversion`,
    format('%.2f', round(`monto_garantizado`, 2)) as `monto_garantizado`,
    format_date('%Y-%m-%d', cast(`fecha_otorgamiento` as date)) as `fecha_otorgamiento`

from {{ ref("int_nrp_92_16_garantia_criptomonedas") }}
