with
    dummy as (

        select
            null as `Nombre`,
            null as `num_referencia`,
            null as `cod_cartera`,
            null as `cod_activo`,
            null as `identificacion_garantia`,
            null as `cod_banco`,
            null as `monto_aval`,
            null as `fecha_otorgamiento`,
            null as `fecha_vencimiento`

    )

select *
from dummy
where false
