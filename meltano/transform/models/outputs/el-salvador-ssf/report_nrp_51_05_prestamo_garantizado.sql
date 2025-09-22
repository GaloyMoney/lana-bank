{{ config(materialized='table') }}

select
    cast(round(`valor`, 2) as string) as `valor`,
    left(`id_codigo_banco`, 10) as `id_codigo_banco`,
    left(`nom_banco`, 80) as `nom_banco`,
    left(`Pais`, 20) as `Pais`,
    left(`categoria`, 2) as `categoria`
from
    {{ ref('int_nrp_51_05_prestamo_garantizado') }}
