with raw_core_withdrawal_events_rollup as (select * from {{ source("lana", "core_withdrawal_events_rollup")}} )
select
    id as withdrawal_id,
    version,
    created_at,
    modified_at,
    amount,
    approval_process_id,
    approved,
    deposit_account_id,
    public_id,
    reference,
    status,
    ledger_tx_ids,
    is_approval_process_concluded,
    is_cancelled,
    is_confirmed,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_core_withdrawal_events_rollup


