select
    left({{ ident('id_codigo_titulo_extranjero') }}, 10) as {{ ident('id_codigo_titulo_extranjero') }},
    left({{ ident('desc_tv_extranj') }}, 254) as {{ ident('desc_tv_extranj') }},
    cast(round({{ ident('valor_tv_extranj') }}, 2) as string) as {{ ident('valor_tv_extranj') }}
from {{ ref("int_nrp_51_04_titulo_valor_extranjero") }}
