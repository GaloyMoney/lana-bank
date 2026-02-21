with
    dummy as (
        select
            null as {{ ident('identificacion_garantia') }},
            null as {{ ident('numero_registro') }},
            null as {{ ident('nit_propietario') }},
            null as {{ ident('fecha_registro') }},
            null as {{ ident('estado') }},
            null as {{ ident('cod_ubicacion') }},
            null as {{ ident('descripción') }},
            null as {{ ident('fecha_valuo') }},
            null as {{ ident('valor_pericial') }},
            null as {{ ident('valor_contractual') }},
            null as {{ ident('valor_mercado') }},
            null as {{ ident('grado_hipoteca') }},
            null as {{ ident('direccion_gtia') }},
            null as {{ ident('cod_perito') }},
            null as {{ ident('nombre_perito') }},
            null as {{ ident('tipo_perito') }}

    )

select *
from dummy
where false
