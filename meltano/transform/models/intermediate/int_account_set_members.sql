select
    account_set_id,
    member_account_id as member_id,
    'Account' as member_type

from {{ ref('stg_account_set_member_accounts') }}

union all

select
    account_set_id,
    member_account_id as member_id,
    'AccountSet' as member_type

from {{ ref('stg_account_set_member_accounts') }}
