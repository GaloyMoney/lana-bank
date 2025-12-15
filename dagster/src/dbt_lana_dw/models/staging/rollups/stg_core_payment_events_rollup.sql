{{ config(
    unique_key = ['id', 'version'],
) }}

with raw_stg_core_payment_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        amount,
        credit_facility_id,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_payment_events_rollup")}}
)
select
    id as payment_id,
    version,
    created_at,
    modified_at,
    amount,
    credit_facility_id,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_payment_events_rollup
