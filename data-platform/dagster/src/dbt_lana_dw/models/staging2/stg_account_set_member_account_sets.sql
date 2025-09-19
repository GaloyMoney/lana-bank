select
    account_set_id,
    member_account_set_id,
    created_at

from {{ source("lana", "public_cala_account_set_member_account_sets_view") }}
