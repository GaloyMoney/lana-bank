with
    dummy as (

        select
            null as {{ ident('cod_cartera') }},
            null as {{ ident('cod_activo') }},
            null as {{ ident('num_referencia') }},
            null as {{ ident('codigo_unidad') }},
            null as {{ ident('cantidad_unidad') }}

    )

select *
from dummy
where false
