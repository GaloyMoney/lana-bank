with
    dummy as (

        select
            null as {{ ident('nit_deudor') }},
            null as {{ ident('nit_miembro') }},
            null as {{ ident('cod_cargo') }},
            null as {{ ident('fecha_inicial_jd') }},
            null as {{ ident('fecha_final_jd') }},
            null as {{ ident('numero_credencial') }}

    )

select *
from dummy
where false
