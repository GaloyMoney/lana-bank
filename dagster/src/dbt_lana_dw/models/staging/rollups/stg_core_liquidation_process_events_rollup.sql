with raw_stg_core_liquidation_process_events_rollup as (
    select
        id,
        version,
        created_at,
        modified_at,
        credit_facility_id,
        effective,
        in_liquidation_account_id,
        initial_amount,
        ledger_tx_id,
        obligation_id,
        is_completed,
        _dlt_load_id,
        _dlt_id
    from {{ source("lana", "core_liquidation_process_events_rollup")}}
)
select
    id as liquidation_process_id,
    version,
    created_at,
    modified_at,
    credit_facility_id,
    effective,
    in_liquidation_account_id,
    initial_amount,
    ledger_tx_id,
    obligation_id,
    is_completed,
    TIMESTAMP_MICROS(CAST(CAST(_dlt_load_id AS DECIMAL) * 1e6 as INT64 )) as loaded_to_dw_at
from raw_stg_core_liquidation_process_events_rollup
