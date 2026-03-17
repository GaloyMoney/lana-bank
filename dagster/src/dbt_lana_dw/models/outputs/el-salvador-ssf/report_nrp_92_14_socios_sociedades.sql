with
    dummy as (

        select
            null as `documento_deudor`,
            null as `documento_socio`,
            null as `porcentaje_participacion`

    )

select *
from dummy
where false
