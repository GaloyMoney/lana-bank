select
    cast(format('%.2f', round({{ ident('valor') }}, 2)) as string) as {{ ident('valor') }},
    right({{ ident('id_codigo_cuenta') }}, 10) as {{ ident('id_codigo_cuenta') }},
    upper(left(regexp_replace({{ ident('nom_cuenta') }}, r'[&<>"]', '_'), 80)) as {{ ident('nom_cuenta') }}
from {{ ref("int_nrp_51_01_saldo_cuenta") }}
