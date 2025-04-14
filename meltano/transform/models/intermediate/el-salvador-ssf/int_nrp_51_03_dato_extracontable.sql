with

off_balances as (
    select *
    from {{ ref('int_account_balances') }}
    where 1 = 0

)
,

final as (

    select *
    from off_balances
)

select
    'TODO' as `id_codigo_extracontable`,
    'TODO' as `desc_extra_contable`,
    7060.0 as `Valor`
from
    final
