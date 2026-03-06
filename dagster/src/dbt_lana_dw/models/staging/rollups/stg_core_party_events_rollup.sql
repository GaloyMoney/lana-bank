{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_party_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            event_type,
            customer_type,
            email,
            personal_info,
            telegram_handle,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_party_events_rollup") }}
    )
select
    id as party_id,
    version,
    created_at,
    modified_at,
    event_type,
    customer_type,
    email,
    personal_info,
    telegram_handle,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_party_events_rollup
