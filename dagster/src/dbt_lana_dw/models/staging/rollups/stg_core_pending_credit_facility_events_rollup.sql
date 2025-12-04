{{ config(
    unique_key = ['id', 'version'],
) }}

with raw_stg_core_pending_credit_facility_events_rollup as (select * from {{ source("lana", "core_pending_credit_facility_events_rollup")}} )
select
    id as pending_credit_facility_id,
    *,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_pending_credit_facility_events_rollup
