{{ config(materialized='table') }}

select
    cast(format('%.2f', round(`valor`, 2)) as string) as `valor`,
    right(`id_codigo_cuenta`, 10) as `id_codigo_cuenta`,
    left(regexp_replace(`nom_cuenta`, r'[&<>"]', '_'), 80) as `nom_cuenta`
from
    {{ ref('int_nrp_51_01_saldo_cuenta') }}
