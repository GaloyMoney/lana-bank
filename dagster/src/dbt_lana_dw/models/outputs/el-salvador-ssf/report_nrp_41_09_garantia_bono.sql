with
    dummy as (

        select
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('tipo_prenda') }},
            null as {{ ident('descripción') }},
            null as {{ ident('fecha_certificado') }},
            null as {{ ident('valor_prenda') }},
            null as {{ ident('saldo_prenda') }},
            null as {{ ident('cod_almacenadora') }}

    )

select *
from dummy
where false
