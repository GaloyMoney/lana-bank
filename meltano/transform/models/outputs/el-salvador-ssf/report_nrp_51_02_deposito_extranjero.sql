{{ config(materialized='table') }}

select
    cast(round(`Valor`, 2) as string) as `Valor`,
    left(`id_codigo_banco`, 10) as `id_codigo_banco`,
    left(`nom_banco`, 80) as `nom_banco`,
    left(`Pais`, 20) as `Pais`,
    left(`Categoria`, 2) as `Categoria`
from
    {{ ref('int_nrp_51_02_deposito_extranjero') }}
