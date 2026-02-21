with

    account_balances as (select * from {{ ref("int_account_balances") }} where 1 = 0),

    final as (select * from account_balances)

select
    cast(null as string) as {{ ident('id_codigo_deuda') }},
    cast(null as string) as {{ ident('desc_deuda') }},
    cast(null as numeric) as {{ ident('valor_deuda') }}
from final
