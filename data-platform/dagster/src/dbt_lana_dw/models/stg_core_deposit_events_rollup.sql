with raw_core_deposit_events_rollup as (select * from {{ source("lana", "core_deposit_events_rollup")}} )
select
    id as deposit_id,
    version,
    created_at,
    modified_at,
    amount,
    deposit_account_id,
    public_id,
    reference,
    status,
    ledger_tx_ids,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_core_deposit_events_rollup
