with
    dummy as (

        select
            null as {{ ident('nit_deudor') }},
            null as {{ ident('nit_socio') }},
            null as {{ ident('porcentaje_participacion') }}

    )

select *
from dummy
where false
