{{ config(materialized='table') }}

select
    left(`id_codigo_deuda`, 10) as `id_codigo_deuda`,
    left(`desc_deuda`, 80) as `desc_deuda`,
    cast(round(`valor_deuda`, 2) as string) as `valor_deuda`
from
    {{ ref('int_nrp_51_07_deuda_subordinada') }}
