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
    cast(null as string) as `id_codigo_titulo_extranjero`,
    cast(null as string) as `desc_tv_extranj`,
    cast(null as numeric) as `valor_tv_extranj`
from
    final
