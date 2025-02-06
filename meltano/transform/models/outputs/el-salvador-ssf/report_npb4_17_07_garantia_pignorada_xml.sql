{{ config(materialized='table') }}

select
    `tipo_deposito`,
    `cod_banco`,
    left(`identificacion_garantia`, 20) as `identificacion_garantia`,
    left(replace(nit_depositante, '-', ''), 14) as `nit_depositante`,
    format_date('%Y-%m-%d', cast(`fecha_deposito` as date)) as `fecha_deposito`,
    format_date('%Y-%m-%d', cast(`fecha_vencimiento` as date)) as `fecha_vencimiento`,
    format('%.2f', round(`valor_deposito`, 2)) as `valor_deposito`

from {{ ref('int_npb4_17_07_garantia_pignorada_xml_raw') }}
