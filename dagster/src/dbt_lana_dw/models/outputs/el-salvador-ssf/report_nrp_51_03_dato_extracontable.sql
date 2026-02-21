select
    cast(round(`Valor`, 2) as string) as `Valor`,
    left({{ ident('id_codigo_extracontable') }}, 10) as {{ ident('id_codigo_extracontable') }},
    left({{ ident('desc_extra_contable') }}, 80) as {{ ident('desc_extra_contable') }}
from {{ ref("int_nrp_51_03_dato_extracontable") }}
