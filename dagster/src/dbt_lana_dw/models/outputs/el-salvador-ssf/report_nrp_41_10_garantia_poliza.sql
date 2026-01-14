with dummy as (

    select
        null as `identificacion_garantia`,
        null as `monto_poliza`,
        null as `fecha_inicial`,
        null as `fecha_final`,
        null as `nombre_asegurado`,
        null as `monto_reserva`,
        null as `valor_garantia`

)

select *
from dummy
where false
