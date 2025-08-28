SELECT
    JSON_VALUE(data, '$.account_set_id') as account_set_id,
    JSON_VALUE(data, '$.member_account_id') as member_account_id,
    CAST(JSON_VALUE(data, '$.transitive') as BOOLEAN) as transitive,
    CAST(JSON_VALUE(data, '$.created_at') as TIMESTAMP) as created_at,
    _sdc_extracted_at as _sdc_extracted_at,
    _sdc_deleted_at as _sdc_deleted_at,
    _sdc_received_at as _sdc_received_at,
    _sdc_batched_at as _sdc_batched_at,
    _sdc_table_version as _sdc_table_version,
    _sdc_sequence as _sdc_sequence,
 from {{ source("lana", "public_cala_account_set_member_accounts") }}
