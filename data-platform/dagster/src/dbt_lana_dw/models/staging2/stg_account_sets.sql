select
    id,
    journal_id,
    name as set_name,
    created_at

from {{ source("lana", "public_cala_account_sets_view") }}
