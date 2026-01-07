-- Wrapper model that converts JSON array strings from seed to BigQuery ARRAY<STRING>
select
    order_by,
    title,
    eng_title,
    array(
        select json_value(item, '$')
        from unnest(json_query_array(sum_account_codes, '$')) as item
    ) as sum_account_codes,
    array(
        select json_value(item, '$')
        from unnest(json_query_array(diff_account_codes, '$')) as item
    ) as diff_account_codes
from {{ ref("static_nrp_28_01_account_config_seed") }}
order by order_by
