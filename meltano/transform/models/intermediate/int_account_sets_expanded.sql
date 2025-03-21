with recursive account_set_members as (

    select distinct
        account_set_id,
        member_id,
        member_type

    from {{ ref('int_account_set_members') }}

),

account_set_members_expanded as (

    select
        account_set_id,
        member_id,
        member_type,
        [account_set_id] as set_hierarchy
    from account_set_members

    union all

    select
        l.account_set_id,
        r.member_id,
        r.member_type,
        array_concat(l.set_hierarchy, [r.account_set_id]) as set_hierarchy
    from account_set_members_expanded as l
    left join account_set_members as r
        on l.member_id = r.account_set_id
    where l.member_type = 'AccountSet'

)

select
    account_set_id,
    member_id,
    member_type,
    any_value(set_hierarchy having max array_length(set_hierarchy))
        as set_hierarchy

from account_set_members_expanded

where member_id is not null

group by account_set_id, member_id, member_type
