with

account_balances as (
    select *
    from {{ ref('int_account_balances') }}
    where 1 = 0

)
,

final as (

    select *
    from account_balances
)

select
    cast(null as string) as `id_codigo_deuda`,
    cast(null as string) as `desc_deuda`,
    cast(null as numeric) as `valor_deuda`
from
    final
