with raw_stg_core_public_ids as (select * from {{ source("lana", "core_public_ids")}} )
select
    id,
    target_id,
    created_at,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_public_ids
