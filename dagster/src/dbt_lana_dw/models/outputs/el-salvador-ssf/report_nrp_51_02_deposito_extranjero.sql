select
    left({{ ident('id_codigo_banco') }}, 10) as {{ ident('id_codigo_banco') }},
    left({{ ident('nom_banco') }}, 80) as {{ ident('nom_banco') }},
    left(`Pais`, 20) as `Pais`,
    left(`Categoria`, 2) as `Categoria`,
    cast(round(`Valor`, 2) as string) as `Valor`
from {{ ref("int_nrp_51_02_deposito_extranjero") }}
