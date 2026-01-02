{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_collateral_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            abs_diff,
            account_id,
            action,
            collateral_amount,
            credit_facility_id,
            custody_wallet_id,
            pending_credit_facility_id,
            ledger_tx_ids,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_collateral_events_rollup") }}
    )
select
    id as collateral_id,
    version,
    created_at,
    modified_at,
    abs_diff,
    account_id,
    action,
    collateral_amount,
    credit_facility_id,
    custody_wallet_id,
    pending_credit_facility_id,
    ledger_tx_ids,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_collateral_events_rollup
