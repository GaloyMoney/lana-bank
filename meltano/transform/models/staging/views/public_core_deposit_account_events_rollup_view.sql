SELECT
    JSON_VALUE(data, '$.id') as id,
    CAST(JSON_VALUE(data, '$.version') as INT64) as version,
    CAST(JSON_VALUE(data, '$.created_at') as TIMESTAMP) as created_at,
    CAST(JSON_VALUE(data, '$.modified_at') as TIMESTAMP) as modified_at,
    JSON_VALUE(data, '$.account_holder_id') as account_holder_id,
    -- TODO: need to fix this
    JSON_VALUE(data, '$.frozen_deposit_account_id') as frozen_deposit_account_id,
    JSON_VALUE(data, '$.ledger_account_id') as ledger_account_id,
    JSON_VALUE(data, '$.public_id') as public_id,
    JSON_VALUE(data, '$.status') as status,
    JSON_VALUE(data, '$.audit_entry_ids') as audit_entry_ids,
    _sdc_extracted_at as _sdc_extracted_at,
    _sdc_deleted_at as _sdc_deleted_at,
    _sdc_received_at as _sdc_received_at,
    _sdc_batched_at as _sdc_batched_at,
    _sdc_table_version as _sdc_table_version,
    _sdc_sequence as _sdc_sequence,
from {{ source("lana", "public_core_deposit_account_events_rollup") }}
