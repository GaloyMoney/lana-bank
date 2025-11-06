with raw_stg_core_interest_accrual_cycle_events_rollup as (select * from {{ source("lana", "core_interest_accrual_cycle_events_rollup")}} )
select
    s.id as interest_accrual_cycle_id,
    s.facility_id as credit_facility_id,
    s.*,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_interest_accrual_cycle_events_rollup
