SELECT
    JSON_VALUE(data, '$.id') as id,
    CAST(JSON_VALUE(data, '$.sequence') as INT64) as sequence,
    JSON_VALUE(data, '$.event_type') as event_type,
    JSON_VALUE(data, '$.event') as event,
    CAST(JSON_VALUE(data, '$.recorded_at') as TIMESTAMP) as recorded_at,
    _sdc_extracted_at as _sdc_extracted_at,
    _sdc_deleted_at as _sdc_deleted_at,
    _sdc_received_at as _sdc_received_at,
    _sdc_batched_at as _sdc_batched_at,
    _sdc_table_version as _sdc_table_version,
    _sdc_sequence as _sdc_sequence,
 from {{ source("lana", "public_core_chart_events") }}
