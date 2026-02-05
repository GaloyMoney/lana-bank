{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_payment_allocation_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            payment_holding_account_id,
            amount,
            beneficiary_id as credit_facility_id,
            effective,
            ledger_tx_id,
            obligation_id,
            payment_allocation_idx,
            obligation_type,
            payment_id,
            receivable_account_id,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_payment_allocation_events_rollup") }}
    )
select
    id as payment_allocation_id,
    version,
    created_at,
    modified_at,
    payment_holding_account_id,
    amount,
    credit_facility_id,
    effective,
    ledger_tx_id,
    obligation_id,
    payment_allocation_idx,
    obligation_type,
    payment_id,
    receivable_account_id,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_payment_allocation_events_rollup
