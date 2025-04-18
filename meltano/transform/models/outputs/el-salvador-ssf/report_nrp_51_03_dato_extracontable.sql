{{ config(materialized='table') }}

select
    cast(round(`Valor`, 2) as string) as `Valor`,
    left(`id_codigo_extracontable`, 10) as `id_codigo_extracontable`,
    left(`desc_extra_contable`, 80) as `desc_extra_contable`
from
    {{ ref('int_nrp_51_03_dato_extracontable') }}
