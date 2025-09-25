{{ config(materialized='table') }}

with dummy as (

    select
        null as `identificacion_garantia`,
        null as `valor_garantia`,
        null as `valor_porcentual`,
        null as `tipo_fondo`,
        null as `estado`


)

select *
from dummy
where false
