select
    left({{ ident('id_codigo_banco') }}, 10) as {{ ident('id_codigo_banco') }},
    left({{ ident('nom_banco') }}, 80) as {{ ident('nom_banco') }},
    left(`Pais`, 20) as `Pais`,
    left({{ ident('categoria') }}, 2) as {{ ident('categoria') }},
    cast(round({{ ident('valor_aval_fianza') }}, 2) as string) as {{ ident('valor_aval_fianza') }}
from {{ ref("int_nrp_51_06_aval_garantizado") }}
