-- Wrapper model that converts JSON array strings from seed to SQL ARRAY
-- Uses cross-platform macro for JSON array extraction
select
    order_by,
    title,
    eng_title,
    {{ array_from_json_strings('sum_account_codes') }} as sum_account_codes,
    {{ array_from_json_strings('diff_account_codes') }} as diff_account_codes
from {{ ref("static_nrp_28_01_account_config_seed") }}
order by order_by
