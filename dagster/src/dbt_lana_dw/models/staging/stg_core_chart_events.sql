{{ config(
    unique_key = ['id', 'sequence'],
) }}

with raw_stg_core_chart_events as (
    select
        id,
        sequence,
        event_type,
        event,
        context,
        recorded_at,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_chart_events")}}
)
select
    id,
    sequence,
    event_type,
    event,
    recorded_at,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_chart_events
