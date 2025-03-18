{% if target.type == 'bigquery' %}
with set_hierarchy_strings as (

    select
        expanded.account_set_id,
        expanded.member_id,
        expanded.member_type,
        string_agg(
            set_name, ':'
            order by o
        ) as set_hierarchy_string

    from {{ ref('int_account_sets_expanded') }} as expanded,
        unnest(set_hierarchy) as parent_set_id with offset as o

    inner join
        {{ ref('int_account_sets') }} as account_sets
        on parent_set_id = account_sets.account_set_id

    group by account_set_id, member_id, member_type

)
{% elif target.type == 'snowflake' %}
with set_hierarchy_strings as (

    select
        expanded.account_set_id,
        expanded.member_id,
        expanded.member_type,
        LISTAGG(
            set_name, ':'
        ) WITHIN GROUP (ORDER BY parent_set.index) as set_hierarchy_string

    from {{ ref('int_account_sets_expanded') }} as expanded,
        table(flatten(input=>set_hierarchy)) as parent_set

    inner join
        {{ ref('int_account_sets') }} as account_sets
        on parent_set.value = account_sets.account_set_id

    group by expanded.account_set_id, member_id, member_type

)
{% endif %}

select
    member_id as id_codigo_cuenta,
    set_hierarchy_string || ':' || account_name as nom_cuenta,
    coalesce(case
        when normal_balance_type = 'credit' then settled_cr - settled_dr
        when normal_balance_type = 'debit' then settled_dr - settled_cr
    end, 0) as valor

from set_hierarchy_strings

left join {{ ref('int_account_sets') }} using (account_set_id)

left join {{ ref('int_accounts') }} as accounts
    on accounts.account_id = member_id

left join {{ ref('int_account_balances') }} as balances
    on balances.account_id = member_id

where
    member_type = 'Account'
