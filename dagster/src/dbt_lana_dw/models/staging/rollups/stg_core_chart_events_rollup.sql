{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_chart_events_rollup as (
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
        from {{ source("lana", "core_chart_events_rollup") }}
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
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_chart_events_rollup
