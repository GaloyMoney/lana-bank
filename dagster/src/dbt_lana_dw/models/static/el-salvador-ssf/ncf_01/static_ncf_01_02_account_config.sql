-- Wrapper model that converts JSON array strings from seed to BigQuery ARRAY<STRING>
select
    order_by,
    title,
    eng_title,
    array(
        select json_value(item, '$')
        from unnest(json_query_array(source_account_spaced_codes, '$')) as item
    ) as source_account_spaced_codes
from {{ ref("static_ncf_01_02_account_config_seed") }}
order by order_by
