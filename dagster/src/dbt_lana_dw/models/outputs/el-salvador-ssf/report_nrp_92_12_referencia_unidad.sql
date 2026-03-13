with
    dummy as (

        select
            null as `cod_cartera`,
            null as `cod_activo`,
            null as `num_referencia`,
            null as `codigo_unidad`,
            null as `cantidad_unidad`

    )

select *
from dummy
where false
