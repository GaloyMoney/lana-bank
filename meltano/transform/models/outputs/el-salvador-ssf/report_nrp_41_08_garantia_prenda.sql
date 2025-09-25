{{ config(materialized='table') }}

with dummy as (

    select
        null as `identificacion_garantia`,
        null as `denominacion_titulo`,
        null as `local_extranjera`,
        null as `monto_inversion`,
        null as `fecha_vencimiento`,
        null as `clasificación`,
        null as `nombre_clasificadora`


)

select *
from dummy
where false
