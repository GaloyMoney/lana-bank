{{ config(materialized='table') }}

with dummy as (
    select
        null as `identificacion_garantia`,
        null as `numero_registro`,
        null as `nit_propietario`,
        null as `fecha_registro`,
        null as `estado`,
        null as `cod_ubicacion`,
        null as `descripción`,
        null as `fecha_valuo`,
        null as `valor_pericial`,
        null as `valor_contractual`,
        null as `valor_mercado`,
        null as `grado_hipoteca`,
        null as `direccion_gtia`,
        null as `cod_perito`,
        null as `nombre_perito`,
        null as `tipo_perito`

)

select *
from dummy
where false
