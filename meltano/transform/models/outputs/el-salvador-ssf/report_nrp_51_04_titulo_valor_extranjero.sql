{{ config(materialized='table') }}

select
    left(`id_codigo_titulo_extranjero`, 10) as `id_codigo_titulo_extranjero`,
    left(`desc_tv_extranj`, 254) as `desc_tv_extranj`,
    cast(round(`valor_tv_extranj`, 2) as string) as `valor_tv_extranj`
from
    {{ ref('int_nrp_51_04_titulo_valor_extranjero') }}
