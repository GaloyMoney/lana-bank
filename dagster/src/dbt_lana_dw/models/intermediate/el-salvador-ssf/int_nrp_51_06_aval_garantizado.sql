with

    account_balances as (select * from {{ ref("int_account_balances") }} where 1 = 0),

    final as (select * from account_balances)

select
    cast(null as string) as {{ ident('id_codigo_banco') }},
    cast(null as string) as {{ ident('nom_banco') }},
    cast(null as string) as `Pais`,
    cast(null as string) as {{ ident('categoria') }},
    cast(null as numeric) as {{ ident('valor_aval_fianza') }}
from final
