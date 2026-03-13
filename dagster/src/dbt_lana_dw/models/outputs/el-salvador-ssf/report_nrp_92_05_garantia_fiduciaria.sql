with
    dummy as (

        select
            null as `num_referencia`,
            null as `cod_cartera`,
            null as `cod_activo`,
            null as `documento_fiador`,
            null as `fiador`

    )

select *
from dummy
where false
