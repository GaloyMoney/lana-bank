-- Wrapper model that converts JSON array strings from seed to SQL ARRAY
-- Uses cross-platform macro for JSON array extraction
select
    order_by,
    title,
    eng_title,
    {{ array_from_json_strings('source_account_codes', '') }} as source_account_codes
from {{ ref("static_ncf_01_01_account_config_seed") }}
order by order_by
