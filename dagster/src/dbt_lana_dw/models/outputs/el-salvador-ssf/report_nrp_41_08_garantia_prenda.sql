with
    dummy as (

        select
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('denominacion_titulo') }},
            null as {{ ident('local_extranjera') }},
            null as {{ ident('monto_inversion') }},
            null as {{ ident('fecha_vencimiento') }},
            null as {{ ident('clasificación') }},
            null as {{ ident('nombre_clasificadora') }}

    )

select *
from dummy
where false
