with raw_stg_core_approval_process_events_rollup as (select * from {{ source("lana", "core_approval_process_events_rollup")}} )
select
    s.id as approval_process_id,
    s.*,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_approval_process_events_rollup
