with
    dummy as (

        select
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('monto_poliza') }},
            null as {{ ident('fecha_inicial') }},
            null as {{ ident('fecha_final') }},
            null as {{ ident('nombre_asegurado') }},
            null as {{ ident('monto_reserva') }},
            null as {{ ident('valor_garantia') }}

    )

select *
from dummy
where false
