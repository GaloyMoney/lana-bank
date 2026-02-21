with
    dummy as (

        select
            null as {{ ident('num_referencia') }},
            null as {{ ident('cod_cartera') }},
            null as {{ ident('cod_activo') }},
            null as {{ ident('nit_fiador_codeudor') }},
            null as {{ ident('fiador_codeudor') }}

    )

select *
from dummy
where false
