with raw_stg_core_interest_accrual_cycle_events_rollup as (select * from {{ source("lana", "core_interest_accrual_cycle_events_rollup")}} )
select
    id as interest_accrual_cycle_id,
    facility_id as credit_facility_id,
    *,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_interest_accrual_cycle_events_rollup
