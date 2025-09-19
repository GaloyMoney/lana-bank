{{ config(materialized='table') }}

select
    cast(round(`valor_aval_fianza`, 2) as string) as `valor_aval_fianza`,
    left(`id_codigo_banco`, 10) as `id_codigo_banco`,
    left(`nom_banco`, 80) as `nom_banco`,
    left(`Pais`, 20) as `Pais`,
    left(`categoria`, 2) as `categoria`
from
    {{ ref('int_nrp_51_06_aval_garantizado') }}
