with chart as (
    select
        code,
        dotted_code,
        name,
        account_set_id,
    from {{ ref('int_core_chart_of_accounts') }}
),

balances as (
    select
        *,
    from {{ ref('int_account_sets_expanded_with_balances') }}
),


final as (
    select
        c.code,
        c.dotted_code,
        c.name,
        c.account_set_id,
        coalesce(sum(balance), 0) as balance
    from chart as c
    left join balances using (account_set_id)
    group by code, dotted_code, name, account_set_id
)

select * from final
