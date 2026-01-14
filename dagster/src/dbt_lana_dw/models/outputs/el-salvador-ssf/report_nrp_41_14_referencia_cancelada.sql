with
    dummy as (

        select
            null as `cod_cartera`,
            null as `cod_activo`,
            null as `num_referencia`,
            null as `cod_cartera_canc`,
            null as `cod_activo_canc`,
            null as `num_referencia_canc`,
            null as `pago_capital`,
            null as `pago_interes`,
            null as `saldo_total_interes`,
            null as `fecha_cancelacion`

    )

select *
from dummy
where false
