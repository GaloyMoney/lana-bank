with raw_stg_core_chart_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        account_set_id,
        closed_as_of,
        name,
        reference,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_chart_events_rollup")}}
)
select
    id as chart_id,
    version,
    created_at,
    modified_at,
    account_set_id,
    closed_as_of,
    name,
    reference,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_chart_events_rollup
