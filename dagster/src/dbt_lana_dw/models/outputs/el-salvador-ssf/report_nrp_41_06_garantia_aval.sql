with
    dummy as (

        select
            null as `Nombre`,
            null as {{ ident('num_referencia') }},
            null as {{ ident('cod_cartera') }},
            null as {{ ident('cod_activo') }},
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('cod_banco') }},
            null as {{ ident('monto_aval') }},
            null as {{ ident('fecha_otorgamiento') }},
            null as {{ ident('fecha_vencimiento') }}

    )

select *
from dummy
where false
