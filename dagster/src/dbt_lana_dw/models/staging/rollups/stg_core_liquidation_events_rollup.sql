{{
    config(
        unique_key=["id", "version"],
    )
}}

with
    raw_stg_core_liquidation_events_rollup as (
        select
            id,
            version,
            created_at,
            modified_at,
            amount,
            credit_facility_id,
            current_price,
            expected_to_receive,
            initially_estimated_to_liquidate,
            initially_expected_to_receive,
            outstanding,
            payment_id,
            to_liquidate_at_current_price,
            trigger_price,
            is_completed,
            _dlt_load_id,
            _dlt_id
        from {{ source("lana", "core_liquidation_events_rollup") }}
    )
select
    id as liquidation_id,
    version,
    created_at,
    modified_at,
    credit_facility_id,
    current_price,
    expected_to_receive,
    initially_estimated_to_liquidate,
    initially_expected_to_receive,
    outstanding,
    payment_id,
    to_liquidate_at_current_price,
    trigger_price,
    is_completed,
    timestamp_micros(
        cast(cast(_dlt_load_id as decimal) * 1e6 as int64)
    ) as loaded_to_dw_at
from raw_stg_core_liquidation_events_rollup
