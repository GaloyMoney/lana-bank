select
    id,
    code,
    name,
    normal_balance_type,
    null as latest_values,  -- TODO: need fixing, where did old latest_values go which held "$.config.is_account_set" flag
    created_at

from {{ source("lana", "public_cala_accounts_view") }}
