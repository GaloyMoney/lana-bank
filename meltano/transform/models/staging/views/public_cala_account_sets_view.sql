SELECT
    JSON_VALUE(data, '$.id') as id,
    JSON_VALUE(data, '$.journal_id') as journal_id,
    JSON_VALUE(data, '$.name') as name,
    JSON_VALUE(data, '$.external_id') as external_id,
    JSON_VALUE(data, '$.data_source_id') as data_source_id,
    CAST(JSON_VALUE(data, '$.created_at') as TIMESTAMP) as created_at,
    _sdc_extracted_at as _sdc_extracted_at,
    _sdc_deleted_at as _sdc_deleted_at,
    _sdc_received_at as _sdc_received_at,
    _sdc_batched_at as _sdc_batched_at,
    _sdc_table_version as _sdc_table_version,
    _sdc_sequence as _sdc_sequence,
 from {{ source("lana", "public_cala_account_sets") }}
