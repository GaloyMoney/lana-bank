with
    dummy as (

        select
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('valor_garantia') }},
            null as {{ ident('valor_porcentual') }},
            null as {{ ident('tipo_fondo') }},
            null as {{ ident('estado') }}

    )

select *
from dummy
where false
