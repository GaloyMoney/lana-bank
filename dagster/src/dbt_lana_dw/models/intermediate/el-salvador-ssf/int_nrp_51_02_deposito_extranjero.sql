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
    cast(null as string) as `id_codigo_banco`,
    cast(null as string) as `nom_banco`,
    cast(null as string) as `Pais`,
    cast(null as string) as `Categoria`,
    cast(null as numeric) as `Valor`
from
    final
