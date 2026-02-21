with
    dummy as (

        select
            null as {{ ident('cod_cartera') }},
            null as {{ ident('cod_activo') }},
            null as {{ ident('num_referencia') }},
            null as {{ ident('codigo_gasto') }},
            null as {{ ident('tipo_gasto') }},
            null as {{ ident('monto_gasto') }}

    )

select *
from dummy
where false
