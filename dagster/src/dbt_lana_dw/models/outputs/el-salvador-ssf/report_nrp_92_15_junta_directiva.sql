with
    dummy as (

        select
            null as `documento_deudor`,
            null as `documento_miembro`,
            null as `cod_cargo`,
            null as `fecha_inicial_jd`,
            null as `fecha_final_jd`,
            null as `numero_credencial`

    )

select *
from dummy
where false
