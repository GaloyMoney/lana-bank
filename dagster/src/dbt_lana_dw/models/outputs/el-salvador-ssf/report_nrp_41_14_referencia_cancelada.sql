with
    dummy as (

        select
            null as {{ ident('cod_cartera') }},
            null as {{ ident('cod_activo') }},
            null as {{ ident('num_referencia') }},
            null as {{ ident('cod_cartera_canc') }},
            null as {{ ident('cod_activo_canc') }},
            null as {{ ident('num_referencia_canc') }},
            null as {{ ident('pago_capital') }},
            null as {{ ident('pago_interes') }},
            null as {{ ident('saldo_total_interes') }},
            null as {{ ident('fecha_cancelacion') }}

    )

select *
from dummy
where false
